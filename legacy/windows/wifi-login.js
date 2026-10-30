const ssid = process.argv[2] || 'UNKNOWN_SSID';
const { chromium } = require('playwright');
const fs = require('fs');
const { execFile } = require('child_process');

const logFile = 'C:\\wifi\\wifi-login.log';

const username = process.env.WIFI_USERNAME;
const password = process.env.WIFI_PASSWORD;
if (!username || !password) {
    throw new Error('Set WIFI_USERNAME and WIFI_PASSWORD before running the legacy script.');
}


function logLine(text)
{
    const ts = new Date().toLocaleString('en-GB', {
        day: '2-digit', month: 'short', year: 'numeric',
        hour: '2-digit', minute: '2-digit', second: '2-digit'
    });
    fs.appendFileSync(logFile, `[${ts}] ${text}\n`);
}

(async () =>
{
    const browser = await chromium.launch({ headless: true });

    const context = await browser.newContext({
        ignoreHTTPSErrors: true
    });

    const page = await context.newPage();


    let alreadyOnline = false;

    // 1. Connectivity check (DO NOT exit)
    try
    {
        await page.goto('https://www.google.com', { timeout: 5000 });
        alreadyOnline = true;
    }
    catch
    {
        alreadyOnline = false;
    }

    // 2. Open captive portal anyway (safe)
    await page.goto('https://172.22.2.6/connect/PortalMain', {
        waitUntil: 'domcontentloaded'
    });

    // If already online and portal doesn't show login form, exit silently
    const loginForm = await page.$('#LoginUserPassword_auth_username');
    if (!loginForm && alreadyOnline)
    {
        await browser.close();
        return;
    }

    // 3. Wait for login form
    await page.waitForSelector('#LoginUserPassword_auth_username');

    // 4. Fill credentials
    await page.fill('#LoginUserPassword_auth_username', username);
    await page.fill('#LoginUserPassword_auth_password', password);

    // 5. Submit using portal JS
    await page.evaluate(() =>
    {
        oAuthentication.submitActiveForm();
    });

    // 6. Detect success vs failure
    let loginSucceeded = false;

    try
    {
        await Promise.race([
            page.waitForNavigation({ timeout: 8000 }),
            page.waitForSelector('#LoginUserPassword_auth_form', {
                state: 'detached',
                timeout: 8000
            })
        ]);
        loginSucceeded = true;
    }
    catch
    {
        loginSucceeded = false;
    }

    if (loginSucceeded)
    {
        logLine(`SSID = ${ssid} LOGIN_SUCCESS`);
	fs.existsSync('C:\\wifi\\last-failure.txt') &&
        fs.unlinkSync('C:\\wifi\\last-failure.txt');


        execFile(
            'powershell.exe',
            [
                '-ExecutionPolicy', 'Bypass',
                '-File', 'C:\\wifi\\toast.ps1',
                '-Title', 'College Wi-Fi Connected',
                '-Message', 'You have been signed in successfully.'
            ]
        );
    }
    else
    {
        logLine(`SSID = ${ssid} LOGIN_FAILED`);
	fs.writeFileSync('C:\\wifi\\last-failure.txt', new Date().toISOString());
    }

    await browser.close();
})();
