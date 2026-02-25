# PaperBanana - Windows Build Script (PowerShell)
# This script builds the Tauri v2 app for Windows
# Prerequisites: Rust, Node.js, Python 3.12+, Tauri CLI v2

param(
    [switch]$Debug,
    [switch]$SkipDeps
)

$ErrorActionPreference = "Stop"

# Navigate to project root (parent of scripts/)
$ProjectRoot = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Push-Location $ProjectRoot
Write-Host "=== PaperBanana Windows Build ===" -ForegroundColor Yellow
Write-Host "Project root: $ProjectRoot" -ForegroundColor Gray

# Check prerequisites
Write-Host "`n[1/5] Checking prerequisites..." -ForegroundColor Cyan

# Check Rust
try {
    $rustVersion = rustc --version
    Write-Host "  Rust: $rustVersion" -ForegroundColor Green
} catch {
    Write-Host "  ERROR: Rust is not installed. Install from https://rustup.rs" -ForegroundColor Red
    exit 1
}

# Check Node.js
try {
    $nodeVersion = node --version
    Write-Host "  Node.js: $nodeVersion" -ForegroundColor Green
} catch {
    Write-Host "  ERROR: Node.js is not installed." -ForegroundColor Red
    exit 1
}

# Check Python
try {
    $pythonVersion = python --version
    Write-Host "  Python: $pythonVersion" -ForegroundColor Green
} catch {
    Write-Host "  ERROR: Python is not installed. Install Python 3.12+" -ForegroundColor Red
    exit 1
}

# Install Tauri CLI if not present
Write-Host "`n[2/5] Checking Tauri CLI..." -ForegroundColor Cyan
try {
    cargo tauri --version 2>$null
    Write-Host "  Tauri CLI is installed" -ForegroundColor Green
} catch {
    Write-Host "  Installing Tauri CLI v2..." -ForegroundColor Yellow
    cargo install tauri-cli --version "^2.0" --locked
}

# Install Python dependencies
if (-not $SkipDeps) {
    Write-Host "`n[3/5] Installing Python dependencies..." -ForegroundColor Cyan
    pip install -r requirements.txt
    Write-Host "  Python dependencies installed" -ForegroundColor Green
} else {
    Write-Host "`n[3/5] Skipping dependency installation" -ForegroundColor Yellow
}

# Build the app
Write-Host "`n[4/5] Building Tauri application..." -ForegroundColor Cyan

if ($Debug) {
    Write-Host "  Building in DEBUG mode..." -ForegroundColor Yellow
    cargo tauri build --debug
} else {
    Write-Host "  Building in RELEASE mode..." -ForegroundColor Yellow
    cargo tauri build
}

# Report output
Write-Host "`n[5/5] Build complete!" -ForegroundColor Green
$bundleDir = "src-tauri\target\release\bundle"
if ($Debug) {
    $bundleDir = "src-tauri\target\debug\bundle"
}

Write-Host "`nOutput files:" -ForegroundColor Cyan
if (Test-Path "$bundleDir\nsis") {
    Write-Host "  NSIS Installer: $bundleDir\nsis\" -ForegroundColor Green
    Get-ChildItem "$bundleDir\nsis\*.exe" | ForEach-Object { Write-Host "    $($_.Name) ($([math]::Round($_.Length/1MB, 1)) MB)" }
}
if (Test-Path "$bundleDir\msi") {
    Write-Host "  MSI Installer: $bundleDir\msi\" -ForegroundColor Green
    Get-ChildItem "$bundleDir\msi\*.msi" | ForEach-Object { Write-Host "    $($_.Name) ($([math]::Round($_.Length/1MB, 1)) MB)" }
}

Write-Host "`nDone!" -ForegroundColor Yellow
Pop-Location
