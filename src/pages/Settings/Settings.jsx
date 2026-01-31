import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./Settings.css";

const Settings = ({ onClose }) => {
  const [theme, setTheme] = useState("dark");
  const [path, setPath] = useState("");
  const [lang, setLang] = useState("en");
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");

  useEffect(() => {
    // Load settings on mount
    const loadSettings = async () => {
      try {
        // Load theme and language from localStorage (UI preferences)
        setTheme(localStorage.getItem("theme") || "dark");
        setLang(localStorage.getItem("lang") || "en");
        
        // Load server path from Rust backend
        const serverPath = await invoke("get_server_path");
        setPath(serverPath);
      } catch (err) {
        console.error("Failed to load settings:", err);
        setError("Failed to load server path");
      } finally {
        setLoading(false);
      }
    };

    loadSettings();
  }, []);

  const handleSave = async () => {
    try {
      setError("");
      
      // Save UI preferences to localStorage
      localStorage.setItem("theme", theme);
      localStorage.setItem("lang", lang);
      
      // Save server path to Rust backend
      await invoke("set_server_path", { path });
      
      onClose();
    } catch (err) {
      setError(err.toString());
      console.error("Failed to save settings:", err);
    }
  };

  if (loading) {
    return (
      <div className="settings-overlay" onClick={onClose}>
        <div className="settings-window" onClick={e => e.stopPropagation()}>
          <header>
            <h2>Settings</h2>
            <button className="settings-close" onClick={onClose}>✕</button>
          </header>
          <div className="settings-content">
            <p style={{ color: '#b0b8c4', textAlign: 'center' }}>Loading...</p>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="settings-overlay" onClick={onClose}>
      <div className="settings-window" onClick={e => e.stopPropagation()}>

        <header>
          <h2>Settings</h2>
          <button className="settings-close" onClick={onClose}>✕</button>
        </header>

        <div className="settings-content">
          {error && <div className="settings-error">{error}</div>}

          <div className="settings-group">
            <label>Theme</label>
            <select value={theme} onChange={e => setTheme(e.target.value)}>
              <option value="dark">Dark</option>
              <option value="light">Light</option>
            </select>
          </div>

          <div className="settings-group">
            <label>Server Folder</label>
            <input 
              type="text"
              value={path} 
              onChange={e => setPath(e.target.value)} 
              placeholder="/path/to/server"
            />
          </div>

          <div className="settings-group">
            <label>Language</label>
            <select value={lang} onChange={e => setLang(e.target.value)}>
              <option value="en">English</option>
              <option value="uk">Українська</option>
            </select>
          </div>

          <button className="settings-save" onClick={handleSave}>Save</button>
        </div>

      </div>
    </div>
  );
};

export default Settings;