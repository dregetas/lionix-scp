// Prevents additional console window on Windows
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Write},
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::Mutex,
    thread,
    time::Instant,
};

use once_cell::sync::{Lazy, OnceCell};
use regex::Regex;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

// ---------------- CONFIG ----------------

const SERVER_CMD: &str = "java -jar server.jar nogui";

// ---------------- CONFIG STRUCT ----------------

#[derive(Serialize, Deserialize)]
struct Config {
    server_path: String,
}

// ---------------- GLOBALS ----------------

static SERVER_DIR: Lazy<Mutex<String>> = 
    Lazy::new(|| Mutex::new(load_server_path()));

static SERVER_PROCESS: Lazy<Mutex<Option<Child>>> =
    Lazy::new(|| Mutex::new(None));

static SERVER_STATUS: Lazy<Mutex<String>> =
    Lazy::new(|| Mutex::new("offline".into()));

static SERVER_START: Lazy<Mutex<Option<Instant>>> =
    Lazy::new(|| Mutex::new(None));

static PLAYERS: Lazy<Mutex<String>> =
    Lazy::new(|| Mutex::new("0/0".into()));

static LOGS: Lazy<Mutex<Vec<String>>> =
    Lazy::new(|| Mutex::new(Vec::new()));

static APP_HANDLE: OnceCell<AppHandle> = OnceCell::new();

// ---------------- CONFIG HELPERS ----------------

fn get_config_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("minecraft-server-manager");
    std::fs::create_dir_all(&path).ok();
    path.push("config.json");
    path
}

fn load_server_path() -> String {
    if let Ok(content) = std::fs::read_to_string(get_config_path()) {
        if let Ok(config) = serde_json::from_str::<Config>(&content) {
            return config.server_path;
        }
    }
    "/home/maksym/server".to_string() // default fallback
}

fn save_server_path(path: &str) -> Result<(), String> {
    let config = Config {
        server_path: path.to_string(),
    };
    
    let json = serde_json::to_string_pretty(&config)
        .map_err(|e| e.to_string())?;
    
    std::fs::write(get_config_path(), json)
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

// ---------------- HELPERS ----------------

fn push_log(line: String) {
    let mut logs = LOGS.lock().unwrap();

    if logs.len() > 1000 {
        logs.remove(0);
    }

    logs.push(line.clone());

    if let Some(app) = APP_HANDLE.get() {
        let _ = app.emit("server-log", line);
    }
}

// ---------------- SERVER PATH COMMANDS ----------------

#[tauri::command]
fn get_server_path() -> String {
    SERVER_DIR.lock().unwrap().clone()
}

#[tauri::command]
fn set_server_path(path: String) -> Result<(), String> {
    // Validate path exists
    if !std::path::Path::new(&path).exists() {
        return Err("Path does not exist".to_string());
    }

    // Save to config file
    save_server_path(&path)?;

    // Update runtime value
    *SERVER_DIR.lock().unwrap() = path;

    Ok(())
}

// ---------------- COMMANDS ----------------

#[tauri::command]
fn start_server() -> String {
    let mut server = SERVER_PROCESS.lock().unwrap();

    if server.is_some() {
        return "Server already running".into();
    }

    *SERVER_STATUS.lock().unwrap() = "starting".into();

    let server_dir = SERVER_DIR.lock().unwrap().clone();

    let mut child = match Command::new("bash")
        .arg("-c")
        .arg(format!("cd {} && {}", server_dir, SERVER_CMD))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(p) => p,
        Err(e) => {
            *SERVER_STATUS.lock().unwrap() = "offline".into();
            return format!("Failed: {}", e);
        }
    };

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    let player_re =
        Regex::new(r"There are (\d+) of (?:a max of )?(\d+) players online")
            .unwrap();

    // ---------- STDOUT ----------
    if let Some(stdout) = stdout {
        thread::spawn(move || {
            let reader = BufReader::new(stdout);

            for line in reader.lines().flatten() {

                if line.contains("Done (") {
                    *SERVER_STATUS.lock().unwrap() = "online".into();
                    *SERVER_START.lock().unwrap() = Some(Instant::now());
                }

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

// ---------------- CONTROL ----------------

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
    let mut server = SERVER_PROCESS.lock().unwrap();

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

// ---------------- OPTIONS ----------------

#[tauri::command]
fn load_options() -> HashMap<String, String> {
    let server_dir = SERVER_DIR.lock().unwrap().clone();
    let path = format!("{}/server.properties", server_dir);

    let mut map = HashMap::new();

    if let Ok(content) = std::fs::read_to_string(path) {
        for line in content.lines() {
            if line.starts_with('#') || !line.contains('=') {
                continue;
            }

            let parts: Vec<&str> = line.splitn(2, '=').collect();
            map.insert(parts[0].to_string(), parts[1].to_string());
        }
    }

    map
}

#[tauri::command]
fn save_options(options: HashMap<String, String>) -> Result<(), String> {
    let server_dir = SERVER_DIR.lock().unwrap().clone();
    let path = format!("{}/server.properties", server_dir);

    let mut output = String::new();

    for (k, v) in options {
        output.push_str(&format!("{}={}\n", k, v));
    }

    std::fs::write(path, output).map_err(|e| e.to_string())?;

    Ok(())
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

// ---------------- VERSION ----------------

#[tauri::command]
fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
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
            get_uptime,
            load_options,
            save_options,
            get_server_path,
            set_server_path,
            get_app_version
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri app");
}