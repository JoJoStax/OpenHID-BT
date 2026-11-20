param(
    [string]$Version = "0.9.0"
)

$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$InstallDir = Join-Path $env:LOCALAPPDATA "OpenHID-BT\drivers\bth-a2dp"

New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null

# Stage INF and any bundled open-source codec DLLs (ldacBT_enc.dll, freeaptx.dll)
Copy-Item -Path (Join-Path $ScriptDir "*") -Destination $InstallDir -Recurse -Force

$InfPath = Join-Path $InstallDir "OpenHidBthA2dp.inf"
if (Test-Path (Join-Path $InstallDir "OpenHidBthA2dp.cat")) {
    pnputil /add-driver $InfPath /install | Out-Host
} else {
    Write-Warning "Signed catalog (.cat) not present; staging codec libraries & driver metadata only."
}

$Version | Out-File -FilePath (Join-Path $InstallDir "VERSION") -Encoding ascii -NoNewline
Write-Host "OpenHID/BT Bluetooth A2DP driver v$Version staged at $InstallDir"
