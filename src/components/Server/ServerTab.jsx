import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./ServerTab.css";

const ServerTab = () => {
  // ================= STATE =================

  const [status, setStatus] = useState("offline");
  const [players, setPlayers] = useState("0/0");

  // RAM (MB)
  const [ramUsed, setRamUsed] = useState(0);
  const ramMax = 4096; // 4 GB

  // UPTIME
  const [uptime, setUptime] = useState(0);

  // ================= HELPERS =================

  function formatTime(sec) {
    const h = Math.floor(sec / 3600);
    const m = Math.floor((sec % 3600) / 60);
    const s = sec % 60;
    return `${h}h ${m}m ${s}s`;
  }

  // ================= RAM =================

  const ramPercent = Math.min((ramUsed / ramMax) * 100, 100);

  const ramClass =
    ramPercent > 80
      ? "ram-fill danger"
      : ramPercent > 60
      ? "ram-fill warn"
      : "ram-fill";

  async function fetchRam() {
    const res = await invoke("get_ram_usage");
    setRamUsed(Number(res));
  }

  // ================= STATUS =================

  async function fetchStatus() {
    const res = await invoke("get_status");
    setStatus(res);
  }

  // ================= PLAYERS =================

  async function fetchPlayers() {
    const res = await invoke("get_players");
    setPlayers(res);
  }

  // ================= CONTROLS =================

  async function startServer() {
    await invoke("start_server");
    fetchStatus();
  }

  async function stopServer() {
    await invoke("stop_server");
    fetchStatus();
  }

  async function restartServer() {
    await invoke("restart_server");
    fetchStatus();
  }

  // ================= UPTIME =================

  async function fetchUptime() {
    const res = await invoke("get_uptime");
    setUptime(Number(res));
  }

  // ================= INTERVALS =================

  useEffect(() => {
    // initial fetch
    fetchStatus();
    fetchPlayers();
    fetchRam();
    fetchUptime();

    const statusInterval = setInterval(fetchStatus, 1000);
    const playersInterval = setInterval(fetchPlayers, 2000);
    const ramInterval = setInterval(fetchRam, 2000);
    const uptimeInterval = setInterval(fetchUptime, 1000);

    return () => {
      clearInterval(statusInterval);
      clearInterval(playersInterval);
      clearInterval(ramInterval);
      clearInterval(uptimeInterval);
    };
  }, []);

  // ================= UI =================

  return (
    <section className="server-tab">
      <header className="server-header">
        <h1 className="server-name">Server</h1>

        <span className={`server-status ${status}`}>
          {status.toUpperCase()}
        </span>
      </header>

      <div className="server-stats">
        {/* UPTIME */}
        <div className="stat">
          <span className="label">Uptime</span>
          <span className="value">{formatTime(uptime)}</span>
        </div>

        {/* PLAYERS */}
        <div className="stat">
          <span className="label">Players</span>
          <span className="value">{players}</span>
        </div>

        {/* RAM */}
        <div className="stat">
          <span className="label">RAM</span>

          <div className="ram-bar">
            <div
              className={ramClass}
              style={{ width: `${ramPercent}%` }}
            />
          </div>

          <span className="value">
            {(ramUsed / 1024).toFixed(2)} GB / {(ramMax / 1024)} GB
          </span>
        </div>
      </div>

      <div className="server-actions">
        <button className="start" onClick={startServer}>
          Start
        </button>

        <button className="stop" onClick={stopServer}>
          Stop
        </button>

        <button className="restart" onClick={restartServer}>
          Restart
        </button>
      </div>
    </section>
  );
};

export default ServerTab;
