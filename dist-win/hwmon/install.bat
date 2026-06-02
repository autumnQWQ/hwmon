@echo off
setlocal enabledelayedexpansion
title hwmon v1.0 Installer
chcp 65001 >/dev/null 2>&1

echo.
echo ==========================================
echo   hwmon v1.0 — Hardware Monitor
echo ==========================================
echo.

set "INSTALL_DIR=%USERPROFILE%\hwmon"

echo [INFO] Installing to: %INSTALL_DIR%
if not exist "%INSTALL_DIR%" mkdir "%INSTALL_DIR%"

REM ---- Copy binary ----
copy /Y "%~dp0hwmon.exe" "%INSTALL_DIR%\hwmon.exe" >/dev/null 2>&1
if errorlevel 1 ( echo [ERROR] Copy failed & pause & exit /b 1 )
echo [ OK ] hwmon.exe copied

REM ---- Copy Electron app ----
if exist "%~dp0hwmon-electron" (
    xcopy /E /I /Y "%~dp0hwmon-electron" "%INSTALL_DIR%\hwmon-electron\" >/dev/null 2>&1
    echo [ OK ] Electron app copied
    
    REM ---- Install Electron dependencies ----
    echo [INFO] Installing Electron dependencies (npm install)...
    cd "%INSTALL_DIR%\hwmon-electron"
    call npm install --production >/dev/null 2>&1
    if errorlevel 1 (
        echo [WARN] npm install failed. Run manually:
        echo   cd %INSTALL_DIR%\hwmon-electron ^&^& npm install
    ) else (
        echo [ OK ] Electron dependencies installed
    )
) else (
    echo [WARN] hwmon-electron/ not found, skipping
)

REM ---- Desktop shortcut ----
set "DESKTOP=%USERPROFILE%\Desktop"
if not exist "%DESKTOP%" set "DESKTOP=%USERPROFILE%\桌面"
if exist "%DESKTOP%" (
    set "PS_CMD=$s=(New-Object -ComObject WScript.Shell).CreateShortcut('%DESKTOP%\hwmon.lnk');$s.TargetPath='%INSTALL_DIR%\hwmon.exe';$s.Arguments='--gui';$s.WorkingDirectory='%INSTALL_DIR%';$s.Description='hwmon - Hardware Monitor';$s.Save()"
    powershell -NoProfile -ExecutionPolicy Bypass -Command "!PS_CMD!" >/dev/null 2>&1 && echo [ OK ] Desktop shortcut created
)

echo.
echo ==========================================
echo   Installation complete!
echo ==========================================
echo.
echo   Run:  %%INSTALL_DIR%%\hwmon.exe --gui
echo.
pause
