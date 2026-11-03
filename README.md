<div align="center">

<img src="assets/wifi-connect.png" width="112" alt="Wi-Fi Connect mark" />

# Wi-Fi Connect

One click to sign in to the campus network.

[![Rust](https://img.shields.io/badge/Rust-core-CE422B?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Tauri](https://img.shields.io/badge/Tauri-desktop-24C8DB?style=flat-square&logo=tauri&logoColor=white)](https://tauri.app/)
[![JavaScript](https://img.shields.io/badge/JavaScript-interface-F7DF1E?style=flat-square&logo=javascript&logoColor=111111)](https://developer.mozilla.org/en-US/docs/Web/JavaScript)
[![GitHub Actions](https://img.shields.io/badge/GitHub_Actions-builds-2088FF?style=flat-square&logo=githubactions&logoColor=white)](https://github.com/Oblivionium/wifi/actions)

[Features](#features) · [Install](#installation) · [Usage](#usage) · [Security](#security)

</div>

Wi-Fi Connect signs in to the campus captive portal on Windows, macOS, and
Linux. The Rust client follows the portal’s HTTP and RSA login flow directly;
the desktop application uses the operating system’s webview and does not bundle
a browser engine.

## Contents

- [Features](#features)
- [Technology](#technology)
- [Installation](#installation)
- [Usage](#usage)
- [Build from source](#build-from-source)
- [Security](#security)
- [Project layout](#project-layout)
- [Status](#status)

## Features

- One-click connection from the desktop app or a silent desktop shortcut.
- Credential entry and updates in a separate settings page; passwords are kept
  in the system credential store.
- Native notifications for successful and failed connection requests, including
  scheduled relogins. Notifications can be disabled in Settings.
- Optional automatic relogin every 5.5 hours.
- Logout through the portal’s logoff endpoint.
- `wifi` command-line login, logout, and credential setup.

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
the portal session and `rsa` for password encryption. The main screen checks
portal authentication on launch. After a successful connection, the button
stays on for 5.5 hours or until you disconnect.

## Installation

Download the installers from the [GitHub Releases page](https://github.com/Oblivionium/wifi/releases).
The published `0.1.0` release provides Linux, Windows, and macOS installers;
`0.2.0` is in development. Each installer includes the `wifi` CLI. Linux
installs it as `/usr/bin/wifi`; Windows adds the installed app directory to the
current user's `PATH`; macOS's `.pkg` installs it as `/usr/local/bin/wifi`.

### Linux

Install the `.deb` package:

```sh
sudo apt install './Wi-Fi.Connect_0.1.0_amd64.deb'
```

If upgrading from the former `college-wi-fi` package, remove it first with
`sudo apt remove college-wi-fi`.

WSL requires WSLg for the desktop window and a running Secret Service provider
such as GNOME Keyring for secure credential storage. Linux keyring credentials
are separate from Windows credentials.

### Windows

Run `Wi-Fi.Connect_0.1.0_x64-setup.exe`. The installer adds a Start menu
shortcut for opening the app and a separate **Connect** shortcut for silent
connection. It also makes `wifi` available in new terminals for the current
Windows user.

### macOS

Open `Wi-Fi.Connect_0.1.0_aarch64.dmg` and move Wi-Fi Connect to Applications.
The app can create a one-click desktop shortcut from Settings. To install the
`wifi` command into `/usr/local/bin`, use `Wi-Fi.Connect_0.1.0.pkg`.

Starting with `0.2.0`, the single DMG contains a package installer that installs
both the app in Applications and the `wifi` command in `/usr/local/bin`.

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

Starting with `0.2.0`, running `wifi` without arguments opens the desktop app.
The published `0.1.0` command defaults to login; `wifi login` works explicitly
in both versions.

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

Credentials are stored in the operating system’s credential store. Do not put
credentials, session cookies, HAR captures, local logs, or signing keys in the
repository. The local `archive/` directory is ignored by Git and must remain
private.

The campus portal is accessed at
`https://172.22.2.6/connect/PortalMain`. Its certificate is not valid for the
private IP address, so the client currently accepts the portal’s certificate
without validation. This allows the login flow to work but does not verify the
server’s identity; a network attacker could impersonate the portal.

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

[LinkedIn](https://www.linkedin.com/in/daksh-/)

## Licence

This project is licensed under the MIT License. See [LICENSE](LICENSE).
