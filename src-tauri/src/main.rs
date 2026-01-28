// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
  app_lib::run();
  start_server();
}
#[tauri::command]
fn start_server() {
  println!("Server started!");
}
