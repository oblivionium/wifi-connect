use reqwest::blocking::{Client, Response};
use reqwest::header::ACCEPT;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{self, Read};
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Wry};

const RELEASE_API: &str = "https://api.github.com/repos/oblivionium/wifi-connect/releases/latest";
const RELEASE_DOWNLOADS: &str = "https://github.com/oblivionium/wifi-connect/releases/download";
const MAX_INSTALLER_SIZE: u64 = 250 * 1024 * 1024;

#[derive(Debug, Deserialize)]
struct GithubRelease {
    tag_name: String,
    assets: Vec<GithubAsset>,
}

#[derive(Debug, Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
    size: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCheck {
    current_version: String,
    latest_version: String,
    update_available: bool,
    installer_available: bool,
}

#[tauri::command]
pub async fn check_for_updates() -> Result<UpdateCheck, String> {
    tauri::async_runtime::spawn_blocking(check_for_updates_blocking)
        .await
        .map_err(|error| format!("Update check worker stopped unexpectedly: {error}"))?
}

fn check_for_updates_blocking() -> Result<UpdateCheck, String> {
    let release = latest_release()?;
    let latest = parse_tag_version(&release.tag_name)?;
    let current = Version::parse(env!("CARGO_PKG_VERSION"))
        .map_err(|error| format!("Could not read the installed version: {error}"))?;
    let update_available = latest > current;
    let installer_available = update_available
        && find_installer(&release, &latest)
            .is_some_and(|asset| asset.size > 0 && asset.size <= MAX_INSTALLER_SIZE);

    Ok(UpdateCheck {
        current_version: format!("v{current}"),
        latest_version: format!("v{latest}"),
        update_available,
        installer_available,
    })
}

#[tauri::command]
pub async fn install_update(app: AppHandle<Wry>, expected_version: String) -> Result<(), String> {
    let app_handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || download_and_launch(&expected_version))
        .await
        .map_err(|error| format!("Update worker stopped unexpectedly: {error}"))??;
    app_handle.exit(0);
    Ok(())
}

fn latest_release() -> Result<GithubRelease, String> {
    let client = Client::builder()
        .user_agent(format!("wifi-connect/{}", env!("CARGO_PKG_VERSION")))
        .timeout(Duration::from_secs(20))
        .build()
        .map_err(|error| format!("Could not prepare the update check: {error}"))?;
    let response = client
        .get(RELEASE_API)
        .header(ACCEPT, "application/vnd.github+json")
        .send()
        .map_err(|error| format!("Could not reach GitHub Releases: {error}"))?
        .error_for_status()
        .map_err(|error| format!("GitHub Releases returned an error: {error}"))?;
    response
        .json::<GithubRelease>()
        .map_err(|error| format!("Could not read the latest release details: {error}"))
}

fn parse_tag_version(tag: &str) -> Result<Version, String> {
    let version = tag
        .strip_prefix('v')
        .ok_or_else(|| format!("The latest release has an unexpected tag: {tag}"))?;
    let parsed = Version::parse(version)
        .map_err(|error| format!("The latest release has an invalid version: {error}"))?;
    if tag != format!("v{parsed}") {
        return Err(format!("The latest release has an unexpected tag: {tag}"));
    }
    Ok(parsed)
}

fn expected_asset_name(version: &Version) -> Option<String> {
    let version = format!("v{version}");
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    {
        Some(format!("Wi-Fi.Connect_{version}_x64-setup.exe"))
    }
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        Some(format!("Wi-Fi.Connect_{version}_amd64.deb"))
    }
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        Some(format!("Wi-Fi.Connect_{version}_aarch64.pkg"))
    }
    #[cfg(not(any(
        all(target_os = "windows", target_arch = "x86_64"),
        all(target_os = "linux", target_arch = "x86_64"),
        all(target_os = "macos", target_arch = "aarch64")
    )))]
    {
        let _ = version;
        None
    }
}

fn find_installer<'a>(release: &'a GithubRelease, version: &Version) -> Option<&'a GithubAsset> {
    let name = expected_asset_name(version)?;
    release.assets.iter().find(|asset| asset.name == name)
}

fn download_and_launch(expected_version: &str) -> Result<(), String> {
    let expected = Version::parse(
        expected_version
            .strip_prefix('v')
            .ok_or_else(|| "The requested update version is invalid.".to_owned())?,
    )
    .map_err(|_| "The requested update version is invalid.".to_owned())?;
    let current = Version::parse(env!("CARGO_PKG_VERSION"))
        .map_err(|error| format!("Could not read the installed version: {error}"))?;
    if expected <= current {
        return Err("That release is not newer than the installed version.".into());
    }

    let release = latest_release()?;
    let latest = parse_tag_version(&release.tag_name)?;
    if latest != expected {
        return Err("The latest release changed. Check for updates again.".into());
    }
    let asset = find_installer(&release, &latest)
        .ok_or_else(|| "No installer for this device is available in that release.".to_owned())?;
    if asset.size == 0 || asset.size > MAX_INSTALLER_SIZE {
        return Err("The installer size is outside the allowed range.".into());
    }

    let expected_url = format!(
        "{RELEASE_DOWNLOADS}/{}/{name}",
        release.tag_name,
        name = asset.name
    );
    if asset.browser_download_url != expected_url {
        return Err("The release listed an unexpected installer download URL.".into());
    }

    let client = Client::builder()
        .user_agent(format!("wifi-connect/{}", env!("CARGO_PKG_VERSION")))
        .timeout(Duration::from_secs(180))
        .build()
        .map_err(|error| format!("Could not prepare the installer download: {error}"))?;
    let response = client
        .get(&asset.browser_download_url)
        .header(ACCEPT, "application/octet-stream")
        .send()
        .map_err(|error| format!("Could not download the update: {error}"))?
        .error_for_status()
        .map_err(|error| format!("The update download failed: {error}"))?;
    verify_download_host(&response)?;

    let path = download_path(&asset.name)?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|error| format!("Could not create a temporary installer file: {error}"))?;
    let copied = io::copy(&mut response.take(asset.size + 1), &mut file);
    let sync_result = file.sync_all();
    drop(file);
    let copied = match copied {
        Ok(size) => size,
        Err(error) => {
            let _ = fs::remove_file(&path);
            return Err(format!("Could not save the downloaded installer: {error}"));
        }
    };
    if let Err(error) = sync_result {
        let _ = fs::remove_file(&path);
        return Err(format!("Could not finish saving the installer: {error}"));
    }
    if copied != asset.size {
        let _ = fs::remove_file(&path);
        return Err("The downloaded installer size did not match the release details.".into());
    }

    if let Err(error) = launch_installer(&path) {
        let _ = fs::remove_file(&path);
        return Err(error);
    }
    Ok(())
}

fn verify_download_host(response: &Response) -> Result<(), String> {
    let url = response.url();
    let host = url.host_str().unwrap_or_default();
    let github_host = host == "github.com" || host.ends_with(".githubusercontent.com");
    if url.scheme() != "https" || !github_host {
        return Err("GitHub redirected the installer to an unexpected address.".into());
    }
    Ok(())
}

fn download_path(filename: &str) -> Result<PathBuf, String> {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("Could not create an installer filename: {error}"))?
        .as_nanos();
    Ok(std::env::temp_dir().join(format!(
        "wifi-connect-update-{}-{nonce}-{filename}",
        std::process::id()
    )))
}

fn launch_installer(path: &PathBuf) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    let mut command = Command::new(path);

    #[cfg(target_os = "macos")]
    let mut command = {
        let mut command = Command::new("/usr/bin/open");
        command.arg("-a").arg("Installer").arg(path);
        command
    };

    #[cfg(target_os = "linux")]
    let mut command = {
        let mut command = Command::new("pkexec");
        command.args(["apt-get", "install", "-y"]).arg(path);
        command
    };

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    let mut command = {
        let _ = path;
        return Err("Installing updates is not supported on this platform.".into());
    };

    command
        .spawn()
        .map_err(|error| format!("Could not launch the system installer: {error}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        expected_asset_name, find_installer, parse_tag_version, GithubAsset, GithubRelease,
        RELEASE_DOWNLOADS,
    };
    use semver::Version;

    #[test]
    fn release_tags_must_be_canonical_semver_tags() {
        assert_eq!(parse_tag_version("v1.2.3").unwrap(), Version::new(1, 2, 3));
        assert!(parse_tag_version("1.2.3").is_err());
        assert!(parse_tag_version("v01.2.3").is_err());
    }

    #[test]
    fn release_lookup_selects_the_installer_for_this_build_platform() {
        let version = Version::new(1, 2, 3);
        let expected_name = expected_asset_name(&version).unwrap();
        let release = GithubRelease {
            tag_name: "v1.2.3".into(),
            assets: vec![GithubAsset {
                name: expected_name.clone(),
                browser_download_url: format!("{RELEASE_DOWNLOADS}/v1.2.3/{expected_name}"),
                size: 1024,
            }],
        };

        assert_eq!(
            find_installer(&release, &version).unwrap().name,
            expected_name
        );
    }
}
