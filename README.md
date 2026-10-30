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
implementation must confirm each detail against the captured flow and verify
success using the portal response, not HTTP status alone.

## Planned user features

- A short `wifi` command and a desktop/Start menu shortcut use the same login
  core.
- A small settings window stores or updates the user's credentials in the
  operating system credential store. The planned UI shell is Tauri so it can
  reuse the Rust core without embedding a separate browser runtime in login.
- A one-click logout uses the portal's logoff endpoint and verifies the
  resulting logged-out state.
- Optional relogin runs every six hours, with a campus-network check and
  bounded retry behavior.
- Native notifications report the result of a requested login or logout.

## Security and captures

Never commit credentials, secrets, HAR files, cookies, session tokens, local
logs containing portal data, or signing keys. HAR files can include submitted
passwords and active session data; `.gitignore` excludes them locally.

## Implementation status and next steps

1. Portal request and RSA behavior have been analyzed from the local HARs.
2. The initial Rust CLI, keyring storage, and HTTP/RSA login and logout paths
   are in place; they still need live validation on the college network.
3. Add the small credential settings UI, notifications, desktop shortcuts,
   and optional six-hour scheduling on top of the same core.
4. Package platform installers in CI and validate clean installs on Windows,
   macOS, and Linux before publishing releases.
