$ErrorActionPreference = "Stop"
$InstallDir = Join-Path $env:LOCALAPPDATA "OpenHID-BT\drivers\bth-a2dp"

# Remove any OpenHidBthA2dp OEM INF packages so Windows falls back to stock BthA2dp.sys
$pnpList = pnputil /enum-drivers | Out-String
if ($pnpList -match "(oem\d+\.inf)\s+.*?OpenHidBthA2dp\.inf") {
    $oemInf = $Matches[1]
    pnputil /delete-driver $oemInf /uninstall /force | Out-Host
}

if (Test-Path $InstallDir) {
    Remove-Item -Path $InstallDir -Recurse -Force
}

Write-Host "Restored default Windows Bluetooth A2DP driver."
