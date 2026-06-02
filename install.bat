@echo off
setlocal enabledelayedexpansion
title hwmon Installer
chcp 65001 >nul 2>&1

echo.
echo ==========================================
echo   hwmon - PC Hardware Monitor Installer
echo ==========================================
echo.

REM ---- Check admin ----
net session >nul 2>&1
if %errorlevel% neq 0 (
    echo [INFO] User install mode - no admin privileges
    set "INSTALL_DIR=%USERPROFILE%\hwmon"
) else (
    echo [INFO] Admin mode - system-wide install
    set "INSTALL_DIR=%ProgramFiles%\hwmon"
)

REM ---- Find hwmon.exe ----
set "SRC=%~dp0hwmon.exe"
if not exist "%SRC%" set "SRC=%~dp0..\hwmon.exe"
if not exist "%SRC%" set "SRC=hwmon.exe"

if not exist "%SRC%" (
    echo [ERROR] Cannot find hwmon.exe
    echo.
    echo   This package contains source code only - you need to build first.
    echo   Run build.bat to compile hwmon.exe automatically.
    echo   Or see README.md for manual build instructions.
    echo.
    if exist "%~dp0build.bat" (
        echo   build.bat found — starting build now...
        pause
        call "%~dp0build.bat"
        if exist "%~dp0hwmon.exe" set "SRC=%~dp0hwmon.exe"
        if not exist "%SRC%" exit /b 1
    ) else (
        pause
        exit /b 1
    )
)
echo [ OK ] Found: %SRC%
echo.

REM ---- Install ----
echo [INFO] Installing to: %INSTALL_DIR%
if not exist "%INSTALL_DIR%" mkdir "%INSTALL_DIR%"
if not exist "%INSTALL_DIR%" (
    echo [ERROR] Cannot create install directory
    echo        Try running as Administrator
    pause
    exit /b 1
)

copy /Y "%SRC%" "%INSTALL_DIR%\hwmon.exe" >nul 2>&1
if errorlevel 1 (
    echo [ERROR] Copy failed - check permissions
    pause
    exit /b 1
)
echo [ OK ] Binary copied
echo.

REM ---- Add to PATH safely ----
echo [INFO] Configuring PATH...
set "PATH_FILE=%INSTALL_DIR%\add-to-path.reg"
(
echo Windows Registry Editor Version 5.00
echo.
echo [HKEY_CURRENT_USER\Environment]
echo "Path"="%INSTALL_DIR%;%PATH%"
) > "%PATH_FILE%"
echo [ OK ] PATH registry file created: %PATH_FILE%
echo        To add to PATH immediately, double-click add-to-path.reg
echo        Or restart your terminal for the change to take effect.
echo.

REM ---- Create desktop shortcut ----
echo [INFO] Creating desktop shortcut...
set "DESKTOP=%USERPROFILE%\Desktop"
if not exist "%DESKTOP%" set "DESKTOP=%USERPROFILE%\桌面"
if not exist "%DESKTOP%" (
    echo [WARN] Desktop folder not found, skipping shortcut
    goto :skip_shortcut
)

set "SHORTCUT=%DESKTOP%\hwmon.lnk"
set "PS_CMD=$WshShell = New-Object -ComObject WScript.Shell; $Shortcut = $WshShell.CreateShortcut('%SHORTCUT%'); $Shortcut.TargetPath = '%INSTALL_DIR%\hwmon.exe'; $Shortcut.WorkingDirectory = '%INSTALL_DIR%'; $Shortcut.Description = 'hwmon - Hardware Monitor'; $Shortcut.Save()"

powershell -NoProfile -ExecutionPolicy Bypass -Command "%PS_CMD%" >nul 2>&1
if errorlevel 1 (
    echo [WARN] Shortcut creation failed - you can create it manually
) else (
    echo [ OK ] Desktop shortcut: hwmon.lnk
)

:skip_shortcut
echo.

REM ---- Done ----
echo ==========================================
echo   Installation complete!
echo ==========================================
echo.
echo   Run from terminal:
echo     %INSTALL_DIR%\hwmon.exe
echo     %INSTALL_DIR%\hwmon.exe --watch
echo     %INSTALL_DIR%\hwmon.exe --json
echo.
echo   Or double-click hwmon.exe in:
echo     %INSTALL_DIR%
echo.

echo Closing in 3 seconds...
timeout /t 3 /nobreak >nul
exit /b 0
