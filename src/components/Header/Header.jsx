import { IoMdSettings, IoMdInformationCircleOutline } from "react-icons/io";
import { MdOutlineAccountCircle, MdOutlineExitToApp } from "react-icons/md";
import "./Header.css";

const Header = ({ onOpenSettings, onOpenAbout }) => {

  const handleSettings = () => {
    onOpenSettings();
  };

  const handleProfile = () => {
    console.log("Open profile");
  };

  const handleExit = () => {
    console.log("Exit app");
  };
  
  const handleInfo = () => {
    onOpenAbout();
  };


  return (
    <header className="header">
      <h1 className="logo">LSCP</h1>

      <nav className="header-nav">

        <button onClick={handleSettings} title="Settings">
          <IoMdSettings />
        </button>

        <button onClick={handleInfo} title="Info">
          <IoMdInformationCircleOutline />
        </button>

        <button onClick={handleProfile} title="Profile">
          <MdOutlineAccountCircle />
        </button>

        <button onClick={handleExit} title="Exit">
          <MdOutlineExitToApp />
        </button>

      </nav>
    </header>
  );
};

export default Header;
