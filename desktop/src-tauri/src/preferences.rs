use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use tauri::{AppHandle, Manager};

#[derive(Default, Serialize, Deserialize)]
struct Preferences {
    #[serde(default = "default_true")]
    notifications_enabled: bool,
}

fn default_true() -> bool {
    true
}

fn preferences_path(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_config_dir()
        .map(|dir| dir.join("preferences.json"))
        .map_err(|error| error.to_string())
}

pub fn notifications_enabled(app: &AppHandle) -> bool {
    let Ok(path) = preferences_path(app) else {
        return true;
    };
    fs::read(path)
        .ok()
        .and_then(|contents| serde_json::from_slice::<Preferences>(&contents).ok())
        .map(|preferences| preferences.notifications_enabled)
        .unwrap_or(true)
}

pub fn set_notifications_enabled(app: &AppHandle, enabled: bool) -> Result<(), String> {
    let path = preferences_path(app)?;
    fs::create_dir_all(path.parent().expect("preferences path has parent"))
        .map_err(|error| error.to_string())?;
    let contents = serde_json::to_vec_pretty(&Preferences {
        notifications_enabled: enabled,
    })
    .map_err(|error| error.to_string())?;
    fs::write(path, contents).map_err(|error| error.to_string())
}
