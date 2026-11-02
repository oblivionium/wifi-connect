use std::{env, fs, path::PathBuf, process::Command};

#[cfg(target_os = "windows")]
const TASK: &str = "College Wi-Fi Auto Relogin";
const INTERVAL_SECONDS: u64 = 19_800;

pub fn enabled() -> bool {
    #[cfg(target_os = "linux")]
    return Command::new("systemctl")
        .args(["--user", "is-enabled", "college-wifi-relogin.timer"])
        .output()
        .is_ok_and(|output| output.status.success());

    #[cfg(target_os = "macos")]
    return home_dir()
        .join("Library/LaunchAgents/edu.lnmiit.college-wifi.relogin.plist")
        .exists();

    #[cfg(target_os = "windows")]
    return Command::new("schtasks")
        .args(["/Query", "/TN", TASK])
        .output()
        .is_ok_and(|o| o.status.success());
}

pub fn set_enabled(enabled: bool) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    return linux(enabled);

    #[cfg(target_os = "macos")]
    return macos(enabled);

    #[cfg(target_os = "windows")]
    return windows(enabled);
}

pub fn create_connect_shortcut() -> Result<(), String> {
    #[cfg(target_os = "linux")]
    return linux_shortcut();

    #[cfg(target_os = "macos")]
    return macos_shortcut();

    #[cfg(target_os = "windows")]
    return windows_shortcut();
}

#[cfg(target_os = "linux")]
fn linux_shortcut() -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    let executable = env::current_exe().map_err(|e| e.to_string())?;
    let desktop = desktop_dir();
    let _ = fs::remove_file(desktop.join("Connect to College Wi-Fi.desktop"));
    let path = desktop.join("Connect.desktop");
    fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
    let exec = executable
        .to_string_lossy()
        .replace('\\', "\\\\")
        .replace('"', "\\\"");
    fs::write(&path, format!("[Desktop Entry]\nType=Application\nName=Connect\nComment=Connect using saved credentials\nExec=\"{exec}\" --connect\nIcon=wifi-desktop\nTerminal=false\nCategories=Network;Utility;\n")).map_err(|e| e.to_string())?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).map_err(|e| e.to_string())
}

#[cfg(target_os = "macos")]
fn macos_shortcut() -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    let executable = env::current_exe().map_err(|e| e.to_string())?;
    let desktop = desktop_dir();
    let _ = fs::remove_dir_all(desktop.join("Connect to College Wi-Fi.app"));
    let app = desktop.join("Connect.app");
    let contents = app.join("Contents");
    let binary = contents.join("MacOS/ConnectWiFi");
    fs::create_dir_all(binary.parent().unwrap()).map_err(|e| e.to_string())?;
    let resources = contents.join("Resources");
    fs::create_dir_all(&resources).map_err(|e| e.to_string())?;
    fs::write(
        resources.join("icon.icns"),
        include_bytes!("../icons/icon.icns"),
    )
    .map_err(|e| e.to_string())?;
    fs::write(contents.join("Info.plist"), "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n<plist version=\"1.0\"><dict><key>CFBundleExecutable</key><string>ConnectWiFi</string><key>CFBundleIdentifier</key><string>edu.lnmiit.college-wifi.connect</string><key>CFBundleName</key><string>Connect</string><key>CFBundleIconFile</key><string>icon.icns</string><key>CFBundlePackageType</key><string>APPL</string></dict></plist>\n").map_err(|e| e.to_string())?;
    let exe = executable.to_string_lossy().replace('\'', "'\\''");
    fs::write(&binary, format!("#!/bin/sh\nexec '{exe}' --connect\n"))
        .map_err(|e| e.to_string())?;
    fs::set_permissions(binary, fs::Permissions::from_mode(0o755)).map_err(|e| e.to_string())
}

#[cfg(target_os = "windows")]
fn windows_shortcut() -> Result<(), String> {
    let executable = env::current_exe().map_err(|e| e.to_string())?;
    let executable = executable.to_string_lossy().replace('\'', "''");
    let script = format!("$w=New-Object -ComObject WScript.Shell; $d=[Environment]::GetFolderPath('Desktop'); Remove-Item -LiteralPath ($d+'\\Connect to College Wi-Fi.lnk') -Force -ErrorAction SilentlyContinue; $s=$w.CreateShortcut($d+'\\Connect.lnk'); $s.TargetPath='{executable}'; $s.Arguments='--connect'; $s.WorkingDirectory=[IO.Path]::GetDirectoryName('{executable}'); $s.Save()");
    let output = Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .output()
        .map_err(|e| e.to_string())?;
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_owned())
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn desktop_dir() -> PathBuf {
    env::var_os("XDG_DESKTOP_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| home_dir().join("Desktop"))
}

#[cfg(target_os = "linux")]
fn linux(enabled: bool) -> Result<(), String> {
    let dir = config_dir().join("systemd/user");
    let service = dir.join("college-wifi-relogin.service");
    let timer = dir.join("college-wifi-relogin.timer");
    if enabled {
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let executable = env::current_exe().map_err(|e| e.to_string())?;
        fs::write(&service, format!(
            "[Unit]\nDescription=Reconnect to College Wi-Fi\n\n[Service]\nType=oneshot\nExecStart=\"{}\" --connect\n",
            executable.display()
        )).map_err(|e| e.to_string())?;
        fs::write(&timer, format!(
            "[Unit]\nDescription=Reconnect to College Wi-Fi every 5.5 hours\n\n[Timer]\nOnActiveSec={INTERVAL_SECONDS}\nOnUnitActiveSec={INTERVAL_SECONDS}\nUnit=college-wifi-relogin.service\n\n[Install]\nWantedBy=timers.target\n"
        )).map_err(|e| e.to_string())?;
        systemctl(&["daemon-reload"])?;
        systemctl(&["enable", "--now", "college-wifi-relogin.timer"])?;
    } else {
        let _ = systemctl(&["disable", "--now", "college-wifi-relogin.timer"]);
        let _ = fs::remove_file(service);
        let _ = fs::remove_file(timer);
        systemctl(&["daemon-reload"])?;
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn systemctl(args: &[&str]) -> Result<(), String> {
    let result = Command::new("systemctl")
        .arg("--user")
        .args(args)
        .output()
        .map_err(|e| format!("Could not run systemctl --user: {e}"))?;
    if result.status.success() {
        Ok(())
    } else {
        Err(format!(
            "systemctl --user failed: {}",
            String::from_utf8_lossy(&result.stderr).trim()
        ))
    }
}

#[cfg(target_os = "macos")]
fn macos(enabled: bool) -> Result<(), String> {
    let path = home_dir().join("Library/LaunchAgents/edu.lnmiit.college-wifi.relogin.plist");
    if enabled {
        fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
        let executable = env::current_exe().map_err(|e| e.to_string())?;
        let executable = xml_escape(&executable.to_string_lossy());
        fs::write(&path, format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n<plist version=\"1.0\"><dict><key>Label</key><string>edu.lnmiit.college-wifi.relogin</string><key>ProgramArguments</key><array><string>{executable}</string><string>--connect</string></array><key>StartInterval</key><integer>{INTERVAL_SECONDS}</integer></dict></plist>\n")).map_err(|e| e.to_string())?;
        launchctl("bootstrap", &path)?;
    } else {
        let _ = launchctl("bootout", &path);
        let _ = fs::remove_file(path);
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn launchctl(action: &str, path: &std::path::Path) -> Result<(), String> {
    let uid = Command::new("id")
        .arg("-u")
        .output()
        .map_err(|e| e.to_string())?;
    let uid = String::from_utf8_lossy(&uid.stdout).trim().to_owned();
    let result = Command::new("launchctl")
        .args([action, &format!("gui/{uid}")])
        .arg(path)
        .output()
        .map_err(|e| e.to_string())?;
    if result.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&result.stderr).trim().to_owned())
    }
}

#[cfg(target_os = "macos")]
fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(target_os = "windows")]
fn windows(enabled: bool) -> Result<(), String> {
    let output = if enabled {
        let executable = env::current_exe().map_err(|e| e.to_string())?;
        let task = format!("\"{}\" --connect", executable.display());
        Command::new("schtasks")
            .args([
                "/Create", "/SC", "MINUTE", "/MO", "330", "/TN", TASK, "/TR", &task, "/F",
            ])
            .output()
    } else {
        Command::new("schtasks")
            .args(["/Delete", "/TN", TASK, "/F"])
            .output()
    }
    .map_err(|e| e.to_string())?;
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_owned())
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn home_dir() -> PathBuf {
    env::var_os("HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("USERPROFILE").map(PathBuf::from))
        .unwrap_or_default()
}

#[cfg(target_os = "linux")]
fn config_dir() -> PathBuf {
    env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home_dir().join(".config"))
}
