<div align="center">

<img src="assets/wifi.png" width="96" alt="Wi-Fi Connect mark" />

# Wi-Fi Connect

**Campus Wi-Fi sign-in for your desktop**<br />
Desktop app and CLI for Windows, macOS, and Linux.

<p>
  <a href="https://oblivionium.github.io/wifi-connect/">
    <img alt="Get installer for your OS" src="https://img.shields.io/badge/Get%20installer-Choose%20your%20OS-5B8750?style=for-the-badge&labelColor=202820" />
  </a>
  <a href="https://github.com/Oblivionium/wifi-connect">
    <img alt="View source on GitHub" src="https://img.shields.io/badge/View%20source-GitHub-374151?style=for-the-badge&logo=github&logoColor=white" />
  </a>
</p>

[![Rust](https://img.shields.io/badge/Rust-core-CE422B?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Tauri](https://img.shields.io/badge/Tauri-desktop-24C8DB?style=flat-square&logo=tauri&logoColor=white)](https://tauri.app/)
[![JavaScript](https://img.shields.io/badge/JavaScript-interface-F7DF1E?style=flat-square&logo=javascript&logoColor=111111)](https://developer.mozilla.org/en-US/docs/Web/JavaScript)
[![GitHub Actions](https://img.shields.io/badge/GitHub_Actions-builds-2088FF?style=flat-square&logo=githubactions&logoColor=white)](https://github.com/Oblivionium/wifi-connect/actions)

[Features](#features) · [Technology](#technology) · [Installation](#installation) · [Usage](#usage) · [Build](#build-from-source) · [Security](#security)

</div>

The application implements the campus captive portal’s HTTP and RSA login flow
on Windows, macOS, and Linux. The desktop application uses the operating
system’s webview and does not bundle a browser engine.

## Contents

[Features](#features) · [Technology](#technology) · [Installation](#installation) · [Usage](#usage) · [Build from source](#build-from-source) · [Security](#security) · [Project layout](#project-layout)

## Features

- **One-click connection** from the desktop app or a silent desktop shortcut.
- **Credential management** in a separate page, with passwords stored in the
  operating system’s credential store.
- **Native notifications** for connection results and scheduled relogins;
  notifications can be disabled in Settings.
- **Optional auto-relogin** every 5.5 hours.
- **Portal logout** through the logoff endpoint.
- **CLI commands** for `wifi login`, `wifi logout`, and `wifi credentials set`.

## Technology

<div align="center">

<a href="https://www.rust-lang.org/"><img alt="Rust" height="38" src="https://img.shields.io/badge/Rust-CE422B?style=for-the-badge&logo=rust&logoColor=white"></a>
<a href="https://tauri.app/"><img alt="Tauri 2" height="38" src="https://img.shields.io/badge/Tauri_2-24C8DB?style=for-the-badge&logo=tauri&logoColor=white"></a>
<a href="https://developer.mozilla.org/en-US/docs/Web/HTML"><img alt="HTML" height="38" src="https://img.shields.io/badge/HTML-E34F26?style=for-the-badge&logo=html5&logoColor=white"></a>
<a href="https://developer.mozilla.org/en-US/docs/Web/CSS"><img alt="CSS" height="38" src="https://img.shields.io/badge/CSS-663399?style=for-the-badge&logo=css&logoColor=white"></a>
<a href="https://developer.mozilla.org/en-US/docs/Web/JavaScript"><img alt="JavaScript" height="38" src="https://img.shields.io/badge/JavaScript-F7DF1E?style=for-the-badge&logo=javascript&logoColor=111111"></a>
<a href="https://github.com/seanmonstar/reqwest"><img alt="reqwest" height="38" src="https://img.shields.io/badge/reqwest-HTTP_client-59636E?style=for-the-badge"></a>
<a href="https://github.com/RustCrypto/RSA"><img alt="RustCrypto RSA" height="38" src="https://img.shields.io/badge/RustCrypto-RSA-59636E?style=for-the-badge"></a>

</div>

The UI is plain HTML, CSS, and JavaScript. Tauri provides the native window,
system notifications, and platform packaging. The Rust core uses `reqwest` for
the portal session and `rsa` for password encryption. While the targeted portal
is available, the main screen checks authentication on launch and keeps the
connection button on for 5.5 hours or until you disconnect.

## Installation

The download page detects your operating system and offers the latest matching
installer. Every installer includes the `wifi` CLI. Choose your platform below
for installation steps.

<div align="center">
  <a href="https://oblivionium.github.io/wifi-connect/">
    <img alt="Get installer for your OS" src="https://img.shields.io/badge/Get%20installer-Download%20for%20your%20OS-5B8750?style=for-the-badge&labelColor=202820" />
  </a>
</div>

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
   Use Settings to enable notifications or 5.5-hour automatic relogin.

The desktop shortcut and platform launcher connect without opening the settings
window. Connection results appear as native notifications when notifications
are enabled and allowed by the operating system.

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

## Security

Credentials are stored in the operating system’s credential store.

The client targets the captive portal at
`https://172.22.2.6/connect/PortalMain`. Its certificate is not valid for the
private IP address, and the client disables certificate validation for this
endpoint. This does not verify the server’s identity and could allow a network
attacker to impersonate the portal. This is a problem at the IT Department's end.

The college has reactivated this portal, but the `v0.2.0` login flow has not yet
been validated against it.

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
  <a href="https://www.linkedin.com/in/daksh-/">
    <img src="https://upload.wikimedia.org/wikipedia/commons/8/81/LinkedIn_icon.svg" width="56" alt="LinkedIn logo" /><br />
    <strong>Daksh Jain on LinkedIn</strong>
  </a>
</div>

## Licence

[![MIT License](https://commons.wikimedia.org/wiki/Special:FilePath/License_icon-mit.svg)](LICENSE)

This project is licensed under the MIT License. See [LICENSE](LICENSE).
