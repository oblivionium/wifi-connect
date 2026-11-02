#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod preferences;
mod schedule;

use tauri::{Manager, Wry};
use tauri_plugin_notification::NotificationExt;

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
async fn connect() -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(|| {
        let credentials = wifi::load_credentials().map_err(|error| error.to_string())?;
        let portal = wifi::Portal::new().map_err(|error| error.to_string())?;
        portal
            .login(&credentials.username, &credentials.password)
            .map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| format!("Connection worker stopped unexpectedly: {error}"))?
}

#[tauri::command]
async fn disconnect() -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(disconnect_once)
    .await
    .map_err(|error| format!("Disconnect worker stopped unexpectedly: {error}"))?
}

fn disconnect_once() -> Result<(), String> {
    let portal = wifi::Portal::new().map_err(|error| error.to_string())?;
    portal.logout().map_err(|error| error.to_string())
}

#[tauri::command]
async fn portal_authenticated() -> Result<bool, String> {
    tauri::async_runtime::spawn_blocking(|| {
        wifi::Portal::new()
            .map_err(|error| error.to_string())?
            .authenticated()
            .map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| format!("Portal status worker stopped unexpectedly: {error}"))?
}

#[tauri::command]
fn auto_relogin_enabled() -> bool {
    schedule::enabled()
}

#[tauri::command]
fn notifications_enabled(app: tauri::AppHandle<Wry>) -> bool {
    preferences::notifications_enabled(&app)
}

#[tauri::command]
fn set_notifications_enabled(app: tauri::AppHandle<Wry>, enabled: bool) -> Result<(), String> {
    preferences::set_notifications_enabled(&app, enabled)
}

#[tauri::command]
fn set_auto_relogin(enabled: bool) -> Result<(), String> {
    if enabled && wifi::load_credentials().is_err() {
        return Err("Save your credentials before enabling automatic relogin.".into());
    }
    schedule::set_enabled(enabled)
}

#[tauri::command]
fn create_connect_shortcut() -> Result<(), String> {
    if wifi::load_credentials().is_err() {
        return Err("Save your credentials before creating a connect shortcut.".into());
    }
    schedule::create_connect_shortcut()
}

fn connect_once() -> Result<(), String> {
    let credentials = wifi::load_credentials().map_err(|error| error.to_string())?;
    let portal = wifi::Portal::new().map_err(|error| error.to_string())?;
    portal
        .login(&credentials.username, &credentials.password)
        .map_err(|error| error.to_string())
}

fn notify(app: &tauri::AppHandle<Wry>, title: &str, body: &str) {
    let _ = app.notification().builder().title(title).body(body).show();
}

fn main() {
    let action = std::env::args().skip(1).find_map(|arg| match arg.as_str() {
        "--connect" => Some(true),
        "--disconnect" => Some(false),
        _ => None,
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .setup(move |app| {
            if let Some(connecting) = action {
                let result = if connecting {
                    connect_once().map(|_| "Connected to college Wi-Fi.")
                } else {
                    disconnect_once().map(|_| "Signed out of college Wi-Fi.")
                };
                let succeeded = result.is_ok();
                if preferences::notifications_enabled(app.handle()) {
                    match result {
                        Ok(message) => notify(app.handle(), "Wi-Fi Connect", message),
                        Err(error) => notify(
                            app.handle(),
                            "Wi-Fi Connect",
                            &format!("Connection failed: {error}"),
                        ),
                    };
                }
                app.handle().exit(if succeeded { 0 } else { 1 });
            } else if let Some(window) = app.get_webview_window("main") {
                window.show()?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            credentials_saved,
            current_username,
            save_credentials,
            connect,
            disconnect,
            portal_authenticated,
            auto_relogin_enabled,
            set_auto_relogin,
            create_connect_shortcut,
            notifications_enabled,
            set_notifications_enabled
        ])
        .run(tauri::generate_context!())
        .expect("failed to start Wi-Fi Connect");
}
