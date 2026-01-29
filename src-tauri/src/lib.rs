// Prevents additional console window on Windows
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    io::{BufRead, BufReader, Write},
    process::{Child, Command, Stdio},
    sync::Mutex,
    thread,
    time::Instant,
};

use once_cell::sync::{Lazy, OnceCell};
use regex::Regex;
use tauri::{AppHandle, Emitter};

// ---------------- CONFIG ----------------

const SERVER_DIR: &str = "/home/maksym/server";
const SERVER_CMD: &str = "java -jar server.jar nogui";

// ---------------- GLOBALS ----------------

static SERVER_PROCESS: Lazy<Mutex<Option<Child>>> =
    Lazy::new(|| Mutex::new(None));

static SERVER_STATUS: Lazy<Mutex<String>> =
    Lazy::new(|| Mutex::new("offline".to_string()));

static SERVER_START: Lazy<Mutex<Option<Instant>>> =
    Lazy::new(|| Mutex::new(None));

static PLAYERS: Lazy<Mutex<String>> =
    Lazy::new(|| Mutex::new("0/0".to_string()));

static LOGS: Lazy<Mutex<Vec<String>>> =
    Lazy::new(|| Mutex::new(Vec::with_capacity(1000)));

static APP_HANDLE: OnceCell<AppHandle> = OnceCell::new();

// ---------------- HELPERS ----------------

fn push_log(line: String) {
    let mut logs = LOGS.lock().unwrap();

    if logs.len() >= 1000 {
        logs.remove(0);
    }

    logs.push(line.clone());

    if let Some(app) = APP_HANDLE.get() {
        let _ = app.emit("server-log", line);
    }
}

// ---------------- COMMANDS ----------------

#[tauri::command]
fn start_server() -> String {
    let mut server = SERVER_PROCESS.lock().unwrap();

    if server.is_some() {
        return "Server already running".into();
    }

    *SERVER_STATUS.lock().unwrap() = "starting".into();

    let mut child = match Command::new("bash")
        .arg("-c")
        .arg(format!("cd {} && {}", SERVER_DIR, SERVER_CMD))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(p) => p,
        Err(e) => {
            *SERVER_STATUS.lock().unwrap() = "offline".into();
            return format!("Failed to start server: {}", e);
        }
    };

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let mut stdin = child.stdin.take();

    let player_re = Regex::new(
        r"There are (\d+) of (?:a max of )?(\d+) players online"
    ).unwrap();

    // ---------- STDOUT ----------
    if let Some(stdout) = stdout {
        thread::spawn(move || {
            let reader = BufReader::new(stdout);

            for line in reader.lines().flatten() {

                // ---- SERVER READY ----
                if line.contains("Done (") {
                    *SERVER_STATUS.lock().unwrap() = "online".into();
                    *SERVER_START.lock().unwrap() = Some(Instant::now());

                    // одразу просимо список гравців
                    if let Some(stdin) = stdin.as_mut() {
                        let _ = stdin.write_all(b"list\n");
                    }
                }

                // ---- PLAYERS ----
                if let Some(caps) = player_re.captures(&line) {
                    let value = format!("{}/{}", &caps[1], &caps[2]);
                    *PLAYERS.lock().unwrap() = value.clone();

                    if let Some(app) = APP_HANDLE.get() {
                        let _ = app.emit("players-update", value);
                    }
                }

                push_log(line);
            }
        });
    }

    // ---------- STDERR ----------
    if let Some(stderr) = stderr {
        thread::spawn(move || {
            let reader = BufReader::new(stderr);

            for line in reader.lines().flatten() {
                push_log(format!("ERROR: {}", line));
            }
        });
    }

    *server = Some(child);
    "Server started".into()
}

#[tauri::command]
fn stop_server() -> String {
    let mut server = SERVER_PROCESS.lock().unwrap();

    if let Some(child) = server.as_mut() {
        let _ = child.kill();
        *server = None;

        *SERVER_STATUS.lock().unwrap() = "offline".into();
        *SERVER_START.lock().unwrap() = None;
        *PLAYERS.lock().unwrap() = "0/0".into();

        "Server stopped".into()
    } else {
        "Server not running".into()
    }
}

#[tauri::command]
fn restart_server() -> String {
    stop_server();
    thread::sleep(std::time::Duration::from_millis(500));
    start_server()
}

#[tauri::command]
fn get_status() -> String {
    SERVER_STATUS.lock().unwrap().clone()
}

// ---------------- CONSOLE ----------------

#[tauri::command]
fn send_command(cmd: String) -> Result<(), String> {
    let mut server = SERVER_PROCESS.lock().map_err(|_| "Mutex error")?;

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

#[tauri::command]
fn get_logs() -> Vec<String> {
    LOGS.lock().unwrap().clone()
}

// ---------------- PLAYERS ----------------

#[tauri::command]
fn request_players() {
    let mut server = SERVER_PROCESS.lock().unwrap();

    if let Some(child) = server.as_mut() {
        if let Some(stdin) = child.stdin.as_mut() {
            let _ = stdin.write_all(b"list\n");
        }
    }
}

#[tauri::command]
fn get_players() -> String {
    PLAYERS.lock().unwrap().clone()
}

// ---------------- RAM (MB) ----------------

#[tauri::command]
fn get_ram_usage() -> f64 {
    let server = SERVER_PROCESS.lock().unwrap();

    if let Some(child) = server.as_ref() {
        let path = format!("/proc/{}/status", child.id());

        if let Ok(content) = std::fs::read_to_string(path) {
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

// ---------------- UPTIME ----------------

#[tauri::command]
fn get_uptime() -> u64 {
    SERVER_START
        .lock()
        .unwrap()
        .map(|s| s.elapsed().as_secs())
        .unwrap_or(0)
}

// ---------------- APP ----------------

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            APP_HANDLE.set(app.handle().clone()).ok();
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_server,
            stop_server,
            restart_server,
            get_status,
            send_command,
            get_logs,
            request_players,
            get_players,
            get_ram_usage,
            get_uptime
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri app");
}
