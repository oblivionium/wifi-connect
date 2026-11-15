<div align="center">

<a href="https://github.com/oblivionium/wifi-connect" target="_blank" rel="noopener noreferrer"><img src="assets/wifi.png" width="96" alt="Wi-Fi Connect on GitHub" /></a>

# Wi-Fi Connect

**A simple wifi login utility which reconnects you to the campus wifi.**

<p align="center">
    <a href="https://oblivionium.github.io/wifi-connect/" target="_blank" rel="noopener noreferrer" title="Open the Wi-Fi Connect download page"><img src="https://img.shields.io/github/downloads/oblivionium/wifi-connect/total?label=download&style=for-the-badge&labelColor=24292f&color=5B8750&logo=github&logoColor=white" height="42" alt="GitHub release download count; open the download page" /></a>
</p>

<a href="https://www.rust-lang.org/" target="_blank" rel="noopener noreferrer"><img alt="Rust" src="https://img.shields.io/badge/Rust-core-CE422B?style=flat-square&logo=rust&logoColor=white"></a>
<a href="https://tauri.app/" target="_blank" rel="noopener noreferrer"><img alt="Tauri" src="https://img.shields.io/badge/Tauri-desktop-24C8DB?style=flat-square&logo=tauri&logoColor=white"></a>
<a href="https://developer.mozilla.org/en-US/docs/Web/JavaScript" target="_blank" rel="noopener noreferrer"><img alt="JavaScript" src="https://img.shields.io/badge/JavaScript-interface-F7DF1E?style=flat-square&logo=javascript&logoColor=111111"></a>
<a href="https://github.com/oblivionium/wifi-connect/actions" target="_blank" rel="noopener noreferrer"><img alt="GitHub Actions" src="https://img.shields.io/badge/GitHub_Actions-builds-2088FF?style=flat-square&logo=githubactions&logoColor=white"></a>
<a href="https://github.com/oblivionium/wifi-connect/releases/latest" target="_blank" rel="noopener noreferrer"><img alt="Latest release" src="https://img.shields.io/github/v/release/oblivionium/wifi-connect"></a>

</div>

## Contents

<div align="center">

[Features](#features) · [Technology](#technology) · [Installation](#installation) · [Usage](#usage) · [Build from source](#build-from-source) · [Security](#security) · [Project layout](#project-layout)

</div>

## Features

- **One-click connection** from the desktop app or a silent desktop shortcut.
- **Credential management** in a separate page, with passwords stored in the
  operating system’s credential store.
- **Native notifications** for connection results and scheduled relogins; the
  **Connect** shortcut includes the Wi-Fi name and local sign-in time.
  Notifications can be disabled in Settings.
- **Optional auto-relogin** every 5.5 hours.
- **Update checks** in Settings, with the matching installer offered when a
  newer release is available.
- **Portal logout** through the logoff endpoint.
- **CLI commands** for `wifi login`, `wifi logout`, and `wifi credentials set`.

## Technology

<div align="center">

<a href="https://www.rust-lang.org/" target="_blank" rel="noopener noreferrer"><img alt="Rust" height="38" src="https://img.shields.io/badge/Rust-CE422B?style=for-the-badge&logo=rust&logoColor=white"></a>
<a href="https://tauri.app/" target="_blank" rel="noopener noreferrer"><img alt="Tauri 2" height="38" src="https://img.shields.io/badge/Tauri_2-24C8DB?style=for-the-badge&logo=tauri&logoColor=white"></a>
<a href="https://developer.mozilla.org/en-US/docs/Web/HTML" target="_blank" rel="noopener noreferrer"><img alt="HTML" height="38" src="https://img.shields.io/badge/HTML-E34F26?style=for-the-badge&logo=html5&logoColor=white"></a>
<a href="https://developer.mozilla.org/en-US/docs/Web/CSS" target="_blank" rel="noopener noreferrer"><img alt="CSS" height="38" src="https://img.shields.io/badge/CSS-663399?style=for-the-badge&logo=css&logoColor=white"></a>
<a href="https://developer.mozilla.org/en-US/docs/Web/JavaScript" target="_blank" rel="noopener noreferrer"><img alt="JavaScript" height="38" src="https://img.shields.io/badge/JavaScript-F7DF1E?style=for-the-badge&logo=javascript&logoColor=111111"></a>
<a href="https://github.com/seanmonstar/reqwest" target="_blank" rel="noopener noreferrer"><img alt="reqwest" height="38" src="https://img.shields.io/badge/reqwest-HTTP_client-59636E?style=for-the-badge"></a>
<a href="https://github.com/RustCrypto/RSA" target="_blank" rel="noopener noreferrer"><img alt="RustCrypto RSA" height="38" src="https://img.shields.io/badge/RustCrypto-RSA-59636E?style=for-the-badge"></a>

</div>

The UI is plain HTML, CSS, and JavaScript. Tauri provides the native window,
system notifications, and platform packaging. The Rust core uses `reqwest` for
the portal session and `rsa` for password encryption. While the targeted portal
is available, the main screen checks authentication on launch and keeps the
connection button on for 5.5 hours or until you disconnect.

The application implements the campus captive portal’s HTTP and RSA login flow
on Windows, macOS, and Linux. The desktop application uses the operating
system’s webview and does not bundle a browser engine.

## Installation

The download page detects your operating system and offers the latest matching
installer. Every installer includes the `wifi` CLI. Choose your platform below
for installation steps.

<p align="center">
  <a href="https://oblivionium.github.io/wifi-connect/" target="_blank" rel="noopener noreferrer" title="Open the Wi-Fi Connect download page">
    <img src="https://img.shields.io/github/downloads/oblivionium/wifi-connect/total?label=download&style=for-the-badge&labelColor=24292f&color=5B8750&logo=github&logoColor=white" height="42" alt="GitHub release download count; open the download page" />
  </a>
</p>

### Linux

From the folder containing the downloaded `.deb`, install the Linux package:

```sh
sudo apt install ./Wi-Fi.Connect*_amd64.deb
```

WSL requires WSLg for the desktop window and a running Secret Service provider
such as GNOME Keyring for secure credential storage. Linux keyring credentials
are separate from Windows credentials.

### Windows

Run the downloaded Windows `.exe` installer. It adds a Start menu shortcut for
opening the app and a separate **Connect** shortcut for silent connection. It
also makes `wifi` available in new terminals for the current Windows user.

### macOS

Open the downloaded macOS `.dmg` (Apple Silicon) and run the package installer
inside. It installs Wi-Fi Connect in Applications and the `wifi` command in
`/usr/local/bin`.

## Usage

1. Open Wi-Fi Connect and visit **Credentials** from the navigation menu.
2. Enter your college username and password, then save them.
3. Return to the main page and press **Connect**.
4. Hover or click the Wi-Fi logo to open Connect, Credentials, and Settings.
   Use Settings to enable notifications, 5.5-hour automatic relogin, or check
   for the latest release.

The desktop shortcut and platform launcher connect without opening the settings
window. A successful **Connect** shortcut notification shows the connected
Wi-Fi name and local sign-in time when notifications are enabled and allowed by
the operating system.

The CLI supports:

```sh
wifi login
wifi logout
wifi credentials set
```

Running `wifi` without arguments opens the desktop app. Explicit commands
`wifi login`, `wifi logout`, and `wifi credentials set` remain available.

The installed command reads and writes credentials through the same operating
system credential store as the desktop app.

The desktop app binary also accepts `--connect` and `--disconnect`; those modes
are used by the platform shortcuts and schedulers.

## Build from source

### Requirements

- Rust stable and Cargo
- Node.js 22 or later and npm
- Tauri’s platform dependencies: GTK/WebKitGTK on Linux, Microsoft C++ Build
  Tools and WebView2 on Windows, and Xcode command-line tools on macOS
- NSIS to build the Windows installer

Build the Rust CLI/core:

```sh
cargo build --release
```

Build the desktop installer from `desktop/`. Tauri runs the CLI sidecar build
before creating the installer:

```sh
npm ci
npm run tauri -- build
```

Installers are written under `desktop/src-tauri/target/release/bundle/`.
GitHub Actions builds and smoke-checks the Windows installer and packages
Windows and macOS installers on their native runners.

## Updates

Use **Check for updates** in Settings to compare against the latest GitHub
release and open the matching system installer. The app also checks about once
a month and sends a native notification when a newer release is available.

## Security

Credentials are stored in the operating system’s credential store.

The client targets the captive portal at
`https://172.22.2.6/connect/PortalMain`. Its certificate is not valid for the
private IP address, and the client disables certificate validation for this
endpoint. This does not verify the server’s identity and could allow a network
attacker to impersonate the portal. This is a problem at the IT Department's end.

## Project layout

| Path | Purpose |
| --- | --- |
| `assets/` | App logo source files |
| `src/portal.rs` | HTTP session, RSA login, logout, and session verification |
| `src/credentials.rs` | OS credential-store integration |
| `src/main.rs` | `wifi` command-line interface |
| `desktop/dist/` | Desktop interface assets |
| `desktop/src-tauri/` | Native commands, notifications, scheduling, and installers |
| `legacy/windows/` | Sanitized historical Windows reference |

## Contact

<div align="center">
<a href="https://www.linkedin.com/in/daksh-/" target="_blank" rel="noopener noreferrer">
  <img src="https://upload.wikimedia.org/wikipedia/commons/8/81/LinkedIn_icon.svg" width="56" alt="LinkedIn logo" /><br />
  <strong>Daksh Jain on LinkedIn</strong>
</a>
</div>

## Licence

<a href="LICENSE">
  <img src="https://commons.wikimedia.org/wiki/Special:FilePath/License_icon-mit.svg" width="64" alt="MIT License logo" />
</a>

This project is licensed under the MIT License. See [LICENSE](LICENSE).
