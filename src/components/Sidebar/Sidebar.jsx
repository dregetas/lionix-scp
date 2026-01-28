import "./Sidebar.css";

const items = [
  { id: "server", label: "Server" },
  { id: "options", label: "Options" },
  { id: "console", label: "Console" },
  { id: "journal", label: "Journal" },
  { id: "players", label: "Players" },
  { id: "software", label: "Software" },
  { id: "files", label: "Files" },
  { id: "world", label: "World" },
];

const Sidebar = ({ activeTab, onChange }) => {
  return (
    <aside className="sidebar">
      <ul className="sidebar-list">
        {items.map((item) => (
          <li key={item.id}>
            <button
              className={activeTab === item.id ? "active" : ""}
              onClick={() => onChange(item.id)}
            >
              {item.label}
            </button>
          </li>
        ))}
      </ul>
    </aside>
  );
};

export default Sidebar;
