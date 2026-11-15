const owner = "oblivionium";
const repository = "wifi-connect";
const releaseManifest = new URL("release.json", document.baseURI);
const releasePage = `https://github.com/${owner}/${repository}/releases/latest`;

const platformNames = {
  windows: "Windows",
  macos: "macOS",
  linux: "Linux",
};

const platformMarks = {
  windows: "WIN",
  macos: "MAC",
  linux: "LIN",
};

const platform = detectPlatform();
const platformLabel = document.querySelector("#platform-label");
const installerTitle = document.querySelector("#installer-title");
const installerDescription = document.querySelector("#installer-description");
const downloadButton = document.querySelector("#download-button");
const downloadLabel = document.querySelector("#download-label");
const downloadStatus = document.querySelector("#download-status");
const releaseVersion = document.querySelector("#release-version");
const otherReleaseLabel = document.querySelector("#other-release-label");
const platformDownloads = document.querySelector("#platform-downloads");

platformLabel.textContent = platformNames[platform.key]
  ? `Detected · ${platform.label}`
  : platform.key === "mobile" ? "Mobile device" : "Choose a platform";

loadLatestRelease();

function detectPlatform() {
  const platformHint = navigator.userAgentData?.platform || navigator.platform || "";
  const browserHint = navigator.userAgent || "";
  const hint = `${platformHint} ${browserHint}`.toLowerCase();

  const iPadDesktopMode = /macintosh/.test(hint) && navigator.maxTouchPoints > 1;
  if (/android|iphone|ipad|ipod/.test(hint) || iPadDesktopMode) {
    return { key: "mobile", label: "Mobile device" };
  }
  if (/windows/.test(hint)) return { key: "windows", label: platformNames.windows };
  if (/macintosh|mac os|macos|darwin/.test(hint)) return { key: "macos", label: platformNames.macos };
  if (/linux/.test(hint)) return { key: "linux", label: platformNames.linux };
  return { key: "unknown", label: "your computer" };
}

async function loadLatestRelease() {
  try {
    const response = await fetch(releaseManifest, { cache: "no-store", signal: AbortSignal.timeout(9000) });
    if (!response.ok) throw new Error(`Release manifest returned ${response.status}`);

    const release = await response.json();
    if (!Array.isArray(release.assets)) throw new Error("Release assets are unavailable");
    showRelease(release);
  } catch {
    showReleaseUnavailable();
  }
}

function showRelease(release) {
  const tag = release.tag_name || release.name || "Latest release";
  const releaseDate = formatDate(release.published_at);
  releaseVersion.textContent = releaseDate ? `${tag} · ${releaseDate}` : tag;
  otherReleaseLabel.textContent = tag;

  const assets = release.assets
    .map((asset) => ({ ...asset, platform: classifyAsset(asset.name), url: safeDownloadUrl(asset) }))
    .filter((asset) => asset.platform && asset.url);

  renderPlatformDownloads(assets);

  if (!platformNames[platform.key]) {
    installerTitle.textContent = platform.key === "mobile" ? "Desktop app for your computer" : "Choose your installer";
    installerDescription.textContent = platform.key === "mobile"
      ? "Wi-Fi Connect is a desktop app. Open this page on a Windows, macOS, or Linux computer to get a matching installer."
      : "We couldn’t identify your operating system. Choose an installer below or browse the release page.";
    setPrimaryLink(releasePage, "Browse latest release");
    setStatus("Choose a desktop installer below.", "success");
    return;
  }

  const matchingAssets = assets.filter((asset) => asset.platform === platform.key);
  const selectedAsset = chooseAsset(matchingAssets);

  if (!selectedAsset) {
    const needsManualChoice = matchingAssets.length > 1 && !getArchitectureHint();
    installerTitle.textContent = matchingAssets.length ? `Choose a ${platform.label} installer` : `No ${platform.label} installer found`;
    installerDescription.textContent = matchingAssets.length
      ? needsManualChoice
        ? "There are several installers for your platform. Choose the file that matches your processor."
        : "We couldn’t match an installer to your processor. Check the file names below for alternatives."
      : `The latest release does not include an installer for ${platform.label}. Check the release page for available downloads.`;
    setPrimaryLink(releasePage, "Browse latest release");
    setStatus(matchingAssets.length ? "Check the installer architecture before downloading." : "No matching installer was found.", "error");
    return;
  }

  platformLabel.hidden = true;
  installerTitle.textContent = `${platform.label} installer`;
  installerDescription.textContent = "";
  installerDescription.hidden = true;
  setPrimaryLink(selectedAsset.url, `Download for ${platform.label}`);
  setStatus(`Ready from the latest release${releaseDate ? ` published ${releaseDate}` : ""}.`, "success");
}

function classifyAsset(name) {
  const filename = name.toLowerCase();
  if (/\.(exe|msi)$/.test(filename)) return "windows";
  if (/\.dmg$/.test(filename)) return "macos";
  if (/\.(deb|rpm|appimage)$/.test(filename)) return "linux";
  return null;
}

function safeDownloadUrl(asset) {
  try {
    const url = new URL(asset.browser_download_url);
    return url.protocol === "https:" && url.hostname === "github.com" ? url.href : null;
  } catch {
    return null;
  }
}

function chooseAsset(assets) {
  if (!assets.length) return null;

  const architecture = getArchitectureHint();
  if (assets.length === 1) {
    if (architecture && architectureScore(assets[0].name, architecture) < 0) return null;
    return assets[0];
  }
  if (!architecture) return null;

  const scored = assets
    .map((asset) => ({ asset, score: architectureScore(asset.name, architecture) }))
    .sort((left, right) => right.score - left.score);
  if (scored.length > 1 && scored[0].score === scored[1].score) return null;
  if (scored[0].score < 1) return null;
  return scored[0].asset;
}

function getArchitectureHint() {
  const browserArchitecture = navigator.userAgentData?.architecture?.toLowerCase() || "";
  if (/arm/.test(browserArchitecture)) return "arm64";
  if (/x86|x64/.test(browserArchitecture)) return "x64";

  const hint = `${navigator.platform || ""} ${navigator.userAgent || ""}`.toLowerCase();
  if (/aarch64|arm64/.test(hint)) return "arm64";
  if (/x86_64|amd64|win64|x64/.test(hint)) return "x64";
  if (/i[3-6]86|x86/.test(hint)) return "x86";
  return "";
}

function architectureScore(filename, architecture) {
  const name = filename.toLowerCase();
  const isArm = /aarch64|arm64/.test(name);
  const isX86 = /x86|i386|i686/.test(name) && !/x86_64/.test(name);
  const isX64 = /x64|amd64|x86_64/.test(name);
  if (architecture === "arm64") return isArm ? 4 : isX64 || isX86 ? -2 : 0;
  if (architecture === "x86") return isX86 ? 4 : isArm || isX64 ? -2 : 0;
  if (architecture === "x64") return isX64 ? 4 : isArm || isX86 ? -2 : 0;
  return 0;
}

function renderPlatformDownloads(assets) {
  platformDownloads.replaceChildren();

  const orderedPlatforms = ["windows", "macos", "linux"];
  for (const platformKey of orderedPlatforms) {
    const group = assets.filter((asset) => asset.platform === platformKey);
    for (const asset of group) {
      const link = document.createElement("a");
      link.className = "platform-option";
      link.href = asset.url;
      link.target = "_blank";
      link.rel = "noopener noreferrer";
      link.setAttribute("aria-label", `Download ${asset.name} for ${platformNames[platformKey]}`);

      const mark = document.createElement("span");
      mark.className = "platform-icon";
      mark.setAttribute("aria-hidden", "true");
      mark.textContent = platformMarks[platformKey];

      const copy = document.createElement("span");
      copy.className = "platform-copy";

      const name = document.createElement("strong");
      name.textContent = platformNames[platformKey];
      const filename = document.createElement("small");
      filename.textContent = asset.name;
      copy.append(name, filename);

      const action = document.createElement("span");
      action.className = "platform-action";
      action.textContent = "Download";
      link.append(mark, copy, action);
      platformDownloads.append(link);
    }
  }

  if (assets.length === 0) {
    const note = document.createElement("p");
    note.className = "download-list-note";
    note.textContent = "No supported installer files were listed. Browse the release page for details.";
    platformDownloads.append(note);
  }
}

function setPrimaryLink(url, label) {
  downloadButton.href = url;
  downloadLabel.textContent = label;
}

function setStatus(message, state) {
  downloadStatus.textContent = message;
  downloadStatus.dataset.state = state;
}

function showReleaseUnavailable() {
  installerTitle.textContent = "Latest release unavailable";
  installerDescription.textContent = "Release details could not be loaded. You can still browse GitHub Releases and choose an installer there.";
  releaseVersion.textContent = "Release lookup failed";
  otherReleaseLabel.textContent = "GitHub Releases";
  platformDownloads.replaceChildren();
  const note = document.createElement("p");
  note.className = "download-list-note";
  note.textContent = "The installer list could not be loaded right now.";
  platformDownloads.append(note);
  setPrimaryLink(releasePage, "Browse latest release");
  setStatus("Couldn’t load release details. Try again in a moment.", "error");
}

function formatDate(value) {
  if (!value) return "";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return "";
  return new Intl.DateTimeFormat(undefined, { year: "numeric", month: "short", day: "numeric" }).format(date);
}
