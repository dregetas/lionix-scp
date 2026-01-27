use std::{
    io::{BufRead, BufReader, Write},
    process::{Command, Stdio},
    thread,
};

use tauri::{AppHandle, State, Emitter};

use crate::state::SharedServer;

#[tauri::command]
pub fn start_server(
    app: AppHandle,
    state: State<SharedServer>,
    path: String,
    ram: String,
) -> Result<(), String> {
    let mut server = state.lock().unwrap();

    if server.process.is_some() {
        return Err("Server already running".into());
    }

    let mut child = Command::new("java")
        .current_dir(path)
        .args([
            format!("-Xmx{}", ram),
            "-jar".into(),
            "server.jar".into(),
            "nogui".into(),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;

    let stdout = child.stdout.take().unwrap();
    let reader = BufReader::new(stdout);

    thread::spawn(move || {
        for line in reader.lines().flatten() {
            let _ = app.emit("mc_log", line);
        }
    });

    server.process = Some(child);
    Ok(())
}

#[tauri::command]
pub fn send_command(
    state: State<SharedServer>,
    cmd: String,
) -> Result<(), String> {
    let mut server = state.lock().unwrap();

    if let Some(proc) = &mut server.process {
        if let Some(stdin) = &mut proc.stdin {
            stdin.write_all(format!("{cmd}\n").as_bytes()).unwrap();
        }
    }

    Ok(())
}

#[tauri::command]
pub fn stop_server(
    state: State<SharedServer>,
) -> Result<(), String> {
    send_command(state, "stop".into())
}
