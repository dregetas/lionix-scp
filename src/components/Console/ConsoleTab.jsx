import { useEffect, useState, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import "./ConsoleTab.css";

const ConsoleTab = ({logs, setLogs}) => {
  const [input, setInput] = useState("");
  const bottomRef = useRef(null);

  useEffect(() => {
  const unlistenPromise = listen("server-log", (event) => {
    setLogs((prev) => [...prev.slice(-500), event.payload]);
  });

    return () => {
      unlistenPromise.then((f) => f());
    };
  }, [setLogs]);


  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [logs]);

  async function send() {
    if (!input.trim()) return;
    await invoke("send_command", { cmd: input });
    setInput("");
  }

  return (
    <div className="console-page">
      <div className="console-toolbar">
        <button onClick={() => setLogs([])}>Clear</button>
      </div>
      <div className="console-log">
        {logs.map((line, i) => {
          let cls = "log-line";

          if (line.includes("ERROR")) cls += " error";
          else if (line.includes("WARN")) cls += " warn";
          else if (line.includes("INFO")) cls += " info";

          return (
          <div key={i} className={cls}>
            {line}
          </div>
          );
})}

        <div ref={bottomRef}></div>
      </div>

      <input
        className="console-input"
        placeholder="Enter command..."
        value={input}
        onChange={(e) => setInput(e.target.value)}
        onKeyDown={(e) => e.key === "Enter" && send()}
      />
    </div>
  );
};

export default ConsoleTab;
