import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

function App() {
  const [isAuthorized] = useState(false);

  const handleLogin = async () => {
    const phone = prompt("Enter phone number:");
    if (phone) {
      const req = { "@type": "sendCode", "phone_number": phone };
      const data = JSON.stringify(req);
      await invoke("send_telegram", { data });
    }
  };

  return (
    <div className="app">
      <header className="header">
        <h1>Telegram</h1>
      </header>
      <main className="content">
        {!isAuthorized ? (
          <button onClick={handleLogin}>Login</button>
        ) : (
          <div className="chat-list">
            <h2>Chats</h2>
          </div>
        )}
      </main>
    </div>
  );
}

export default App;