use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    pub server_dir: String,
    pub jar: String,
    pub java: JavaConfig,
    pub ui: UiConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JavaConfig {
    pub path: String,
    pub min_ram_mb: u32,
    pub max_ram_mb: u32,
    pub args: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UiConfig {
    pub max_players: u32,
    pub auto_scroll_console: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            server_dir: "/home/maksym/server".into(),
            jar: "server.jar".into(),
            java: JavaConfig {
                path: "java".into(),
                min_ram_mb: 1024,
                max_ram_mb: 4096,
                args: vec![],
            },
            ui: UiConfig {
                max_players: 20,
                auto_scroll_console: true,
            },
        }
    }
}

pub fn load_config() -> ServerConfig {
    let path = Path::new("config/server.json");

    if !path.exists() {
        let cfg = ServerConfig::default();
        save_config(&cfg);
        return cfg;
    }

    let content = fs::read_to_string(path)
        .expect("Failed to read config");

    serde_json::from_str(&content)
        .expect("Invalid config format")
}

pub fn save_config(cfg: &ServerConfig) {
    fs::create_dir_all("config").ok();

    let json = serde_json::to_string_pretty(cfg)
        .expect("Failed to serialize config");

    fs::write("config/server.json", json)
        .expect("Failed to write config");
}
