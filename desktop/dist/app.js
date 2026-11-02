const invoke = window.__TAURI__.core.invoke;
const notification = window.__TAURI__.notification;
const menu = document.querySelector("#side-menu");
const scrim = document.querySelector("#scrim");
const menuButton = document.querySelector("#menu-button");
const pages = [...document.querySelectorAll(".page")];
const navLinks = [...document.querySelectorAll("[data-page]")];
const form = document.querySelector("#credentials-form");
const usernameField = document.querySelector("#username");
const passwordField = document.querySelector("#password");
const badge = document.querySelector("#saved-badge");
const summary = document.querySelector("#credential-summary");
const credentialStatus = document.querySelector("#credentials-status");
const settingsStatus = document.querySelector("#settings-status");
const status = document.querySelector("#status");
const notificationToggle = document.querySelector("#notifications-toggle");
const autoRelogin = document.querySelector("#auto-relogin");
const actionButtons = [document.querySelector("#connect"), document.querySelector("#disconnect")];
let notificationsEnabled = true;

function showStatus(target, message, failed = false) {
  target.textContent = message;
  target.classList.toggle("error", failed);
}

function openMenu(open) {
  menu.classList.toggle("open", open);
  scrim.classList.toggle("visible", open);
  menu.setAttribute("aria-hidden", String(!open));
  menuButton.setAttribute("aria-expanded", String(open));
}

function showPage(name) {
  pages.forEach((page) => {
    const active = page.id === `page-${name}`;
    page.hidden = !active;
    page.classList.toggle("active", active);
  });
  document.querySelectorAll(".menu-link").forEach((link) => link.classList.toggle("active", link.dataset.page === name));
  openMenu(false);
}

async function notify(title, body) {
  if (!notificationsEnabled) return;
  try {
    let permitted = await notification.isPermissionGranted();
    if (!permitted) permitted = (await notification.requestPermission()) === "granted";
    if (permitted) {
      await notification.sendNotification({ title, body });
    }
  } catch (_) {
    // The in-app result remains visible if OS notifications are unavailable.
  }
}

async function refreshCredentials() {
  const [saved, username] = await Promise.all([invoke("credentials_saved"), invoke("current_username")]);
  badge.textContent = saved ? "Saved" : "Not set";
  badge.classList.toggle("is-saved", saved);
  summary.textContent = saved ? `Saved account${username ? ` · ${username}` : ""}` : "Credentials not saved";
  if (username) usernameField.value = username;
  actionButtons[0].disabled = !saved;
  actionButtons[1].disabled = false;
}

menuButton.addEventListener("click", () => openMenu(!menu.classList.contains("open")));
scrim.addEventListener("click", () => openMenu(false));
navLinks.forEach((link) => link.addEventListener("click", () => showPage(link.dataset.page)));
document.querySelector("#settings-button").addEventListener("click", () => showPage("settings"));
document.addEventListener("keydown", (event) => { if (event.key === "Escape") openMenu(false); });

form.addEventListener("submit", async (event) => {
  event.preventDefault();
  try {
    await invoke("save_credentials", { username: usernameField.value.trim(), password: passwordField.value });
    passwordField.value = "";
    await refreshCredentials();
    showStatus(credentialStatus, "Credentials saved securely on this device.");
  } catch (error) {
    showStatus(credentialStatus, String(error), true);
  }
});

async function runAction(command, successMessage) {
  actionButtons.forEach((button) => { button.disabled = true; });
  showStatus(status, "Contacting the sign-in portal…");
  try {
    await invoke(command);
    showStatus(status, successMessage);
    await notify("College Wi-Fi", successMessage);
  } catch (error) {
    showStatus(status, String(error), true);
    await notify("College Wi-Fi", `Request failed: ${String(error)}`);
  } finally {
    await refreshCredentials().catch(() => {});
  }
}

actionButtons[0].addEventListener("click", () => runAction("connect", "Connected to college Wi-Fi."));
actionButtons[1].addEventListener("click", () => runAction("disconnect", "Signed out of the college portal."));

notificationToggle.addEventListener("change", async () => {
  notificationToggle.disabled = true;
  const requested = notificationToggle.checked;
  try {
    if (requested && !(await notification.isPermissionGranted())) {
      const permission = await notification.requestPermission();
      if (permission !== "granted") throw new Error("Allow notifications in your operating system settings to enable this option.");
    }
    await invoke("set_notifications_enabled", { enabled: requested });
    notificationsEnabled = requested;
    showStatus(settingsStatus, requested ? "Native notifications enabled." : "Native notifications disabled.");
  } catch (error) {
    notificationToggle.checked = !requested;
    showStatus(settingsStatus, String(error), true);
  } finally {
    notificationToggle.disabled = false;
  }
});

autoRelogin.addEventListener("change", async () => {
  autoRelogin.disabled = true;
  const requested = autoRelogin.checked;
  try {
    await invoke("set_auto_relogin", { enabled: requested });
    showStatus(settingsStatus, requested ? "Automatic relogin enabled for every 5.5 hours." : "Automatic relogin disabled.");
  } catch (error) {
    autoRelogin.checked = !requested;
    showStatus(settingsStatus, String(error), true);
  } finally {
    autoRelogin.disabled = false;
  }
});

document.querySelector("#create-shortcut").addEventListener("click", async (event) => {
  const button = event.currentTarget;
  button.disabled = true;
  try {
    await invoke("create_connect_shortcut");
    showStatus(settingsStatus, "One-click desktop shortcut created.");
  } catch (error) {
    showStatus(settingsStatus, String(error), true);
  } finally {
    button.disabled = false;
  }
});

async function initialize() {
  try {
    const [savedNotifications, scheduled] = await Promise.all([
      invoke("notifications_enabled"),
      invoke("auto_relogin_enabled"),
    ]);
    notificationsEnabled = savedNotifications;
    notificationToggle.checked = savedNotifications;
    autoRelogin.checked = scheduled;
    if (savedNotifications && !(await notification.isPermissionGranted())) {
      await notification.requestPermission();
    }
    await refreshCredentials();
  } catch (error) {
    showStatus(status, String(error), true);
  }
}

initialize();
