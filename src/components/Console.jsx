import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { sendCommand } from "../api/minecraft";

export default function Console() {
  const [logs, setLogs] = useState([]);
  const [input, setInput] = useState("");

  useEffect(() => {
    const unlisten = listen("mc_log", (e) => {
      setLogs((l) => [...l.slice(-200), e.payload]);
    });

    return () => {
      unlisten.then((f) => f());
    };
  }, []);

  const submit = () => {
    sendCommand(input);
    setInput("");
  };

  return (
    <div>
      <div style={{ height: 300, overflow: "auto", background: "#000", color: "#0f0" }}>
        {logs.map((l, i) => <div key={i}>{l}</div>)}
      </div>

      <input
        value={input}
        onChange={(e) => setInput(e.target.value)}
        onKeyDown={(e) => e.key === "Enter" && submit()}
      />
    </div>
  );
}
