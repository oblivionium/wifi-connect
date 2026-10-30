param (
    [string]$Title,
    [string]$Message
)

Import-Module BurntToast -ErrorAction Stop

$ssidLine = netsh wlan show interfaces | Select-String "^\s*SSID\s*:"

if ($ssidLine)
{
    $ssid = ($ssidLine -split ":")[1].Trim()
}
else
{
    $ssid = "Unknown Wi-Fi"
}

$timestamp = Get-Date -Format "dd MMM yyyy, HH:mm:ss"

New-BurntToastNotification `
    -Text $Title, $Message, "Wi-Fi: $ssid | $timestamp" `
    -AppLogo "C:\Windows\System32\shell32.dll"
