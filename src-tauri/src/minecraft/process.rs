use std::process::{Command, Stdio};
use std::io::Write;

static mut SERVER: Option<std::process::Child> = None;

#[tauri::command]
pub fn start_server(path: String, ram: String) -> Result<(), String> {
    let child = Command::new("java")
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

    unsafe { SERVER = Some(child); }
    Ok(())
}

#[tauri::command]
pub fn stop_server() -> Result<(), String> {
    unsafe {
        if let Some(server) = &mut SERVER {
            if let Some(stdin) = &mut server.stdin {
                stdin.write_all(b"stop\n").unwrap();
            }
        }
    }
    Ok(())
}
