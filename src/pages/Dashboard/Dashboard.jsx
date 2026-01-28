import { useState } from "react";
import Header from "../../components/Header/Header.jsx";
import Sidebar from "../../components/Sidebar/Sidebar.jsx";
import ServerTab from "../../components/ServerTab/ServerTab.jsx";
import "./Dashboard.css";

const Dashboard = () => {
  const [activeTab, setActiveTab] = useState("server");

  const renderContent = () => {
    switch (activeTab) {
      case "server":
        return <h2>Main</h2>;
      case "options":
        return <h2>Options</h2>;
      case "console":
        return <h2>Console</h2>;
      case "journal":
        return <h2>Journal</h2>;
      case "players":
        return <h2>Players</h2>;
      case "software":
        return <h2>Software</h2>;
      case "files":
        return <h2>Files</h2>;
      case "world":
        return <h2>World</h2>;
      default:
        return null;
    }
  };

  return (
    <div className="dashboard">
      <Header />

      <div className="dashboard-body">
        <Sidebar activeTab={activeTab} onChange={setActiveTab} />

        <main className="dashboard-content">
            <h1 className="page-title">Server</h1>
          {renderContent()}
            <ServerTab />
        </main>
      </div>
    </div>
  );
};

export default Dashboard;
