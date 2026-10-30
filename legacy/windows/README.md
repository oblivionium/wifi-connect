# Archived Windows implementation

This folder is a sanitized historical snapshot of the former Windows-only
Playwright implementation. The new cross-platform project lives at the repo
root. This legacy code is retained for reference and may no longer work with
the current captive portal.

The original local script had credentials embedded in its source. In this
public snapshot, `wifi-login.js` requires `WIFI_USERNAME` and `WIFI_PASSWORD`
to be set in its process environment and exits if either is missing. Never
commit real credentials, HAR captures, logs, local shortcuts, or `node_modules`.

Install the Playwright package with `npm ci`. The toast script expects the
BurntToast PowerShell module to be installed separately.
