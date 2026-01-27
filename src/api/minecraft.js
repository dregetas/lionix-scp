import { invoke } from "@tauri-apps/api/core";

export const startServer = (path, ram) =>
  invoke("start_server", { path, ram });

export const stopServer = () =>
  invoke("stop_server");

export const sendCommand = (cmd) =>
  invoke("send_command", { cmd });
