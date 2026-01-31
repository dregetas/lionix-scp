import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import './OptionsTab.css'

export default function OptionsTab() {
  const [motd, setMotd] = useState("");
  const [maxPlayers, setMaxPlayers] = useState("");
  const [port, setPort] = useState("");
  const [onlineMode, setOnlineMode] = useState("true");
  const [message, setMessage] = useState(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    load();
  }, []);

  async function load() {
    try {
      setLoading(true);
      const data = await invoke("load_options");

      setMotd(data["motd"] || "");
      setMaxPlayers(data["max-players"] || "");
      setPort(data["server-port"] || "");
      setOnlineMode(data["online-mode"] || "true");
    } catch (error) {
      setMessage({ type: "error", text: "Failed to load options" });
      console.error("Load error:", error);
    } finally {
      setLoading(false);
    }
  }

  async function handleSave() {
    try {
      setMessage(null);
      
      await invoke("save_options", {
        options: {
          "motd": motd,
          "max-players": maxPlayers,
          "server-port": port,
          "online-mode": onlineMode
        }
      });

      setMessage({ type: "success", text: "Options saved successfully!" });
      
      // Clear message after 3 seconds
      setTimeout(() => setMessage(null), 3000);
    } catch (error) {
      setMessage({ type: "error", text: "Failed to save options" });
      console.error("Save error:", error);
    }
  }

  if (loading) {
    return (
      <div className="options-tab">
        <h1>Server Options</h1>
        <p style={{ color: '#b0b8c4' }}>Loading options...</p>
      </div>
    );
  }

  return (
    <div className="options-tab">
      <h1>Server Options</h1>

      {message && (
        <div className={`options-message ${message.type}`}>
          {message.text}
        </div>
      )}

      <div className="options-section">
        <h3 className="options-section-title">General Settings</h3>
        
        <div className="options-group">
          <label>Message of the Day (MOTD)</label>
          <input 
            type="text" 
            value={motd} 
            onChange={e => setMotd(e.target.value)}
            placeholder="A Minecraft Server"
          />
        </div>

        <div className="options-group">
          <label>Max Players</label>
          <input 
            type="number" 
            value={maxPlayers} 
            onChange={e => setMaxPlayers(e.target.value)}
            placeholder="20"
            min="1"
          />
        </div>

        <div className="options-group">
          <label>Server Port</label>
          <input 
            type="number" 
            value={port} 
            onChange={e => setPort(e.target.value)}
            placeholder="25565"
            min="1"
            max="65535"
          />
        </div>

        <div className="options-group">
          <label>Online Mode</label>
          <select value={onlineMode} onChange={e => setOnlineMode(e.target.value)}>
            <option value="true">Enabled</option>
            <option value="false">Disabled</option>
          </select>
        </div>
      </div>

      <button onClick={handleSave}>Save Changes</button>
    </div>
  );
}