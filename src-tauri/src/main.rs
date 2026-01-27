mod commands;
mod state;

use state::ServerState;

fn main() {
    tauri::Builder::default()
        .manage(std::sync::Mutex::new(ServerState::new()))
        .invoke_handler(tauri::generate_handler![
            commands::start_server,
            commands::send_command,
            commands::stop_server,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri app");
}
