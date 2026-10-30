# Creates a Startup-folder shortcut that launches league_auto_mute.exe minimised to the tray at login.
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
$shortcut.Arguments = "--minimized"
$shortcut.WorkingDirectory = Split-Path $exe
$shortcut.Description = "Auto-types /mute enemy or /mute all once per League match"
$shortcut.Save()

Write-Host "Installed: $lnk"
Write-Host "It will start in the tray at next login. Starting it now..."
Start-Process -FilePath $exe -ArgumentList "--minimized" -WorkingDirectory (Split-Path $exe)
