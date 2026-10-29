<#
.SYNOPSIS
  Builds Horizon Offline Launcher from source on Windows and installs it (install.ps1).

.DESCRIPTION
  Run from the repository's folder:
    powershell -ExecutionPolicy Bypass -File build.ps1

  It installs Rust (rustup) when it is missing, finds fxc.exe (the shader compiler of the
  Windows SDK, which the launcher's UI needs) on any drive, builds the launcher and installs it
  with Start menu and desktop shortcuts. Visual Studio Build Tools with "Desktop development with
  C++" (the MSVC compiler and the Windows SDK) must be installed:
  https://visualstudio.microsoft.com/visual-cpp-build-tools/
#>
param(
    # Build only: don't install or start it.
    [switch]$NoInstall
)

$ErrorActionPreference = "Stop"
$Root = $PSScriptRoot
function Say($text) { Write-Host "  $text" }

# Rust
$cargo = Join-Path $env:USERPROFILE ".cargo\bin\cargo.exe"
if (-not (Test-Path $cargo)) {
    $found = Get-Command cargo -ErrorAction SilentlyContinue
    if ($found) { $cargo = $found.Source }
}
if (-not (Test-Path $cargo)) {
    Say "Installing Rust (rustup)"
    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
    $init = Join-Path $env:TEMP "rustup-init.exe"
    Invoke-WebRequest "https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe" -OutFile $init -UseBasicParsing
    & $init -y --default-toolchain stable --profile minimal
    if ($LASTEXITCODE -ne 0) { throw "rustup failed" }
    $cargo = Join-Path $env:USERPROFILE ".cargo\bin\cargo.exe"
}

# fxc.exe: the environment's, on PATH, or the newest x64 one in any Windows Kits folder.
if (-not ($env:GPUI_FXC_PATH -and (Test-Path $env:GPUI_FXC_PATH)) -and -not (Get-Command fxc.exe -ErrorAction SilentlyContinue)) {
    $kits = @("${env:ProgramFiles(x86)}\Windows Kits\10\bin", "$env:ProgramFiles\Windows Kits\10\bin")
    $kits += Get-PSDrive -PSProvider FileSystem | ForEach-Object { Join-Path $_.Root "Windows Kits\10\bin" }
    $fxc = $kits | Where-Object { Test-Path $_ } |
        ForEach-Object { Get-ChildItem $_ -Recurse -Filter fxc.exe -ErrorAction SilentlyContinue } |
        Where-Object { $_.Directory.Name -eq "x64" } |
        Sort-Object { $_.Directory.Parent.Name } -Descending |
        Select-Object -First 1
    if (-not $fxc) {
        throw "fxc.exe (Windows SDK) not found. Install Visual Studio Build Tools with 'Desktop development with C++': https://visualstudio.microsoft.com/visual-cpp-build-tools/"
    }
    $env:GPUI_FXC_PATH = $fxc.FullName
    Say "Shader compiler: $($fxc.FullName)"
}

Say "Building the launcher (the first build takes a few minutes)"
Push-Location (Join-Path $Root "launcher")
try {
    & $cargo build --release --locked
    if ($LASTEXITCODE -ne 0) { throw "The build failed (see above)" }
} finally {
    Pop-Location
}
$exe = Join-Path $Root "launcher\target\release\horizon-launcher.exe"
Say "Built $exe"

if (-not $NoInstall) {
    & (Join-Path $Root "install.ps1") -Exe $exe
}
