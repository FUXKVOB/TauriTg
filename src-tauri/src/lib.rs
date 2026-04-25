use log::info;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

#[cfg(target_arch = "x86_64")]
#[cfg(target_os = "windows")]
mod tdlib {
    use std::ffi::{CStr, CString};
    use std::os::raw::c_char;
    
    #[link(name = "tdjson")]
    extern "C" {
        pub fn td_create_client_id() -> i32;
        pub fn td_send(client_id: i32, request: *const c_char);
        pub fn td_receive(timeout: f64) -> *const c_char;
    }
    
    pub fn create_client() -> i32 {
        unsafe { td_create_client_id() }
    }
    
    pub fn send(client_id: i32, request: &str) {
        unsafe {
            let cstr = CString::new(request).unwrap();
            td_send(client_id, cstr.as_ptr());
        }
    }
    
    pub fn receive(timeout: f64) -> Option<String> {
        unsafe {
            let ptr = td_receive(timeout);
            if ptr.is_null() {
                None
            } else {
                Some(CStr::from_ptr(ptr).to_string_lossy().into_owned())
            }
        }
    }
}

pub struct AppState {
    pub client_id: Mutex<Option<i32>>,
    pub authorized: Mutex<bool>,
}

fn parse_response(response: Option<String>) -> Result<serde_json::Value, String> {
    match response {
        Some(s) => serde_json::from_str(&s).map_err(|e| format!("Parse error: {}", e)),
        None => Ok(serde_json::json!({"@type": "null"})),
    }
}

#[tauri::command]
async fn init_tdlib() -> Result<i32, String> {
    info!("Initializing tdlib...");
    
    #[cfg(target_arch = "x86_64")]
    #[cfg(target_os = "windows")]
    {
        let client_id = tdlib::create_client();
        info!("Tdlib client created with ID: {}", client_id);
        return Ok(client_id);
    }
    
    #[cfg(not(all(target_arch = "x86_64", target_os = "windows")))]
    {
        info!("tdlib not available on this platform, using placeholder");
        Ok(1)
    }
}

#[tauri::command]
async fn send_td_request(
    client_id: i32,
    request: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let request_str = serde_json::to_string(&request)
        .map_err(|e| format!("Serialize error: {}", e))?;
    
    #[cfg(target_arch = "x86_64")]
    #[cfg(target_os = "windows")]
    {
        tdlib::send(client_id, &request_str);
        let response = tdlib::receive(10.0);
        return parse_response(response);
    }
    
    #[cfg(not(all(target_arch = "x86_64", target_os = "windows")))]
    {
        Ok(serde_json::json!({"@type": "not_supported"}))
    }
}

#[tauri::command]
async fn receive_update(_client_id: i32) -> Result<serde_json::Value, String> {
    #[cfg(target_arch = "x86_64")]
    #[cfg(target_os = "windows")]
    {
        let response = tdlib::receive(1.0);
        return parse_response(response);
    }
    
    #[cfg(not(all(target_arch = "x86_64", target_os = "windows")))]
    {
        Ok(serde_json::json!({"@type": "null"}))
    }
}

#[tauri::command]
async fn get_me(client_id: i32) -> Result<serde_json::Value, String> {
    send_td_request(client_id, serde_json::json!({"@type": "getMe"})).await
}

#[tauri::command]
async fn get_chats(client_id: i32) -> Result<serde_json::Value, String> {
    send_td_request(client_id, serde_json::json!({
        "@type": "getChats",
        "offset_order": "9223372036854775807",
        "offset_chat_id": 0,
        "limit": 100
    })).await
}

#[tauri::command]
async fn send_message(
    client_id: i32,
    chat_id: i64,
    text: String,
) -> Result<serde_json::Value, String> {
    let request = serde_json::json!({
        "@type": "messages.sendMessage",
        "chat_id": chat_id,
        "input_message_content": {
            "@type": "inputMessageText",
            "text": {
                "@type": "formattedText",
                "text": text
            }
        }
    });
    send_td_request(client_id, request).await
}

#[tauri::command]
async fn set_tdlib_parameters(
    client_id: i32,
    api_id: i32,
    api_hash: String,
    device_model: String,
    system_version: String,
    application_version: String,
) -> Result<serde_json::Value, String> {
    let params = serde_json::json!({
        "@type": "setTdlibParameters",
        "parameters": {
            "api_id": api_id,
            "api_hash": api_hash,
            "device_model": device_model,
            "system_version": system_version,
            "application_version": application_version,
            "use_message_database": true,
            "use_secret_chats": true,
            "use_pfs": true
        }
    });
    send_td_request(client_id, params).await
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