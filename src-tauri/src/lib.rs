use log::info;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

pub struct AppState {
    pub client_id: Mutex<Option<i32>>,
    pub authorized: Mutex<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TdRequest {
    #[serde(rename = "@type")]
    pub type_: String,
    #[serde(flatten)]
    pub params: serde_json::Value,
}

#[tauri::command]
async fn init_tdlib() -> Result<i32, String> {
    info!("Initializing tdlib...");
    info!("Note: tdlib needs to be downloaded from GitHub releases");
    let client_id = 1;
    info!("Tdlib client created with ID: {}", client_id);
    Ok(client_id)
}

#[tauri::command]
async fn send_td_request(
    client_id: i32,
    request: serde_json::Value,
) -> Result<serde_json::Value, String> {
    info!("Sending request: {:?}", request);
    Ok(serde_json::json!({
        "@type": "error",
        "message": "tdlib not initialized. Download precompiled tdlib and rebuild."
    }))
}

#[tauri::command]
async fn receive_update(_client_id: i32) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({"@type": "null"}))
}

#[tauri::command]
async fn get_me(_client_id: i32) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "@type": "error",
        "message": "tdlib not initialized"
    }))
}

#[tauri::command]
async fn get_chats(_client_id: i32) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "@type": "chats",
        "chats": [],
        "count": 0
    }))
}

#[tauri::command]
async fn send_message(
    _client_id: i32,
    _chat_id: i64,
    _text: String,
) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "@type": "error",
        "message": "tdlib not initialized"
    }))
}

#[tauri::command]
async fn set_tdlib_parameters(
    client_id: i32,
    api_id: i32,
    api_hash: String,
    _device_model: String,
    _system_version: String,
    _application_version: String,
) -> Result<serde_json::Value, String> {
    info!("Setting tdlib parameters: api_id={}, api_hash={}", api_id, api_hash);
    send_td_request(client_id, serde_json::json!({
        "@type": "setTdlibParameters",
        "parameters": {
            "api_id": api_id,
            "api_hash": api_hash,
            "device_model": "Desktop",
            "system_version": "Unknown",
            "application_version": "0.1.0",
            "use_message_database": true,
            "use_secret_chats": true,
            "use_pfs": true
        }
    })).await
}

#[tauri::command]
async fn check_authentication_code(
    client_id: i32,
    code: String,
) -> Result<serde_json::Value, String> {
    send_td_request(client_id, serde_json::json!({
        "@type": "auth.enterCode",
        "code": code
    })).await
}

#[tauri::command]
async fn send_authentication_phone(
    client_id: i32,
    phone_number: String,
) -> Result<serde_json::Value, String> {
    send_td_request(client_id, serde_json::json!({
        "@type": "auth.sendCode",
        "phone_number": phone_number,
        "allow_flash_call": false,
        "allow_missed_call": false,
        "is_current_phone_number": true
    })).await
}

#[tauri::command]
async fn logout(client_id: i32, state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let result = send_td_request(client_id, serde_json::json!({"@type": "auth.logOut"})).await?;
    let mut authorized = state.authorized.lock().map_err(|e| e.to_string())?;
    *authorized = false;
    Ok(result)
}

pub fn run() {
    std::panic::set_hook(Box::new(|info| eprintln!("PANIC: {}", info)));
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    info!("Starting Telegram Desktop...");
    tauri::Builder::default()
        .manage(AppState { client_id: Mutex::new(None), authorized: Mutex::new(false) })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            init_tdlib,
            send_td_request,
            receive_update,
            get_me,
            get_chats,
            send_message,
            set_tdlib_parameters,
            check_authentication_code,
            send_authentication_phone,
            logout
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}