const invoke = window.__TAURI__.core.invoke;
const notification = window.__TAURI__.notification;
const menuWrap = document.querySelector("#menu-wrap");
const logoButton = document.querySelector("#logo-button");
const pageMenu = document.querySelector("#page-menu");
const pages = [...document.querySelectorAll(".page")];
const pageLinks = [...document.querySelectorAll(".page-link")];
const toggle = document.querySelector("#connection-toggle");
const portalStatus = document.querySelector("#portal-status");
const form = document.querySelector("#credentials-form");
const username = document.querySelector("#username");
const password = document.querySelector("#password");
const credentialsStatus = document.querySelector("#credentials-status");
const settingsStatus = document.querySelector("#settings-status");
const notificationsToggle = document.querySelector("#notifications-toggle");
const autoReloginToggle = document.querySelector("#auto-relogin");
const shortcutButton = document.querySelector("#create-shortcut");
const checkUpdatesButton = document.querySelector("#check-for-updates");
const installUpdateButton = document.querySelector("#install-update");
const updateStatus = document.querySelector("#update-status");
let availableUpdateVersion = null;
let isAuthenticated = null;
let hasCredentials = false;
let notificationsEnabled = true;
let connectionPending = false;
let connectionExpiryTimer = null;
const CONNECTION_WINDOW_MS = 5.5 * 60 * 60 * 1000;
const CONNECTION_EXPIRY_KEY = "college-wifi-connection-expires-at";
const UPDATE_CHECK_INTERVAL_MS = 30 * 24 * 60 * 60 * 1000;
const UPDATE_CHECK_KEY = "wifi-connect-last-update-check-at";
let updateCheckInProgress = false;

function message(target, text, error = false) {
  target.textContent = text;
  target.classList.toggle("error", error);
}

function setStatus(state, label) {
  const nextLabel = label || (state === "connected" ? "Connected" : state === "disconnected" ? "Not connected" : "Status unavailable");
  if (portalStatus.dataset.state !== state || portalStatus.textContent !== nextLabel) {
    portalStatus.classList.remove("status-change");
    void portalStatus.offsetWidth;
    portalStatus.classList.add("status-change");
  }
  portalStatus.dataset.state = state;
  portalStatus.textContent = nextLabel;
}

function setPortalState(state, label) {
  const changed = state !== isAuthenticated;
  isAuthenticated = state;
  toggle.setAttribute("aria-checked", String(state === true));
  if (changed) {
    toggle.classList.remove("state-change");
    void toggle.offsetWidth;
    toggle.classList.add("state-change");
  }
  toggle.disabled = connectionPending || (state === false && !hasCredentials);
  setStatus(state === true ? "connected" : state === null ? "error" : "disconnected", label);
}

function setConnectionPending(pending) {
  connectionPending = pending;
  toggle.setAttribute("aria-busy", String(pending));
  toggle.classList.toggle("is-pending", pending);
  toggle.disabled = pending || (isAuthenticated !== true && !hasCredentials);
}

function clearConnectionExpiry() {
  window.clearTimeout(connectionExpiryTimer);
  connectionExpiryTimer = null;
  localStorage.removeItem(CONNECTION_EXPIRY_KEY);
}

function scheduleConnectionExpiry(expiresAt) {
  window.clearTimeout(connectionExpiryTimer);
  connectionExpiryTimer = window.setTimeout(() => {
    if (Date.now() < expiresAt) return scheduleConnectionExpiry(expiresAt);
    clearConnectionExpiry();
    setPortalState(false, "Not connected");
  }, Math.max(0, expiresAt - Date.now()));
}

function markConnected() {
  const expiresAt = Date.now() + CONNECTION_WINDOW_MS;
  localStorage.setItem(CONNECTION_EXPIRY_KEY, String(expiresAt));
  scheduleConnectionExpiry(expiresAt);
  setPortalState(true, "Connected");
}

function refreshConnectionExpiry() {
  const expiresAt = Number(localStorage.getItem(CONNECTION_EXPIRY_KEY));
  if (!expiresAt) return;
  if (expiresAt <= Date.now()) {
    clearConnectionExpiry();
    setPortalState(false, "Not connected");
  } else {
    scheduleConnectionExpiry(expiresAt);
  }
}

function showPage(name) {
  pages.forEach((page) => {
    const active = page.id === `page-${name}`;
    page.hidden = !active;
    page.classList.toggle("active", active);
  });
  pageLinks.forEach((link) => link.classList.toggle("active", link.dataset.page === name));
  menuWrap.classList.remove("open");
  logoButton.setAttribute("aria-expanded", "false");
}

async function refreshPortalStatus() {
  const savedExpiry = Number(localStorage.getItem(CONNECTION_EXPIRY_KEY));
  if (savedExpiry) {
    if (savedExpiry > Date.now()) {
      setPortalState(true, "Connected");
      scheduleConnectionExpiry(savedExpiry);
    } else {
      clearConnectionExpiry();
      setPortalState(false, "Not connected");
    }
    return;
  }
  if (isAuthenticated === null) setStatus("pending", "Checking…");
  try {
    const authenticated = await invoke("portal_authenticated");
    if (authenticated) markConnected();
    else setPortalState(false, "Not connected");
  } catch (_) {
    if (isAuthenticated === null) setPortalState(null, "Status unavailable");
    else setStatus("error", "Status unavailable");
  }
}

async function refreshCredentials() {
  const [saved, savedUsername] = await Promise.all([invoke("credentials_saved"), invoke("current_username")]);
  hasCredentials = saved;
  if (savedUsername) username.value = savedUsername;
  toggle.disabled = connectionPending || (isAuthenticated !== true && !hasCredentials);
}

async function sendNotification(title, body) {
  if (!notificationsEnabled) return;
  try {
    let permitted = await notification.isPermissionGranted();
    if (!permitted) permitted = (await notification.requestPermission()) === "granted";
    if (permitted) await notification.sendNotification({ title, body });
  } catch (_) {
    // The status line remains the in-app result.
  }
}

logoButton.addEventListener("click", () => {
  const open = !menuWrap.classList.contains("open");
  menuWrap.classList.toggle("open", open);
  logoButton.setAttribute("aria-expanded", String(open));
});
pageLinks.forEach((link) => link.addEventListener("click", () => showPage(link.dataset.page)));
document.addEventListener("click", (event) => {
  if (!menuWrap.contains(event.target)) {
    menuWrap.classList.remove("open");
    logoButton.setAttribute("aria-expanded", "false");
  }
});
document.addEventListener("keydown", (event) => {
  if (event.key === "Escape") {
    menuWrap.classList.remove("open");
    logoButton.setAttribute("aria-expanded", "false");
  }
});

toggle.addEventListener("click", async () => {
  setConnectionPending(true);
  setStatus("pending", isAuthenticated ? "Disconnecting…" : "Connecting…");
  try {
    if (isAuthenticated) {
      await invoke("disconnect");
      clearConnectionExpiry();
      setPortalState(false, "Not connected");
      setConnectionPending(false);
      await sendNotification("Wi-Fi Connect", "Logged out");
    } else {
      await invoke("connect");
      markConnected();
      setConnectionPending(false);
      await sendNotification("Wi-Fi Connect", "Connected");
    }
  } catch (error) {
    setStatus("error", isAuthenticated ? "Disconnect failed" : "Connection failed");
    setConnectionPending(false);
    await sendNotification("Wi-Fi Connect", String(error));
  } finally {
    setConnectionPending(false);
    await refreshCredentials();
  }
});

form.addEventListener("submit", async (event) => {
  event.preventDefault();
  const saveButton = form.querySelector("button[type=submit]");
  saveButton.disabled = true;
  try {
    await invoke("save_credentials", { username: username.value.trim(), password: password.value });
    password.value = "";
    hasCredentials = true;
    toggle.disabled = false;
    message(credentialsStatus, "Saved");
  } catch (error) {
    message(credentialsStatus, String(error), true);
  } finally {
    saveButton.disabled = false;
  }
});

notificationsToggle.addEventListener("change", async () => {
  const requested = notificationsToggle.checked;
  notificationsToggle.disabled = true;
  try {
    if (requested && !(await notification.isPermissionGranted())) {
      const permission = await notification.requestPermission();
      if (permission !== "granted") throw new Error("Notifications are blocked by the operating system");
    }
    await invoke("set_notifications_enabled", { enabled: requested });
    notificationsEnabled = requested;
    message(settingsStatus, "");
    if (requested) void maybeCheckForMonthlyUpdate();
  } catch (error) {
    notificationsToggle.checked = !requested;
    message(settingsStatus, String(error), true);
  } finally {
    notificationsToggle.disabled = false;
  }
});

autoReloginToggle.addEventListener("change", async () => {
  const requested = autoReloginToggle.checked;
  autoReloginToggle.disabled = true;
  try {
    await invoke("set_auto_relogin", { enabled: requested });
    message(settingsStatus, "");
  } catch (error) {
    autoReloginToggle.checked = !requested;
    message(settingsStatus, String(error), true);
  } finally {
    autoReloginToggle.disabled = false;
  }
});

shortcutButton.addEventListener("click", async () => {
  shortcutButton.disabled = true;
  try {
    await invoke("create_connect_shortcut");
    message(settingsStatus, "Shortcut created");
  } catch (error) {
    message(settingsStatus, String(error), true);
  } finally {
    shortcutButton.disabled = false;
  }
});

checkUpdatesButton.addEventListener("click", async () => {
  if (updateCheckInProgress) return;
  updateCheckInProgress = true;
  checkUpdatesButton.disabled = true;
  installUpdateButton.hidden = true;
  availableUpdateVersion = null;
  message(updateStatus, "Checking GitHub Releases…");
  try {
    const update = await invoke("check_for_updates");
    recordUpdateCheck();
    if (!update.updateAvailable) {
      message(updateStatus, `You're up to date (${update.currentVersion}).`);
    } else if (!update.installerAvailable) {
      message(updateStatus, `${update.latestVersion} is available, but no installer for this device was published.`, true);
    } else {
      availableUpdateVersion = update.latestVersion;
      installUpdateButton.textContent = `Install ${update.latestVersion}`;
      installUpdateButton.hidden = false;
      message(updateStatus, `Version ${update.latestVersion} is available.`);
    }
  } catch (error) {
    message(updateStatus, `Could not check for updates: ${String(error)}`, true);
  } finally {
    updateCheckInProgress = false;
    checkUpdatesButton.disabled = false;
  }
});

installUpdateButton.addEventListener("click", async () => {
  if (!availableUpdateVersion) return;
  const version = availableUpdateVersion;
  checkUpdatesButton.disabled = true;
  installUpdateButton.disabled = true;
  message(updateStatus, `Downloading ${version}. The app will close when the installer opens.`);
  try {
    await invoke("install_update", { expectedVersion: version });
    message(updateStatus, `Opening the ${version} installer…`);
  } catch (error) {
    message(updateStatus, `Could not install the update: ${String(error)}`, true);
  } finally {
    checkUpdatesButton.disabled = false;
    installUpdateButton.disabled = false;
  }
});

function recordUpdateCheck() {
  try {
    localStorage.setItem(UPDATE_CHECK_KEY, String(Date.now()));
  } catch (_) {
    // A storage restriction should not turn a successful release check into an error.
  }
}

function updateCheckIsDue() {
  try {
    const lastCheck = Number(localStorage.getItem(UPDATE_CHECK_KEY));
    const now = Date.now();
    return !(lastCheck > 0 && now >= lastCheck && now - lastCheck < UPDATE_CHECK_INTERVAL_MS);
  } catch (_) {
    return true;
  }
}

async function maybeCheckForMonthlyUpdate() {
  if (!notificationsEnabled || updateCheckInProgress || !updateCheckIsDue()) return;
  updateCheckInProgress = true;
  checkUpdatesButton.disabled = true;
  try {
    const update = await invoke("check_for_updates");
    recordUpdateCheck();
    if (update.updateAvailable) {
      let nextStep = "Check GitHub Releases for an installer for your device.";
      if (update.installerAvailable) {
        availableUpdateVersion = update.latestVersion;
        installUpdateButton.textContent = `Install ${update.latestVersion}`;
        installUpdateButton.hidden = false;
        message(updateStatus, `Version ${update.latestVersion} is available.`);
        nextStep = "Open Settings and choose Install to update.";
      }
      await sendNotification("Wi-Fi Connect update available", `${update.latestVersion} is available. ${nextStep}`);
    }
  } catch (_) {
    // Try again on a later app launch or daily check if GitHub is unreachable.
  } finally {
    updateCheckInProgress = false;
    checkUpdatesButton.disabled = false;
  }
}

async function initialize() {
  const [, , savedNotifications, scheduled] = await Promise.allSettled([
    refreshPortalStatus(),
    refreshCredentials(),
    invoke("notifications_enabled"),
    invoke("auto_relogin_enabled"),
  ]);
  if (savedNotifications.status === "fulfilled") {
    notificationsEnabled = savedNotifications.value;
    notificationsToggle.checked = notificationsEnabled;
    if (notificationsEnabled && !(await notification.isPermissionGranted())) await notification.requestPermission();
  }
  if (scheduled.status === "fulfilled") autoReloginToggle.checked = scheduled.value;
  else message(settingsStatus, `Could not refresh the auto reconnect schedule: ${String(scheduled.reason)}`, true);
  void maybeCheckForMonthlyUpdate();
}

window.setInterval(() => void maybeCheckForMonthlyUpdate(), 24 * 60 * 60 * 1000);
window.addEventListener("focus", () => {
  refreshConnectionExpiry();
  void maybeCheckForMonthlyUpdate();
});
initialize();
