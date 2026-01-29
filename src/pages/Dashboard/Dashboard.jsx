import { useEffect, useState } from "react";
import Header from "../../components/Header/Header.jsx";
import Sidebar from "../../components/Sidebar/Sidebar.jsx";
import ServerTab from "../../components/Server/ServerTab.jsx";
import ConsoleTab from "../../components/Console/ConsoleTab.jsx";
import "./Dashboard.css";

const Dashboard = () => {
  const [activeTab, setActiveTab] = useState("server");
  const [consoleLogs, setConsoleLogs] = useState([]);

  const renderContent = () => {
    switch (activeTab) {
      case "server":
        return <ServerTab />;

      case "console":
        return (
        <ConsoleTab
          logs={consoleLogs}
          setLogs={setConsoleLogs}
        />
        );

      case "options":
        return <h2>Under Construction</h2>;

      case "journal":
        return <h2>Under Construction</h2>;

      case "players":
        return <h2>Under Construction</h2>;

      case "software":
        return <h2>Under Construction</h2>;

      case "files":
        return <h2>Under Construction</h2>;

      case "world":
        return <h2>Under Construction</h2>;

      default:
        return null;
    }
  };
  useEffect(() => {
  localStorage.setItem("logs", JSON.stringify(consoleLogs));
  }, [consoleLogs]);


  return (
    <div className="dashboard">
      <Header />

      <div className="dashboard-body">
        <Sidebar activeTab={activeTab} onChange={setActiveTab} />

        <main className="dashboard-content">
          <h1 className="page-title">
            {activeTab.toUpperCase()}
          </h1>

          {renderContent()}
        </main>
      </div>
    </div>
  );
};

export default Dashboard;
