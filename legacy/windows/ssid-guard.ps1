$allowedSSIDs = @(
    "BH-4",
    "BH4-T2",
    "LNMIIT",
    "LH"
)

$logFile = "C:\wifi\wifi-login.log"
$failureFile = "C:\wifi\last-failure.txt"
$cooldownMinutes = 5

$ssidLine = netsh wlan show interfaces | Select-String "^\s*SSID\s*:"

if (-not $ssidLine)
{
    exit
}

$ssid = ($ssidLine -split ":")[1].Trim()

if ($allowedSSIDs -notcontains $ssid)
{
    exit
}

# Backoff check
if (Test-Path $failureFile)
{
    $lastFail = Get-Date (Get-Content $failureFile)
    if ((Get-Date) - $lastFail -lt (New-TimeSpan -Minutes $cooldownMinutes))
    {
        exit
    }
}

$timestamp = Get-Date -Format "dd MMM yyyy, HH:mm:ss"
Add-Content -Path $logFile -Value "[$timestamp] SSID = $ssid LOGIN_ATTEMPTED"

node "C:\wifi\wifi-login.js" "$ssid"
