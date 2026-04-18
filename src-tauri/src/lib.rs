mod tdlib;

use serde::{Deserialize, Serialize};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use tauri::{AppHandle, Emitter, Manager};

static UPDATE_TX: once_cell::sync::OnceCell<Sender<String>> = once_cell::sync::OnceCell::new();

#[derive(Debug, Serialize, Deserialize)]
pub struct TdUpdate {
    #[serde(flatten)]
    data: serde_json::Value,
}

#[tauri::command]
fn send_telegram(data: String) -> Result<(), String> {
    tdlib::send_telegram(&data)
}

#[tauri::command]
fn execute_telegram(data: String) -> Result<Option<String>, String> {
    tdlib::execute_telegram(&data)
}

#[tauri::command]
async fn listen_updates(app: AppHandle) -> Result<(), String> {
    let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();

    UPDATE_TX.get_or_init(|| tx);

    thread::spawn(move || {
        loop {
            if let Some(update) = tdlib::receive_telegram(0.1) {
                let _ = app.emit("telegram-update", &update);
            }
        }
    });

    Ok(())
}

#[tauri::command]
fn get_storage_dir(app: AppHandle) -> Result<String, String> {
    let path = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    log::info!("Starting Telegram Tauri...");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            send_telegram,
            execute_telegram,
            listen_updates,
            get_storage_dir,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}