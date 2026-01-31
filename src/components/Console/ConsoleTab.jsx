import { useEffect, useState, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import "./ConsoleTab.css";

const ConsoleTab = ({ logs, setLogs }) => {
  const [input, setInput] = useState("");
  const [autoScroll, setAutoScroll] = useState(true);
  const [commandHistory, setCommandHistory] = useState([]);
  const [historyIndex, setHistoryIndex] = useState(-1);
  const bottomRef = useRef(null);
  const logRef = useRef(null);

  // ================= INIT LOGS =================
  useEffect(() => {
    let unlisten;

    async function init() {
      try {
        // Fetch history
        const history = await invoke("get_logs");
        setLogs(history);

        // Subscribe to realtime logs
        unlisten = await listen("server-log", (event) => {
          setLogs((prev) => [...prev.slice(-999), event.payload]);
        });
      } catch (error) {
        console.error("Failed to initialize console:", error);
      }
    }

    init();

    return () => {
      if (unlisten) unlisten();
    };
  }, [setLogs]);

  // ================= AUTOSCROLL =================
  useEffect(() => {
    if (autoScroll) {
      bottomRef.current?.scrollIntoView({ behavior: "smooth" });
    }
  }, [logs, autoScroll]);

  // ================= DETECT MANUAL SCROLL =================
  useEffect(() => {
    const logElement = logRef.current;
    if (!logElement) return;

    const handleScroll = () => {
      const isAtBottom = 
        logElement.scrollHeight - logElement.scrollTop <= logElement.clientHeight + 50;
      setAutoScroll(isAtBottom);
    };

    logElement.addEventListener("scroll", handleScroll);
    return () => logElement.removeEventListener("scroll", handleScroll);
  }, []);

  // ================= SEND COMMAND =================
  async function send() {
    if (!input.trim()) return;
    
    try {
      await invoke("send_command", { cmd: input });
      setCommandHistory(prev => [...prev, input]);
      setHistoryIndex(-1);
      setInput("");
    } catch (error) {
      console.error("Failed to send command:", error);
    }
  }

  // ================= CLEAR LOGS =================
  function handleClear() {
    setLogs([]);
  }

  // ================= SCROLL TO BOTTOM =================
  function scrollToBottom() {
    setAutoScroll(true);
    bottomRef.current?.scrollIntoView({ behavior: "smooth" });
  }

  // ================= HANDLE KEY DOWN =================
  function handleKeyDown(e) {
    if (e.key === "Enter") {
      send();
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      if (commandHistory.length > 0) {
        const newIndex = historyIndex < commandHistory.length - 1 
          ? historyIndex + 1 
          : historyIndex;
        setHistoryIndex(newIndex);
        setInput(commandHistory[commandHistory.length - 1 - newIndex]);
      }
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      if (historyIndex > 0) {
        const newIndex = historyIndex - 1;
        setHistoryIndex(newIndex);
        setInput(commandHistory[commandHistory.length - 1 - newIndex]);
      } else {
        setHistoryIndex(-1);
        setInput("");
      }
    }
  }

  // ================= FORMAT LOG LINE =================
  function formatLogLine(line) {
    // Remove ANSI color codes
    const cleaned = line.replace(/\x1B\[[0-9;]*m/g, '');
    
    let type = "default";
    if (cleaned.includes("ERROR") || cleaned.includes("SEVERE")) type = "error";
    else if (cleaned.includes("WARN")) type = "warn";
    else if (cleaned.includes("INFO")) type = "info";
    else if (cleaned.includes("Done (")) type = "success";
    
    return { text: cleaned, type };
  }

  // ================= RENDER =================
  return (
    <div className="console-page">
      <div className="console-toolbar">
        <div className="console-stats">
          <span className="log-count">{logs.length} lines</span>
          {!autoScroll && (
            <button 
              className="scroll-bottom-btn"
              onClick={scrollToBottom}
            >
              ↓ Scroll to Bottom
            </button>
          )}
        </div>
        <button className="clear-btn" onClick={handleClear}>Clear</button>
      </div>

      <div className="console-log" ref={logRef}>
        {logs.length === 0 ? (
          <div className="console-empty">
            <div className="console-empty-icon">📋</div>
            <p>No logs yet. Start the server to see output.</p>
          </div>
        ) : (
          logs.map((line, i) => {
            const { text, type } = formatLogLine(line);
            return (
              <div key={i} className={`log-line log-${type}`}>
                <span className="log-index">{i + 1}</span>
                <span className="log-text">{text}</span>
              </div>
            );
          })
        )}
        <div ref={bottomRef} />
      </div>

      <div className="console-input-wrapper">
        <span className="console-prompt">&gt;</span>
        <input
          className="console-input"
          placeholder="Enter command (e.g., list, stop, help)..."
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={handleKeyDown}
        />
        <button 
          className="send-btn" 
          onClick={send}
          disabled={!input.trim()}
        >
          Send
        </button>
      </div>
    </div>
  );
};

export default ConsoleTab;