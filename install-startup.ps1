# Creates a Startup-folder shortcut that launches league_auto_mute.exe (minimised) at login.
$ErrorActionPreference = "Stop"

# Packaged zip: exe sits next to this script. Source checkout: exe is under target\release.
$exe = Join-Path $PSScriptRoot "league_auto_mute.exe"
if (-not (Test-Path $exe)) {
    $exe = Join-Path $PSScriptRoot "target\release\league_auto_mute.exe"
}
if (-not (Test-Path $exe)) {
    Write-Error "league_auto_mute.exe not found next to this script (or build it with: cargo build --release)"
}

$startup = [Environment]::GetFolderPath("Startup")
$lnk = Join-Path $startup "League Auto Mute.lnk"

$shell = New-Object -ComObject WScript.Shell
$shortcut = $shell.CreateShortcut($lnk)
$shortcut.TargetPath = $exe
$shortcut.WorkingDirectory = Split-Path $exe
$shortcut.WindowStyle = 7   # minimised
$shortcut.Description = "Auto-types /mute enemy once per League match"
$shortcut.Save()

Write-Host "Installed: $lnk"
Write-Host "It will start minimised at next login. Starting it now..."
Start-Process -FilePath $exe -WorkingDirectory (Split-Path $exe) -WindowStyle Minimized
