<#
.SYNOPSIS
  Installs Horizon Offline Launcher on Windows: the launcher, a Start menu shortcut and a desktop
  shortcut. No administrator rights needed.

.DESCRIPTION
  One line, in PowerShell:
    irm https://github.com/gerdaroot/Horizon-Offline-Launcher/releases/latest/download/install.ps1 | iex

  Or with a launcher you built yourself (build.ps1 does this for you):
    .\install.ps1 -Exe launcher\target\release\horizon-launcher.exe
#>
param(
    # A local horizon-launcher.exe to install instead of downloading the latest release.
    [string]$Exe = "",
    # Don't start the launcher when it is installed.
    [switch]$NoStart
)

$ErrorActionPreference = "Stop"
$Repo = "gerdaroot/Horizon-Offline-Launcher"
$Asset = "horizon-launcher-windows-x64.zip"
$Name = "Horizon Offline Launcher"
$Dir = Join-Path $env:LOCALAPPDATA "Programs\HorizonOfflineLauncher"
$Target = Join-Path $Dir "horizon-offline-launcher.exe"

function Say($text) { Write-Host "  $text" }

Write-Host ""
Write-Host "$Name" -ForegroundColor Magenta
Write-Host ""

# A running copy keeps its file locked.
Get-Process | Where-Object { $_.Path -eq $Target } | ForEach-Object {
    Say "Closing the running launcher"
    $_ | Stop-Process -Force
    Start-Sleep -Milliseconds 500
}

New-Item -ItemType Directory -Force -Path $Dir | Out-Null

if ($Exe) {
    if (-not (Test-Path $Exe)) { throw "No file at $Exe" }
    Say "Installing $Exe"
    Copy-Item $Exe $Target -Force
} else {
    $url = "https://github.com/$Repo/releases/latest/download/$Asset"
    Say "Downloading the latest release"
    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
    $work = Join-Path $env:TEMP "horizon-offline-launcher-install"
    Remove-Item $work -Recurse -Force -ErrorAction SilentlyContinue
    New-Item -ItemType Directory -Force -Path $work | Out-Null
    $zip = Join-Path $work $Asset
    Invoke-WebRequest -Uri $url -OutFile $zip -UseBasicParsing
    Expand-Archive $zip -DestinationPath $work -Force
    $new = Get-ChildItem $work -Recurse -Filter "horizon-launcher.exe" | Select-Object -First 1
    if (-not $new) { throw "The download has no horizon-launcher.exe" }
    Move-Item $new.FullName $Target -Force
    Remove-Item $work -Recurse -Force -ErrorAction SilentlyContinue
}
# Downloaded files carry a "from the internet" mark that makes Windows ask before each start.
Unblock-File $Target -ErrorAction SilentlyContinue

$shell = New-Object -ComObject WScript.Shell
$places = @(
    (Join-Path ([Environment]::GetFolderPath("Programs")) "$Name.lnk"),
    (Join-Path ([Environment]::GetFolderPath("Desktop")) "$Name.lnk")
)
foreach ($lnk in $places) {
    $s = $shell.CreateShortcut($lnk)
    $s.TargetPath = $Target
    $s.WorkingDirectory = $Dir
    $s.IconLocation = "$Target,0"
    $s.Description = "Play Horizon (Minecraft 26.3 in Rust) without a Microsoft account"
    $s.Save()
}
Say "Shortcuts: Start menu and desktop"
Say "Installed in $Dir"
Write-Host ""
Write-Host "  Next: open the launcher, press Prepare (it uses your Minecraft 26.3), then Play." -ForegroundColor Green
Write-Host ""

if (-not $NoStart) { Start-Process $Target }
