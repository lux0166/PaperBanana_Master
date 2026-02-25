@echo off
REM PaperBanana - Windows Build Script (CMD)
REM This script builds the Tauri v2 app for Windows

REM Navigate to project root (parent of scripts/)
cd /d "%~dp0.."
echo === PaperBanana Windows Build ===
echo Project root: %cd%

echo.
echo [1/4] Checking prerequisites...

rustc --version >nul 2>&1
if %errorlevel% neq 0 (
    echo ERROR: Rust is not installed. Install from https://rustup.rs
    exit /b 1
)
echo   Rust: OK

python --version >nul 2>&1
if %errorlevel% neq 0 (
    echo ERROR: Python is not installed. Install Python 3.12+
    exit /b 1
)
echo   Python: OK

echo.
echo [2/4] Installing Python dependencies...
pip install -r requirements.txt

echo.
echo [3/4] Ensuring Tauri CLI is installed...
cargo tauri --version >nul 2>&1
if %errorlevel% neq 0 (
    echo Installing Tauri CLI v2...
    cargo install tauri-cli --version "^2.0" --locked
)

echo.
echo [4/4] Building Tauri application...
cargo tauri build

echo.
echo Build complete! Check src-tauri\target\release\bundle\ for installers.
pause
