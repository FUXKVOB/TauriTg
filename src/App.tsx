import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

interface User {
  id: number;
  first_name: string;
  last_name?: string;
  username?: string;
}

interface Chat {
  id: number;
  title?: string;
  type: string;
  last_message?: { text: string };
  unread_count: number;
}

interface Message {
  id: number;
  sender_id: number;
  text: string;
  date: number;
  is_outgoing: boolean;
}

type AuthState = "init" | "phone" | "code" | "authorized" | "loading";
type TdResponse = Record<string, unknown>;

function App() {
  const [authState, setAuthState] = useState<AuthState>("init");
  const [phone, setPhone] = useState("");
  const [code, setCode] = useState("");
  const [error, setError] = useState("");
  const [apiId, setApiId] = useState("");
  const [apiHash, setApiHash] = useState("");
  const [clientId, setClientId] = useState<number | null>(null);
  const [me, setMe] = useState<User | null>(null);
  const [chats, setChats] = useState<Chat[]>([]);
  const [selectedChat, setSelectedChat] = useState<Chat | null>(null);
  const [messages, setMessages] = useState<Message[]>([]);
  const [messageText, setMessageText] = useState("");

  const loadChats = useCallback(async () => {
    try {
      const result = await invoke<TdResponse>("get_chats");
      const chatsData = result.chats as Chat[] | undefined;
      if (chatsData) {
        setChats(chatsData.slice(0, 50));
      }
    } catch (e) {
      console.error("Failed to load chats:", e);
    }
  }, []);

  const loadMessages = useCallback(async (chatId: number) => {
    try {
      const result = await invoke<TdResponse>("send_td_request", {
        clientId,
        request: {
          "@type": "messages.getHistory",
          chat_id: chatId,
          limit: 50
        }
      });
      const msgs = result.messages as Message[] | undefined;
      if (msgs) {
        setMessages(msgs.slice(0, 50).map((m: any) => ({
          id: m.id,
          sender_id: m.sender_id?.user_id || 0,
          text: m.content?.text?.text || "",
          date: m.date,
          is_outgoing: m.is_outgoing
        })));
      }
    } catch (e) {
      console.error("Failed to load messages:", e);
    }
  }, [clientId]);

  useEffect(() => {
    if (authState === "authorized") {
      loadChats();
    }
  }, [authState, loadChats]);

  useEffect(() => {
    if (selectedChat) {
      loadMessages(selectedChat.id);
    }
  }, [selectedChat, loadMessages]);

  const handleInit = async () => {
    if (!apiId || !apiHash) {
      setError("Please enter API ID and Hash");
      return;
    }
    setAuthState("loading");
    setError("");
    try {
      const id = await invoke<number>("init_tdlib");
      setClientId(id);
      await invoke("set_tdlib_parameters", {
        clientId: id,
        apiId: parseInt(apiId),
        apiHash,
        deviceModel: "Desktop",
        systemVersion: "Unknown",
        applicationVersion: "0.1.0"
      });
      setAuthState("phone");
    } catch (e: any) {
      setError(e.toString());
      setAuthState("init");
    }
  };

  const handlePhoneSubmit = async () => {
    if (!clientId || !phone) return;
    try {
      await invoke("send_td_request", {
        clientId,
        request: {
          "@type": "auth.sendCode",
          phone_number: phone,
          allow_flash_call: false,
          allow_missed_call: false,
          is_current_phone_number: true
        }
      });
      setAuthState("code");
    } catch (e: any) {
      setError(e.toString());
    }
  };

  const handleCodeSubmit = async () => {
    if (!clientId || !code) return;
    try {
      await invoke("send_td_request", {
        clientId,
        request: {
          "@type": "auth.enterCode",
          code: code
        }
      });
      
      const meResult = await invoke<TdResponse>("get_me", { clientId });
      if (meResult && (meResult as any)["@type"] === "user") {
        setMe(meResult as unknown as User);
        setAuthState("authorized");
      }
    } catch (e: any) {
      setError(e.toString());
    }
  };

  const handleSendMessage = async () => {
    if (!clientId || !selectedChat || !messageText) return;
    try {
      await invoke("send_message", {
        clientId,
        chatId: selectedChat.id,
        text: messageText
      });
      setMessageText("");
      loadMessages(selectedChat.id);
    } catch (e: any) {
      setError(e.toString());
    }
  };

  if (authState === "init") {
    return (
      <main className="container">
        <h1>Telegram Desktop</h1>
        <p>Get credentials from <a href="https://my.telegram.org" target="_blank">my.telegram.org</a></p>
        
        <div className="form">
          <label>
            API ID:
            <input type="text" value={apiId} onChange={(e) => setApiId(e.target.value)} placeholder="Enter API ID" />
          </label>
          <label>
            API Hash:
            <input type="text" value={apiHash} onChange={(e) => setApiHash(e.target.value)} placeholder="Enter API Hash" />
          </label>
          <button onClick={handleInit}>Start</button>
        </div>
        {error && <p className="error">{error}</p>}
      </main>
    );
  }

  if (authState === "phone") {
    return (
      <main className="container">
        <h1>Login</h1>
        <div className="form">
          <label>
            Phone:
            <input type="text" value={phone} onChange={(e) => setPhone(e.target.value)} placeholder="+1234567890" />
          </label>
          <button onClick={handlePhoneSubmit}>Send Code</button>
        </div>
      </main>
    );
  }

  if (authState === "code") {
    return (
      <main className="container">
        <h1>Login</h1>
        <div className="form">
          <label>
            Code:
            <input type="text" value={code} onChange={(e) => setCode(e.target.value)} placeholder="12345" />
          </label>
          <button onClick={handleCodeSubmit}>Confirm</button>
        </div>
      </main>
    );
  }

  if (authState === "loading") {
    return (
      <main className="container">
        <h1>Loading...</h1>
      </main>
    );
  }

  return (
    <main className="app">
      <div className="sidebar">
        <div className="sidebar-header">
          <h2>{me?.first_name || "Telegram"}</h2>
        </div>
        <div className="chat-list">
          {chats.map((chat) => (
            <div
              key={chat.id}
              className={`chat-item ${selectedChat?.id === chat.id ? "selected" : ""}`}
              onClick={() => setSelectedChat(chat)}
            >
              <div className="chat-title">{chat.title || "Chat"}</div>
              {chat.last_message && <div className="chat-last-message">{chat.last_message.text}</div>}
              {chat.unread_count > 0 && <span className="unread-badge">{chat.unread_count}</span>}
            </div>
          ))}
        </div>
      </div>
      
      <div className="chat-view">
        {selectedChat ? (
          <>
            <div className="chat-header">
              <h2>{selectedChat.title || "Chat"}</h2>
            </div>
            <div className="messages">
              {messages.map((msg) => (
                <div key={msg.id} className={`message ${msg.is_outgoing ? "outgoing" : "incoming"}`}>
                  {msg.text}
                </div>
              ))}
            </div>
            <div className="message-input">
              <input
                type="text"
                value={messageText}
                onChange={(e) => setMessageText(e.target.value)}
                placeholder="Type a message..."
                onKeyDown={(e) => e.key === "Enter" && handleSendMessage()}
              />
              <button onClick={handleSendMessage}>Send</button>
            </div>
          </>
        ) : (
          <div className="no-chat">
            <p>Select a chat</p>
          </div>
        )}
      </div>
    </main>
  );
}

export default App;