use std::{env, fs, path::Path, path::PathBuf, process::Command};

#[cfg(any(target_os = "windows", test))]
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
        .args(["/Query", "/TN", TASK, "/XML"])
        .output()
        .is_ok_and(|output| {
            output.status.success()
                && windows_task_enabled(&String::from_utf8_lossy(&output.stdout))
        });
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
        fs::write(&service, linux_service_unit(&executable)).map_err(|e| e.to_string())?;
        fs::write(&timer, linux_timer_unit()).map_err(|e| e.to_string())?;
        systemctl(&["daemon-reload"])?;
        systemctl(&["enable", "--now", "college-wifi-relogin.timer"])?;
    } else {
        systemctl(&["disable", "--now", "college-wifi-relogin.timer"])?;
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
        fs::write(&path, macos_agent_plist(&executable)).map_err(|e| e.to_string())?;
        // Refresh an already-loaded agent too, so upgrades pick up the current executable.
        let _ = launchctl("bootout", &path);
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

#[cfg(any(target_os = "macos", test))]
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
        Command::new("schtasks")
            .args(windows_create_args(&executable))
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

#[cfg(any(target_os = "linux", test))]
fn linux_service_unit(executable: &Path) -> String {
    format!(
        "[Unit]\nDescription=Reconnect to College Wi-Fi\n\n[Service]\nType=oneshot\nExecStart=\"{}\" --connect\n",
        executable.display()
    )
}

#[cfg(any(target_os = "linux", test))]
fn linux_timer_unit() -> String {
    format!(
        "[Unit]\nDescription=Reconnect to College Wi-Fi every 5.5 hours\n\n[Timer]\nOnActiveSec={INTERVAL_SECONDS}\nOnUnitActiveSec={INTERVAL_SECONDS}\nUnit=college-wifi-relogin.service\n\n[Install]\nWantedBy=timers.target\n"
    )
}

#[cfg(any(target_os = "macos", test))]
fn macos_agent_plist(executable: &Path) -> String {
    let executable = xml_escape(&executable.to_string_lossy());
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n<plist version=\"1.0\"><dict><key>Label</key><string>edu.lnmiit.college-wifi.relogin</string><key>ProgramArguments</key><array><string>{executable}</string><string>--connect</string></array><key>StartInterval</key><integer>{INTERVAL_SECONDS}</integer></dict></plist>\n"
    )
}

#[cfg(any(target_os = "windows", test))]
fn windows_task_run_command(executable: &Path) -> String {
    format!("\"{}\" --connect", executable.display())
}

#[cfg(any(target_os = "windows", test))]
fn windows_task_enabled(xml: &str) -> bool {
    xml.find("<Enabled>")
        .and_then(|start| {
            let value = &xml[start + "<Enabled>".len()..];
            value.find("</Enabled>").map(|end| value[..end].trim())
        })
        .is_none_or(|enabled| enabled.eq_ignore_ascii_case("true"))
}

#[cfg(any(target_os = "windows", test))]
fn windows_create_args(executable: &Path) -> Vec<String> {
    vec![
        "/Create".into(),
        "/SC".into(),
        "MINUTE".into(),
        "/MO".into(),
        "330".into(),
        "/TN".into(),
        TASK.into(),
        "/TR".into(),
        windows_task_run_command(executable),
        "/IT".into(),
        "/F".into(),
    ]
}

#[cfg(test)]
mod tests {
    use super::{
        linux_service_unit, linux_timer_unit, macos_agent_plist, windows_create_args,
        windows_task_enabled,
    };
    use std::path::Path;

    #[test]
    fn linux_timer_starts_and_repeats_with_connect_after_five_and_a_half_hours() {
        let service = linux_service_unit(Path::new("/opt/Wi-Fi Connect/wifi-desktop"));
        let timer = linux_timer_unit();

        assert!(service.contains("ExecStart=\"/opt/Wi-Fi Connect/wifi-desktop\" --connect"));
        assert!(timer.contains("OnActiveSec=19800"));
        assert!(timer.contains("OnUnitActiveSec=19800"));
        assert!(timer.contains("Unit=college-wifi-relogin.service"));
    }

    #[test]
    fn macos_agent_runs_the_installed_app_with_connect_argument() {
        let plist = macos_agent_plist(Path::new(
            "/Applications/Wi-Fi & Connect.app/Contents/MacOS/Wi-Fi Connect",
        ));

        assert!(plist.contains(
            "<string>/Applications/Wi-Fi &amp; Connect.app/Contents/MacOS/Wi-Fi Connect</string>"
        ));
        assert!(plist.contains("<string>--connect</string>"));
        assert!(plist.contains("<key>StartInterval</key><integer>19800</integer>"));
    }

    #[test]
    fn windows_task_is_interactive_and_repeats_every_330_minutes() {
        let args = windows_create_args(Path::new(
            r"C:\Program Files\Wi-Fi Connect\wifi-desktop.exe",
        ));

        assert!(args.windows(2).any(|pair| pair == ["/SC", "MINUTE"]));
        assert!(args.windows(2).any(|pair| pair == ["/MO", "330"]));
        assert!(args.iter().any(|argument| argument == "/IT"));
        assert!(args.iter().any(|argument| {
            argument == r#""C:\Program Files\Wi-Fi Connect\wifi-desktop.exe" --connect"#
        }));
    }

    #[test]
    fn windows_disabled_task_is_not_reported_as_enabled() {
        assert!(windows_task_enabled(
            "<Settings><Enabled>true</Enabled></Settings>"
        ));
        assert!(!windows_task_enabled(
            "<Settings><Enabled>false</Enabled></Settings>"
        ));
        assert!(windows_task_enabled("<Settings></Settings>"));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_schedule_install_and_remove_use_the_user_service_manager_commands() {
        use super::{enabled, linux};
        use std::ffi::OsString;
        use std::fs;
        use std::os::unix::fs::PermissionsExt;
        use std::time::{SystemTime, UNIX_EPOCH};

        struct RestoreEnv {
            xdg_config_home: Option<OsString>,
            path: Option<OsString>,
            log: Option<OsString>,
        }

        impl Drop for RestoreEnv {
            fn drop(&mut self) {
                for (name, value) in [
                    ("XDG_CONFIG_HOME", self.xdg_config_home.take()),
                    ("PATH", self.path.take()),
                    ("SYSTEMCTL_TEST_LOG", self.log.take()),
                ] {
                    if let Some(value) = value {
                        std::env::set_var(name, value);
                    } else {
                        std::env::remove_var(name);
                    }
                }
            }
        }

        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp = std::env::temp_dir().join(format!(
            "wifi-connect-schedule-test-{}-{nonce}",
            std::process::id()
        ));
        let bin = temp.join("bin");
        let config = temp.join("config");
        let log = temp.join("systemctl.log");
        fs::create_dir_all(&bin).unwrap();
        let systemctl = bin.join("systemctl");
        fs::write(
            &systemctl,
            "#!/bin/sh\nprintf '%s\\n' \"$*\" >> \"$SYSTEMCTL_TEST_LOG\"\nexit 0\n",
        )
        .unwrap();
        fs::set_permissions(&systemctl, fs::Permissions::from_mode(0o755)).unwrap();

        let restore = RestoreEnv {
            xdg_config_home: std::env::var_os("XDG_CONFIG_HOME"),
            path: std::env::var_os("PATH"),
            log: std::env::var_os("SYSTEMCTL_TEST_LOG"),
        };
        let path = std::env::var_os("PATH").unwrap_or_default();
        std::env::set_var("XDG_CONFIG_HOME", &config);
        std::env::set_var(
            "PATH",
            format!("{}:{}", bin.display(), path.to_string_lossy()),
        );
        std::env::set_var("SYSTEMCTL_TEST_LOG", &log);

        linux(true).unwrap();
        let units = config.join("systemd/user");
        let service = fs::read_to_string(units.join("college-wifi-relogin.service")).unwrap();
        let timer = fs::read_to_string(units.join("college-wifi-relogin.timer")).unwrap();
        assert!(service.contains("--connect"));
        assert!(timer.contains("OnUnitActiveSec=19800"));
        assert!(enabled());

        linux(false).unwrap();
        assert!(!units.join("college-wifi-relogin.service").exists());
        assert!(!units.join("college-wifi-relogin.timer").exists());
        let calls = fs::read_to_string(log).unwrap();
        assert!(calls.contains("--user daemon-reload"));
        assert!(calls.contains("--user enable --now college-wifi-relogin.timer"));
        assert!(calls.contains("--user is-enabled college-wifi-relogin.timer"));
        assert!(calls.contains("--user disable --now college-wifi-relogin.timer"));

        drop(restore);
        fs::remove_dir_all(temp).unwrap();
    }
}
