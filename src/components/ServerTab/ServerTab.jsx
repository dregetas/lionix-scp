import { invoke } from "@tauri-apps/api/core";
import "./ServerTab.css";

const startServer = async () => {
  await invoke("start_server");
};

const ServerTab = () => {
  const status = "offline"; // offline | starting | online
  const players = 0;
  const maxPlayers = 20;
  const ramUsed = 0;
  const ramMax = 4;

  return (
    <section className="server-tab">
      <header className="server-header">
        <h1 className="server-name">Server's Name</h1>
        <span className={`server-status ${status}`}>
          {status.toUpperCase()}
        </span>
      </header>

      <div className="server-stats">
        <div className="stat">
          <span className="label">Players</span>
          <span className="value">
            {players} / {maxPlayers}
          </span>
        </div>

        <div className="stat">
          <span className="label">RAM</span>
          <span className="value">
            {ramUsed} GB / {ramMax} GB
          </span>
        </div>
      </div>

      <div className="server-actions">
        <button className="start" onClick={startServer}>Start</button>
        <button className="stop" disabled>Stop</button>
        <button className="restart" disabled>Restart</button>
      </div>
    </section>
  );
};

export default ServerTab;
