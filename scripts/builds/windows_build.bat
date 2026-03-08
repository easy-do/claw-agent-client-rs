@echo off
setlocal enabledelayedexpansion

echo ========================================
echo   Claw Agent Client Rs Build Script
echo ========================================
echo.

set "SCRIPT_DIR=%~dp0"
set "PROJECT_DIR=%SCRIPT_DIR%.."
set "COMPOSE_DIR=%PROJECT_DIR%\compose"

cd /d "%PROJECT_DIR%"

where rustc >nul 2>&1
if %ERRORLEVEL% neq 0 (
    echo [1/4] Installing Rust...
    powershell -Command "Invoke-WebRequest -Uri https://win.rustup.rs -OutFile rustup-init.exe"
    rustup-init.exe -y
    del rustup-init.exe
    call "%USERPROFILE%\.cargo\env"
) else (
    echo [1/4] Rust already installed
)

echo [2/4] Checking dependencies...

echo [3/4] Building project...
cargo build --release

echo [4/4] Copying files to compose...

if not exist "%COMPOSE_DIR%\config" mkdir "%COMPOSE_DIR%\config"
if not exist "%COMPOSE_DIR%\scripts" mkdir "%COMPOSE_DIR%\scripts"

if exist "target\release\claw-agent-client-rs.exe" (
    copy /Y "target\release\claw-agent-client-rs.exe" "%COMPOSE_DIR%\"
    echo Binary copied to: %COMPOSE_DIR%\claw-agent-client-rs.exe
)

if exist "config\agent.yml" (
    copy /Y "config\agent.yml" "%COMPOSE_DIR%\config\"
    echo Config copied to: %COMPOSE_DIR%\config\agent.yml
)

if exist "config\metadata.json" (
    copy /Y "config\metadata.json" "%COMPOSE_DIR%\config\"
    echo Metadata copied to: %COMPOSE_DIR%\config\metadata.json
)

if exist "%SCRIPT_DIR%..\installs\windows_install.bat" (
    copy /Y "%SCRIPT_DIR%..\installs\windows_install.bat" "%COMPOSE_DIR%\scripts\"
    echo Install script copied to: %COMPOSE_DIR%\scripts\windows_install.bat
)

echo.
echo ========================================
echo Build completed!
echo ========================================
echo.
echo Output location: %COMPOSE_DIR%
echo.
echo Next steps:
echo   1. Configure your agent.yml in config folder
echo   2. Run: cd %COMPOSE_DIR% ^&^& windows_install.bat install
echo.

pause
