# College Wi-Fi

Cross-platform, one-click login and logout for the college captive portal.

This is a clean rebuild. The former Windows/Playwright project and HAR captures
are retained under the local, gitignored `archive/` folder as reference only.
The sanitized, publishable Windows source snapshot is in `legacy/windows/`.

## Login design

The connection path is a single lightweight HTTP flow. It loads the portal,
requests fresh RSA settings, encrypts the password using the portal's
client-side algorithm, submits the login request, and verifies authentication
through the portal attributes endpoint. The CLI core is written in Rust with
`reqwest` and `rsa`; it does not launch or bundle a browser engine. A browser
fallback is out of scope unless cross-platform stability testing proves the
HTTP flow cannot be made reliable.

The captured client code indicates that the portal prepends a per-session
login token to the password, UTF-8 encodes it, uses randomized RSA PKCS#1 v1.5
encryption, and reverses the ciphertext bytes in pairs before submission. The
implementation follows that flow and checks both the portal response and the
authenticated session attributes. Live login and logout still need campus
validation.

The campus portal is accessed at `https://172.22.2.6/connect/PortalMain`.
Captured successful browser sessions ignore its TLS certificate error, so the
Rust client does the same for requests to this fixed portal origin. This allows
the captive login flow to run but does not authenticate the server: an active
network attacker could impersonate the portal and replace its RSA key.

## User features

- A short `wifi` command, a Linux application-menu shortcut, and a Windows Start
  menu shortcut use the same login core. Shortcut launches run silently and
  report the result through a native notification.
- The settings window can create a one-click desktop shortcut on Linux,
  Windows, and macOS.
- A small Tauri settings window stores or updates the user's credentials in
  the operating system credential store. The GUI calls the same Rust login
  core as the CLI; portal login itself remains a direct HTTP/RSA flow.
- A one-click logout uses the portal's logoff endpoint and verifies the
  resulting logged-out state.
- Optional 5.5-hour relogin is configured through the OS scheduler: a user
  systemd timer on Linux, Task Scheduler on Windows, or a LaunchAgent on macOS.
- The settings window can enable or disable automatic relogin without asking
  the user to re-enter credentials. Credentials remain in the OS keyring.
- Native notifications report the result of a requested login or logout.

## Security and captures

Never commit credentials, secrets, HAR files, cookies, session tokens, local
logs containing portal data, or signing keys. HAR files can include submitted
passwords and active session data; `.gitignore` excludes them locally.

The Linux `.deb` installs a Linux GUI application; it is not a Windows
installer. It can run under WSL 2 with WSLg on supported Windows versions, but
the Linux credential backend requires a running Secret Service provider such
as GNOME Keyring. Minimal WSL Ubuntu installs may need that service configured
before credentials can be saved. The package currently installs the settings
GUI; the standalone `wifi` CLI is still packaged separately.

## Implementation status and next steps

1. Portal request and RSA behavior have been analyzed from the local HARs.
2. The Rust CLI, keyring storage, and HTTP/RSA login and logout paths compile.
   The short `wifi` command logs in; `wifi login`, `wifi logout`, and
   `wifi credentials set` remain available. The request sequence and response
   fields match the captured traffic; live Rust login still needs campus
   validation.
3. The Tauri settings window has credential save/update, connect, logout,
   native result notifications, an opt-in 5.5-hour relogin toggle, and a
   cross-platform desktop-shortcut action. Linux and Windows installers also
   include a separate silent connect shortcut. The app binary supports
   `--connect` and `--disconnect` for platform launchers. The Debian package
   uses system GTK/WebKit; Windows/macOS installer behavior still needs
   validation on those operating systems.
4. Package platform installers in CI and validate clean installs on Windows,
   macOS, and Linux before publishing releases.
