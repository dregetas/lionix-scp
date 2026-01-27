use std::process::{Command, Stdio};
use std::io::Write;

pub struct LocalServer {
    pub path: String,
    pub ram: String,
    pub process: Option<std::process::Child>,
}

impl LocalServer {
    pub fn new(path: String, ram: String) -> Self {
        Self { path, ram, process: None }
    }
}

impl super::controller::ServerController for LocalServer {
    fn start(&mut self) -> Result<(), String> {
        let child = Command::new("java")
            .current_dir(&self.path)
            .args([
                format!("-Xmx{}", self.ram),
                "-jar".into(),
                "server.jar".into(),
                "nogui".into(),
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|e| e.to_string())?;

        self.process = Some(child);
        Ok(())
    }

    fn stop(&mut self) -> Result<(), String> {
        if let Some(p) = &mut self.process {
            if let Some(stdin) = &mut p.stdin {
                stdin.write_all(b"stop\n").unwrap();
            }
        }
        Ok(())
    }

    fn send_cmd(&mut self, cmd: &str) -> Result<(), String> {
        if let Some(p) = &mut self.process {
            if let Some(stdin) = &mut p.stdin {
                stdin.write_all(format!("{cmd}\n").as_bytes()).unwrap();
            }
        }
        Ok(())
    }
}
