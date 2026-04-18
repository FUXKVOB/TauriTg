use libloading::{Library, Symbol};
use once_cell::sync::OnceCell;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

static TDJSON: OnceCell<Arc<TdJson>> = OnceCell::new();

pub struct TdJson {
    #[allow(dead_code)]
    lib: Library,
    create: Symbol<'static, extern "C" fn() -> *mut std::ffi::c_void>,
    send: Symbol<'static, extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_void)>,
    receive: Symbol<'static, extern "C" fn(*mut std::ffi::c_void, f64) -> *mut std::ffi::c_void>,
    execute: Symbol<'static, extern "C" fn(*mut std::ffi::c_void) -> *mut std::ffi::c_void>,
    destroy: Symbol<'static, extern "C" fn(*mut std::ffi::c_void)>,
    client: Mutex<*mut std::ffi::c_void>,
}

impl TdJson {
    pub fn new() -> Result<Self, String> {
        unsafe {
            let lib = Library::new("tdjson")
                .or_else(|_| Library::new("tdjson.dll"))
                .or_else(|_| Library::new("libtdjson"))
                .map_err(|e| format!("Failed to load tdjson: {}", e))?;

            let create: Symbol<Self, _> = lib
                .get(b"td_json_client_create")
                .map_err(|e| e.to_string())?;
            let send: Symbol<Self, _> =
                lib.get(b"td_json_client_send").map_err(|e| e.to_string())?;
            let receive: Symbol<Self, _> = lib
                .get(b"td_json_client_receive")
                .map_err(|e| e.to_string())?;
            let execute: Symbol<Self, _> = lib
                .get(b"td_json_client_execute")
                .map_err(|e| e.to_string())?;
            let destroy: Symbol<Self, _> = lib
                .get(b"td_json_client_destroy")
                .map_err(|e| e.to_string())?;

            let client = create();

            Ok(Self {
                lib,
                create,
                send,
                receive,
                execute,
                destroy,
                client: Mutex::new(client),
            })
        }
    }

    pub fn send(&self, data: &str) {
        let client = self.client.lock();
        unsafe {
            let c_str = std::ffi::CString::new(data).unwrap();
            self.send(*client, c_str.as_ptr());
        }
    }

    pub fn receive(&self, timeout: f64) -> Option<String> {
        let client = self.client.lock();
        unsafe {
            let result = self.receive(*client, timeout);
            if result.is_null() {
                None
            } else {
                let c_str = std::ffi::CStr::from_ptr(result);
                let s = c_str.to_string_lossy().into_owned();
                self.free_ptr(result);
                Some(s)
            }
        }
    }

    pub fn execute(&self, data: &str) -> Option<String> {
        unsafe {
            let client = *self.client.lock();
            let c_str = std::ffi::CString::new(data).unwrap();
            let result = self.execute(client, c_str.as_ptr());
            if result.is_null() {
                None
            } else {
                let c_str = std::ffi::CStr::from_ptr(result);
                let s = c_str.to_string_lossy().into_owned();
                self.free_ptr(result);
                Some(s)
            }
        }
    }

    #[allow(dead_code)]
    fn free_ptr(&self, ptr: *mut std::ffi::c_void) {
        unsafe {
            let free: Symbol<'_, extern "C" fn(*mut std::ffi::c_void)> =
                self.lib.get(b"td_json_client_destroy").unwrap();
            free(ptr);
        }
    }
}

impl Drop for TdJson {
    fn drop(&mut self) {
        let client = self.client.lock();
        unsafe {
            self.destroy(*client);
        }
    }
}

pub fn get_tdjson() -> Result<Arc<TdJson>, String> {
    TDJSON.get_or_try(|| {
        let tdjson = TdJson::new()?;
        Ok(Arc::new(tdjson))
    })
}

pub fn send_telegram(data: &str) -> Result<(), String> {
    let td = get_tdjson()?;
    td.send(data);
    Ok(())
}

pub fn execute_telegram(data: &str) -> Result<Option<String>, String> {
    let td = get_tdjson()?;
    Ok(td.execute(data))
}

pub fn receive_telegram(timeout: f64) -> Option<String> {
    let td = get_tdjson().ok()?;
    td.receive(timeout)
}
