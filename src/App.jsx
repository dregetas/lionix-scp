import { useState } from "react";

import Dashboard from "./pages/Dashboard/Dashboard";
import Settings from "./pages/Settings/Settings";
import About from "./components/About/About";

import "./App.css";

function App() {
  const [showSettings, setShowSettings] = useState(false);
  const [showAbout, setShowAbout] = useState(false);

  return (
    <>
      <Dashboard 
        onOpenSettings={() => setShowSettings(true)}
        onOpenAbout={() => setShowAbout(true)}
      />

      {showSettings && (
        <Settings onClose={() => setShowSettings(false)} />
      )}

      {showAbout && (
        <About onClose={() => setShowAbout(false)} />
      )}
    </>
  );
}

export default App;