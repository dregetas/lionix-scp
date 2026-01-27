import { invoke } from "@tauri-apps/api/core";
import { useState } from "react"
import Controls from "./components/Controls";
import Console from "./components/Console";

function App() {
  return (
    <>
    <Controls />
    <Console />
    </>
  )
}

export default App
