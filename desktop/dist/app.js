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

function message(target, text, error = false) {
  target.textContent = text;
  target.classList.toggle("error", error);
}

function setPortalState(state, label) {
  isAuthenticated = state;
  toggle.setAttribute("aria-checked", String(state === true));
  toggle.disabled = state === false && !hasCredentials;
  portalStatus.dataset.state = state === true ? "connected" : state === null ? "error" : "disconnected";
  portalStatus.textContent = label || (state === true ? "Logged in" : state === false ? "Logged out" : "Status unavailable");
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
  if (isAuthenticated === null) setPortalState(null, "Checking status");
  try {
    const authenticated = await invoke("portal_authenticated");
    setPortalState(authenticated);
  } catch (_) {
    if (isAuthenticated === null) setPortalState(null);
    else {
      portalStatus.dataset.state = "error";
      portalStatus.textContent = "Status unavailable";
    }
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
  portalStatus.dataset.state = "pending";
  portalStatus.textContent = isAuthenticated ? "Signing out" : "Connecting";
  try {
    if (isAuthenticated) {
      await invoke("disconnect");
      setPortalState(false);
      await sendNotification("College Wi-Fi", "Logged out");
    } else {
      await invoke("connect");
      setPortalState(true);
      await sendNotification("College Wi-Fi", "Connected");
    }
  } catch (error) {
    portalStatus.dataset.state = "error";
    portalStatus.textContent = isAuthenticated ? "Logout failed" : "Connection failed";
    await sendNotification("College Wi-Fi", String(error));
  } finally {
    await Promise.allSettled([refreshPortalStatus(), refreshCredentials()]);
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

window.addEventListener("focus", refreshPortalStatus);
initialize();
