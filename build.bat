@echo off
setlocal
title hwmon Build Script
chcp 65001 >nul 2>&1

echo.
echo ==========================================
echo   hwmon - Windows Build Script
echo ==========================================
echo.

REM ---- Check Rust ----
where rustc >nul 2>&1
if %errorlevel% neq 0 (
    echo [STEP 1] Installing Rust...
    echo.
    echo   Downloading rustup-init.exe...
    powershell -NoProfile -ExecutionPolicy Bypass -Command ^
        "Invoke-WebRequest -Uri 'https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe' -OutFile '%TEMP%\rustup-init.exe'" 2>&1
    if errorlevel 1 (
        echo [ERROR] Cannot download Rust installer - check internet connection
        echo        Manual install: https://rustup.rs
        echo.
echo Closing in 3 seconds...
timeout /t 3 /nobreak >nul
exit /b
        exit /b 1
    )
    echo   Running installer...
    "%TEMP%\rustup-init.exe" -y --default-toolchain stable
    if errorlevel 1 (
        echo [ERROR] Rust installation failed
        echo.
echo Closing in 3 seconds...
timeout /t 3 /nobreak >nul
exit /b
        exit /b 1
    )
    set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
    echo [ OK ] Rust installed
) else (
    echo [ OK ] Rust found:
    rustc --version
)

REM ---- Build ----
echo.
echo [STEP 2] Building hwmon.exe...

cd /d "%~dp0"
cargo build --release 2>&1
if %errorlevel% neq 0 (
    echo.
    echo [ERROR] Build failed - check the errors above
    echo.
echo Closing in 3 seconds...
timeout /t 3 /nobreak >nul
exit /b
    exit /b 1
)

REM ---- Output ----
set "OUTPUT=%~dp0target\release\hwmon.exe"
if exist "%OUTPUT%" (
    echo.
    echo ==========================================
    echo   Build successful!
    echo ==========================================
    echo.
    echo   Binary: %OUTPUT%
    for %%A in ("%OUTPUT%") do echo   Size:   %%~zA bytes
    echo.
    echo   To install, run: install.bat
    echo   Or just run:     "%OUTPUT%"
    echo.
    copy /Y "%OUTPUT%" "%~dp0hwmon.exe" >nul
    echo [ OK ] Copied hwmon.exe to current folder for install.bat
    echo.
) else (
    echo [ERROR] Build output not found
)

echo.
echo Closing in 3 seconds...
timeout /t 3 /nobreak >nul
exit /b
