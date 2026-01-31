import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./About.css";

const About = ({ onClose }) => {
  const [version, setVersion] = useState("0.2.0");
  const [author] = useState("Maksym Sh.");

  useEffect(() => {
    // Optionally load version from Tauri backend
    const loadVersion = async () => {
      try {
        const ver = await invoke("get_app_version");
        setVersion(ver);
      } catch (err) {
        // If backend command doesn't exist, keep default version
        console.log("Using default version");
      }
    };
    
    loadVersion();
  }, []);

  const handleGitHub = () => {
    window.open("https://github.com/yourusername/lionix-scp", "_blank");
  };

  const handleReportIssue = () => {
    window.open("https://github.com/yourusername/lionix-scp/issues", "_blank");
  };

  return (
    <div className="about-overlay" onClick={onClose}>
      <div className="about-window" onClick={e => e.stopPropagation()}>

        <header>
          <h2>About</h2>
          <button className="about-close" onClick={onClose}>✕</button>
        </header>

        <div className="about-content">
          <div className="about-logo">
            <svg width="80" height="80" viewBox="0 0 80 80" fill="none">
              <rect width="80" height="80" rx="16" fill="#4CAF50"/>
              <path d="M20 40L35 55L60 25" stroke="white" strokeWidth="6" strokeLinecap="round" strokeLinejoin="round"/>
            </svg>
          </div>

          <h3>Lionix Server Control Panel</h3>
          <p className="about-version">Version {version}</p>

          <div className="about-info">
            <div className="about-row">
              <span className="about-label">Author:</span>
              <span className="about-value">{author}</span>
            </div>
            <div className="about-row">
              <span className="about-label">Framework:</span>
              <span className="about-value">Tauri + React</span>
            </div>
            <div className="about-row">
              <span className="about-label">Backend:</span>
              <span className="about-value">Rust</span>
            </div>
          </div>

          <div className="about-description">
            <p>
              A simple and efficient Minecraft server management tool. 
              Control your server, monitor players, and manage settings 
              all from one place.
            </p>
          </div>

          <div className="about-links">
            <button onClick={handleGitHub} className="about-link">
              GitHub
            </button>
            <button onClick={handleReportIssue} className="about-link">
              Report Issue
            </button>
          </div>
        </div>

        <footer className="about-footer">
          <p>© 2026 Lionix Studio. Rights not reserved.</p>
        </footer>

      </div>
    </div>
  );
};

export default About;