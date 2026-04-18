import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface Update {
  "@type": string;
  [key: string]: unknown;
}

function App() {
  const [updates, setUpdates] = useState<string[]>([]);
  const [isAuthorized, setIsAuthorized] = useState(false);

  useEffect(() => {
    const unlisten = invoke<string>("listen_updates").then((unlistenFn) => {
      return () => {};
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const handleLogin = async () => {
    const phone = prompt("Enter phone number:");
    if (phone) {
      await invoke("send_code", { phone });
      const code = prompt("Enter code:");
      if (code) {
        await invoke("check_code", { code });
      }
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
            <div className="updates-log">
              {updates.map((u, i) => (
                <div key={i}>{u}</div>
              ))}
            </div>
          </div>
        )}
      </main>
    </div>
  );
}

export default App;