const invoke = window.__TAURI__.core.invoke;
const notification = window.__TAURI__.notification;
const form = document.querySelector("#credentials-form");
const usernameField = document.querySelector("#username");
const passwordField = document.querySelector("#password");
const status = document.querySelector("#status");
const badge = document.querySelector("#saved-badge");
const autoRelogin = document.querySelector("#auto-relogin");
const shortcutButton = document.querySelector("#create-shortcut");
const actionButtons = [document.querySelector("#connect"), document.querySelector("#disconnect")];

function showStatus(message, failed = false) {
  status.textContent = message;
  status.classList.toggle("error", failed);
}

async function notify(title, body) {
  try {
    let permitted = await notification.isPermissionGranted();
    if (!permitted) permitted = (await notification.requestPermission()) === "granted";
    if (permitted) notification.sendNotification({ title, body });
  } catch (_) {
    // The in-app status remains available when system notifications are disabled.
  }
}

async function refreshCredentials() {
  const [saved, username] = await Promise.all([
    invoke("credentials_saved"),
    invoke("current_username"),
  ]);
  badge.textContent = saved ? "Saved" : "Not set";
  badge.classList.toggle("is-saved", saved);
  if (username) usernameField.value = username;
}

form.addEventListener("submit", async (event) => {
  event.preventDefault();
  const username = usernameField.value.trim();
  const password = passwordField.value;
  try {
    await invoke("save_credentials", { username, password });
    passwordField.value = "";
    await refreshCredentials();
    showStatus("Credentials saved in your device’s secure credential store.");
  } catch (error) {
    showStatus(String(error), true);
  }
});

async function runAction(button, command, successMessage) {
  actionButtons.forEach((item) => { item.disabled = true; });
  showStatus("Working…");
  try {
    await invoke(command);
    showStatus(successMessage);
    await notify("College Wi-Fi", successMessage);
  } catch (error) {
    showStatus(String(error), true);
    await notify("College Wi-Fi", `Connection request failed: ${String(error)}`);
  } finally {
    actionButtons.forEach((item) => { item.disabled = false; });
  }
}

actionButtons[0].addEventListener("click", (event) => runAction(event.currentTarget, "connect", "Connected to college Wi-Fi."));
actionButtons[1].addEventListener("click", (event) => runAction(event.currentTarget, "disconnect", "Signed out of college Wi-Fi."));

refreshCredentials().catch((error) => showStatus(String(error), true));

invoke("auto_relogin_enabled")
  .then((enabled) => { autoRelogin.checked = enabled; })
  .catch((error) => showStatus(String(error), true));

autoRelogin.addEventListener("change", async () => {
  autoRelogin.disabled = true;
  try {
    await invoke("set_auto_relogin", { enabled: autoRelogin.checked });
    showStatus(autoRelogin.checked
      ? "Automatic relogin enabled. Your device will reconnect every 5.5 hours."
      : "Automatic relogin disabled.");
  } catch (error) {
    autoRelogin.checked = !autoRelogin.checked;
    showStatus(String(error), true);
  } finally {
    autoRelogin.disabled = false;
  }
});

shortcutButton.addEventListener("click", async () => {
  shortcutButton.disabled = true;
  try {
    await invoke("create_connect_shortcut");
    showStatus("One-click shortcut created on your desktop. It connects silently using saved credentials.");
  } catch (error) {
    showStatus(String(error), true);
  } finally {
    shortcutButton.disabled = false;
  }
});
