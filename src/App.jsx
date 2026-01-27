import { invoke } from "@tauri-apps/api/tauri"
import { useState } from "react"

function App() {
  const [msg, setMsg] = useState("")

  async function callRust() {
    const res = await invoke("greet", { name: "Максим" })
    setMsg(res)
  }

  return (
    <div>
      <h1>Tauri + React</h1>
      <button onClick={callRust}>Викликати Rust</button>
      <p>{msg}</p>
    </div>
  )
}

export default App
