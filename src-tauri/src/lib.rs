#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
use config::{ServerConfig, load_config};

use std::{
    fs,
    io::{BufRead, BufReader, Write},
    process::{Child, Command, Stdio},
    sync::{Mutex, MutexGuard},
    thread,
    time::{Duration, Instant},
};

use once_cell::sync::OnceCell;
use regex::Regex;
use serde::Serialize;
use tauri::{AppHandle, Emitter};

// =====================
// GLOBAL STATE
// =====================

static APP: OnceCell<AppHandle> = OnceCell::new();

static SERVER: Mutex<Option<Child>> = Mutex::new(None);
static START_TIME: Mutex<Option<Instant>> = Mutex::new(None);
static STATUS: Mutex<ServerStatus> = Mutex::new(ServerStatus::Offline);

static CONFIG: OnceCell<ServerConfig> = OnceCell::new();

// =====================
// TYPES
// =====================

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
enum ServerStatus {
    Offline,
    Starting,
    Online,
    Stopping,
    Error,
}

fn set_status(status: ServerStatus) {
    *STATUS.lock().unwrap() = status.clone();

    if let Some(app) = APP.get() {
        let _ = app.emit("server-status", status);
    }
}

// =====================
// COMMANDS
// =====================

#[tauri::command]
fn get_status() -> ServerStatus {
    STATUS.lock().unwrap().clone()
}

#[tauri::command]
fn start_server() -> Result<(), String> {
    let mut server = SERVER.lock().unwrap();

    if server.is_some() {
        return Err("Server already running".into());
    }

    set_status(ServerStatus::Starting);

    let cfg = CONFIG
        .get()
        .ok_or("Config not loaded")?;

    // ---------- SPAWN PROCESS ----------
    let mut child = Command::new(&cfg.java.path)
        .current_dir(&cfg.server_dir)
        .arg(format!("-Xms{}M", cfg.java.min_ram_mb))
        .arg(format!("-Xmx{}M", cfg.java.max_ram_mb))
        .args(&cfg.java.args)
        .arg("-jar")
        .arg(&cfg.jar)
        .arg("nogui")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;

    // ---------- STDOUT ----------
    if let Some(stdout) = child.stdout.take() {
        thread::spawn(move || {
            let reader = BufReader::new(stdout);

            for line in reader.lines().flatten() {
                if line.contains("Done (") {
                    set_status(ServerStatus::Online);
                    *START_TIME.lock().unwrap() = Some(Instant::now());
                }

                if let Some(app) = APP.get() {
                    let _ = app.emit("server-log", line);
                }
            }
        });
    }

    // ---------- STDERR ----------
    if let Some(stderr) = child.stderr.take() {
        thread::spawn(move || {
            let reader = BufReader::new(stderr);

            for line in reader.lines().flatten() {
                if let Some(app) = APP.get() {
                    let _ = app.emit(
                        "server-log",
                        format!("ERROR: {}", line)
                    );
                }
            }
        });
    }

    *server = Some(child);
    Ok(())
}

#[tauri::command]
fn stop_server() -> Result<(), String> {
    set_status(ServerStatus::Stopping);

    let mut server = SERVER.lock().unwrap();

    if let Some(child) = server.as_mut() {
        if let Some(stdin) = child.stdin.as_mut() {
            stdin
                .write_all(b"stop\n")
                .map_err(|e| e.to_string())?;
        }

        let pid = child.id();

        drop(server); // unlock mutex before waiting

        // wait up to 15 seconds
        for _ in 0..15 {
            if !process_alive(pid) {
                cleanup_after_stop();
                return Ok(());
            }
            thread::sleep(Duration::from_secs(1));
        }

        Err("Server did not stop in time".into())
    } else {
        set_status(ServerStatus::Offline);
        Ok(())
    }
}

#[tauri::command]
fn restart_server() -> Result<(), String> {
    stop_server().ok();
    thread::sleep(Duration::from_secs(1));
    start_server()
}

#[tauri::command]
fn send_command(cmd: String) -> Result<(), String> {
    let mut server = SERVER.lock().map_err(|_| "Mutex error")?;

    if let Some(child) = server.as_mut() {
        if let Some(stdin) = child.stdin.as_mut() {
            stdin
                .write_all(format!("{}\n", cmd).as_bytes())
                .map_err(|e| e.to_string())?;
            return Ok(());
        }
    }

    Err("Server not running".into())
}

// =====================
// PLAYERS
// =====================

#[tauri::command]
fn get_players() -> String {
    let log_path = "/home/haku/server/logs/latest.log";

    if let Ok(content) = fs::read_to_string(log_path) {
        let re = Regex::new(r"There are (\d+) of (\d+) players online").unwrap();

        for line in content.lines().rev() {
            if let Some(caps) = re.captures(line) {
                return format!("{}/{}", &caps[1], &caps[2]);
            }
        }
    }

    "0/0".into()
}

#[tauri::command]
fn request_players() {
    let mut server = SERVER.lock().unwrap();

    if let Some(child) = server.as_mut() {
        if let Some(stdin) = child.stdin.as_mut() {
            let _ = stdin.write_all(b"list\n");
        }
    }
}

// =====================
// RAM (MB)
// =====================

#[tauri::command]
fn get_ram_usage() -> f64 {
    let server = SERVER.lock().unwrap();

    if let Some(child) = server.as_ref() {
        let path = format!("/proc/{}/status", child.id());

        if let Ok(content) = fs::read_to_string(path) {
            for line in content.lines() {
                if line.starts_with("VmRSS:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if let Some(kb) = parts.get(1) {
                        let kb: f64 = kb.parse().unwrap_or(0.0);
                        return kb / 1024.0;
                    }
                }
            }
        }
    }

    0.0
}

// =====================
// UPTIME
// =====================

#[tauri::command]
fn get_uptime() -> u64 {
    START_TIME
        .lock()
        .unwrap()
        .as_ref()
        .map(|t| t.elapsed().as_secs())
        .unwrap_or(0)
}

// =====================
// HELPERS
// =====================

fn process_alive(pid: u32) -> bool {
    #[cfg(target_os = "linux")]
    {
        fs::metadata(format!("/proc/{}", pid)).is_ok()
    }

    #[cfg(not(target_os = "linux"))]
    {
        true // TODO: Windows implementation
    }
}

fn cleanup_after_stop() {
    *SERVER.lock().unwrap() = None;
    *START_TIME.lock().unwrap() = None;
    set_status(ServerStatus::Offline);
}

#[tauri::command]
fn get_config() -> ServerConfig {
    CONFIG.get().unwrap().clone()
}

// =====================
// APP
// =====================

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
    APP.set(app.handle().clone()).ok();

    let cfg = load_config();
    CONFIG.set(cfg).ok();

    Ok(())
})

        .invoke_handler(tauri::generate_handler![
            start_server,
            stop_server,
            restart_server,
            get_status,
            send_command,
            get_players,
            request_players,
            get_ram_usage,
            get_uptime,
            get_config
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri app");
}
