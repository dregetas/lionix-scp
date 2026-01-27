use std::process::Child;
use std::sync::Mutex;

pub struct ServerState {
    pub process: Option<Child>,
}

impl ServerState {
    pub fn new() -> Self {
        Self { process: None }
    }
}

pub type SharedServer = Mutex<ServerState>;
