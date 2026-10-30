#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[tauri::command]
fn credentials_saved() -> bool {
    wifi::load_credentials().is_ok()
}

#[tauri::command]
fn current_username() -> Option<String> {
    wifi::load_credentials()
        .ok()
        .map(|credentials| credentials.username)
}

#[tauri::command]
fn save_credentials(username: String, password: String) -> Result<(), String> {
    if username.trim().is_empty() {
        return Err("Enter your college username.".into());
    }
    if password.is_empty() {
        return Err("Enter your college password.".into());
    }
    let password = zeroize::Zeroizing::new(password);
    wifi::save_credentials(username.trim(), &password).map_err(|error| error.to_string())
}

#[tauri::command]
fn connect() -> Result<(), String> {
    let credentials = wifi::load_credentials().map_err(|error| error.to_string())?;
    let portal = wifi::Portal::new().map_err(|error| error.to_string())?;
    portal
        .login(&credentials.username, &credentials.password)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn disconnect() -> Result<(), String> {
    let portal = wifi::Portal::new().map_err(|error| error.to_string())?;
    portal.logout().map_err(|error| error.to_string())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .invoke_handler(tauri::generate_handler![
            credentials_saved,
            current_username,
            save_credentials,
            connect,
            disconnect
        ])
        .run(tauri::generate_context!())
        .expect("failed to start College Wi-Fi settings");
}
