//! Tauri commands for Vericonomy push API registration.

use crate::error::AppResult;

#[tauri::command]
pub async fn push_sync_device(device_token: String) -> AppResult<()> {
    crate::push_registration::register_device(&device_token).await
}

#[tauri::command]
pub async fn push_heartbeat_device(device_token: String) -> AppResult<()> {
    crate::push_registration::heartbeat_device(&device_token).await
}

#[tauri::command]
pub async fn push_unregister_device(device_token: String) -> AppResult<()> {
    crate::push_registration::unregister_device(&device_token).await
}

#[tauri::command]
pub fn push_registration_configured() -> bool {
    crate::push_registration::push_api_secret_configured()
}

#[tauri::command]
pub fn push_watch_scripthash_counts() -> Result<(usize, usize), String> {
    crate::push_registration::push_watch_scripthash_counts().map_err(|e| e.to_string())
}
