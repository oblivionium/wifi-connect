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
let isAuthenticated = null;
let hasCredentials = false;
let notificationsEnabled = true;
let connectionExpiryTimer = null;
const CONNECTION_WINDOW_MS = 5.5 * 60 * 60 * 1000;
const CONNECTION_EXPIRY_KEY = "college-wifi-connection-expires-at";

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
  toggle.disabled = state === false && !hasCredentials;
  setStatus(state === true ? "connected" : state === null ? "error" : "disconnected", label);
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
  toggle.disabled = isAuthenticated !== true && !hasCredentials;
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
  toggle.disabled = true;
  setStatus("pending", isAuthenticated ? "Disconnecting…" : "Connecting…");
  try {
    if (isAuthenticated) {
      await invoke("disconnect");
      clearConnectionExpiry();
      setPortalState(false, "Not connected");
      await sendNotification("Wi-Fi Connect", "Logged out");
    } else {
      await invoke("connect");
      markConnected();
      await sendNotification("Wi-Fi Connect", "Connected");
    }
  } catch (error) {
    setStatus("error", isAuthenticated ? "Disconnect failed" : "Connection failed");
    await sendNotification("Wi-Fi Connect", String(error));
  } finally {
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
}

window.addEventListener("focus", refreshConnectionExpiry);
initialize();
