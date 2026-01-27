import { startServer, stopServer } from "../api/minecraft";

export default function Controls() {
  return (
    <>
      <button onClick={() => startServer("/home/mc", "2G")}>
        ▶️ Start
      </button>
      <button onClick={stopServer}>
        ⏹ Stop
      </button>
    </>
  );
}
