# Sign wrapper called by Tauri's bundle.windows.signCommand.
# Tauri invokes this once for every signable artifact (the .exe, the NSIS
# installer, the MSI). We gate on $env:AMBIT_SIGN so CI (no token, no cert)
# builds unsigned, and local builds with the token plugged in produce signed
# installers automatically. See SIGNING.md.

param(
    [Parameter(Mandatory = $true)]
    [string]$Target
)

$ErrorActionPreference = "Stop"

if ($env:AMBIT_SIGN -ne "1") {
    Write-Host "[sign.ps1] AMBIT_SIGN!=1 -- skipping signtool for $Target"
    exit 0
}

$thumbprint   = "583F11C19B8F6C0BC04CB304BE537EA525E59EF8"
$timestampUrl = "http://timestamp.sectigo.com"

$signtool = "${env:ProgramFiles(x86)}\Windows Kits\10\bin\10.0.26100.0\x64\signtool.exe"
if (-not (Test-Path $signtool)) {
    $fallback = Get-ChildItem "${env:ProgramFiles(x86)}\Windows Kits\10\bin\10.*.*.*\x64\signtool.exe" -ErrorAction SilentlyContinue |
        Sort-Object FullName -Descending |
        Select-Object -First 1
    if ($fallback) {
        $signtool = $fallback.FullName
    } else {
        Write-Error "[sign.ps1] signtool.exe not found. Install the Windows 10/11 SDK or edit sign.ps1 to point at signtool."
        exit 1
    }
}

Write-Host "[sign.ps1] Signing $Target"
Write-Host "[sign.ps1]   cert thumbprint: $thumbprint"
Write-Host "[sign.ps1]   timestamp:       $timestampUrl"
Write-Host "[sign.ps1]   signtool:        $signtool"

& $signtool sign /sha1 $thumbprint /fd SHA256 /tr $timestampUrl /td SHA256 /v "$Target"
$rc = $LASTEXITCODE

if ($rc -ne 0) {
    Write-Error "[sign.ps1] signtool failed with exit code $rc on $Target"
    exit $rc
}

Write-Host "[sign.ps1] OK: $Target"
exit 0
