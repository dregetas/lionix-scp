// Prevents additional console window on Windows
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    fs,
    io::{BufRead, BufReader, Write},
    process::{Child, Command, Stdio},
    sync::Mutex,
    thread,
    time::Instant,
};

use once_cell::sync::OnceCell;
use regex::Regex;
use tauri::{AppHandle, Emitter};

// ---------------- GLOBALS ----------------

static SERVER_PROCESS: Mutex<Option<Child>> = Mutex::new(None);
static SERVER_STATUS: Mutex<String> = Mutex::new(String::new());
static SERVER_START: Mutex<Option<Instant>> = Mutex::new(None);
static APP_HANDLE: OnceCell<AppHandle> = OnceCell::new();

// ---------------- COMMANDS ----------------

#[tauri::command]
fn start_server() -> String {
    let mut server = SERVER_PROCESS.lock().unwrap();

    if server.is_some() {
        return "Server already running".into();
    }

    *SERVER_STATUS.lock().unwrap() = "starting".into();

    let child = Command::new("bash")
        .arg("-c")
        .arg("cd /home/haku/server && java -jar server.jar nogui")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    match child {
        Ok(mut process) => {
            // ---------- STDOUT ----------
            if let Some(stdout) = process.stdout.take() {
                thread::spawn(move || {
                    let reader = BufReader::new(stdout);

                    for line in reader.lines().flatten() {
                        if line.contains("Done (") {
                            *SERVER_STATUS.lock().unwrap() = "online".into();
                            *SERVER_START.lock().unwrap() = Some(Instant::now());
                        }

                        if let Some(app) = APP_HANDLE.get() {
                            app.emit("server-log", line).ok();
                        }
                    }
                });
            }

            // ---------- STDERR ----------
            if let Some(stderr) = process.stderr.take() {
                thread::spawn(move || {
                    let reader = BufReader::new(stderr);

                    for line in reader.lines().flatten() {
                        if let Some(app) = APP_HANDLE.get() {
                            app.emit("server-log", format!("ERROR: {}", line)).ok();
                        }
                    }
                });
            }

            *server = Some(process);
            "Server started".into()
        }
        Err(e) => {
            *SERVER_STATUS.lock().unwrap() = "offline".into();
            format!("Failed: {}", e)
        }
    }
}

#[tauri::command]
fn stop_server() -> String {
    let mut server = SERVER_PROCESS.lock().unwrap();

    if let Some(child) = server.as_mut() {
        child.kill().ok();
        *server = None;
        *SERVER_STATUS.lock().unwrap() = "offline".into();
        *SERVER_START.lock().unwrap() = None;
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
    let status = SERVER_STATUS.lock().unwrap();

    if status.is_empty() {
        "offline".into()
    } else {
        status.clone()
    }
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
                        return kb / 1024.0; // KB → MB
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
    if let Some(start) = SERVER_START.lock().unwrap().as_ref() {
        return start.elapsed().as_secs();
    }
    0
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
            request_players,
            get_players,
            get_ram_usage,
            get_uptime
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri app");
}
