@echo off
setlocal enabledelayedexpansion

set "AGENT_NAME=claw-agent-client-rs"
set "BINARY_NAME=claw-agent-client-rs.exe"
set "SCRIPT_DIR=%~dp0"
set "BINARY_PATH=%SCRIPT_DIR%%BINARY_NAME%"
set "CONFIG_DIR=%SCRIPT_DIR%config"
set "CONFIG_PATH=%SCRIPT_DIR%config\agent.yml"

if "%~1"=="" goto interactive

goto %1

:install
call :check_admin
call :install_binary
call :install_config
call :create_service
call :enable_service
call :show_status
goto :eof

:start
call :check_admin
sc start %AGENT_NAME%
goto :eof

:stop
call :check_admin
sc stop %AGENT_NAME%
goto :eof

:restart
call :check_admin
sc stop %AGENT_NAME%
timeout /t 1 /nobreak >nul
sc start %AGENT_NAME%
goto :eof

:status
sc query %AGENT_NAME%
goto :eof

:uninstall
call :check_admin
echo Uninstalling service...
sc stop %AGENT_NAME% 2>nul
sc delete %AGENT_NAME% 2>nul
echo Service uninstalled
echo Installation directory preserved
goto :eof

:help
echo ===== Help Information =====
echo.
echo Service: %AGENT_NAME%
echo Binary: %BINARY_PATH%
echo Config: %CONFIG_PATH%
echo.
echo Commands:
echo   install   - Install service
echo   start     - Start service
echo   stop      - Stop service
echo   restart   - Restart service
echo   status    - Show service status
echo   uninstall - Uninstall service
echo   help      - Show this help
goto :eof

:check_admin
net session >nul 2>&1
if %ERRORLEVEL% neq 0 (
    echo Error: Please run as Administrator
    exit /b 1
)
goto :eof

:install_binary
echo [1/4] Checking binary...
if not exist "%BINARY_PATH%" (
    echo Error: Binary not found: %BINARY_PATH%
    echo Please run windows_build.bat first
    exit /b 1
)
echo Binary: %BINARY_PATH%
goto :eof

:install_config
echo [2/4] Checking config...
if not exist "%CONFIG_DIR%" mkdir "%CONFIG_DIR%"
if exist "%CONFIG_PATH%" (
    echo Config found: %CONFIG_PATH%
) else (
    echo Warning: Config not found: %CONFIG_PATH%
)
goto :eof

:create_service
echo [3/4] Creating Windows service...
sc query %AGENT_NAME% >nul 2>&1
if %ERRORLEVEL% equ 0 (
    echo Service exists, deleting...
    sc stop %AGENT_NAME% 2>nul
    sc delete %AGENT_NAME%
    timeout /t 2 /nobreak >nul
)
set "BIN_PATH=""%BINARY_PATH%"""
sc create %AGENT_NAME% binPath= %BIN_PATH% start= auto
sc description %AGENT_NAME% "OpenClaw Remote Agent"
echo Service created: %AGENT_NAME%
goto :eof

:enable_service
echo [4/4] Starting service...
sc start %AGENT_NAME%
goto :eof

:show_status
echo.
echo Service status:
sc query %AGENT_NAME%
goto :eof

:interactive
echo ========================================
echo   Claw Agent Client Rs Install Script
echo ========================================
echo.
echo Options:
echo   1) Install service
echo   2) Start service
echo   3) Stop service
echo   4) Restart service
echo   5) Service status
echo   6) Uninstall service
echo   7) Help
echo   0) Exit
echo.
set /p choice="Enter option [0-7]: "

if "%choice%"=="1" goto install
if "%choice%"=="2" goto start
if "%choice%"=="3" goto stop
if "%choice%"=="4" goto restart
if "%choice%"=="5" goto status
if "%choice%"=="6" goto uninstall
if "%choice%"=="7" goto help
if "%choice%"=="0" exit /b

echo Invalid option
timeout /t 1 >nul
goto interactive
