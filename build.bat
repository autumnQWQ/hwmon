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

REM ---- Check Node.js ----
where node >nul 2>&1
if %errorlevel% neq 0 (
    echo [ERROR] Node.js is required for Electron UI
    echo        Download from: https://nodejs.org
    echo.
echo Closing in 3 seconds...
timeout /t 3 /nobreak >nul
exit /b
    exit /b 1
)
echo [ OK ] Node.js found:
node --version

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
echo [ OK ] Rust build complete

REM ---- Install Electron dependencies ----
echo.
echo [STEP 3] Installing Electron dependencies...
cd /d "%~dp0hwmon-electron"
if exist node_modules (
    echo [ OK ] node_modules already exists, skipping npm install
) else (
    call npm install 2>&1
    if %errorlevel% neq 0 (
        echo [ERROR] npm install failed
        echo.
echo Closing in 3 seconds...
timeout /t 3 /nobreak >nul
exit /b
        exit /b 1
    )
    echo [ OK ] npm install complete
)

REM ---- Package distribution ----
echo.
echo [STEP 4] Packaging distribution...
cd /d "%~dp0"

set "DIST_DIR=%~dp0dist-win\hwmon"
if exist "%DIST_DIR%" rmdir /s /q "%DIST_DIR%"
mkdir "%DIST_DIR%"
mkdir "%DIST_DIR%\hwmon-electron"
mkdir "%DIST_DIR%\hwmon-electron\electron"

REM --- Copy hwmon.exe ---
copy /Y "%~dp0target\release\hwmon.exe" "%DIST_DIR%\hwmon.exe" >nul
echo [ OK ] hwmon.exe

REM --- Copy Electron runtime ---
copy /Y "%~dp0hwmon-electron\node_modules\electron\dist\electron.exe" "%DIST_DIR%\hwmon-electron\electron\" >nul
for %%F in (
    chrome_100_percent.pak chrome_200_percent.pak
    d3dcompiler_47.dll dxcompiler.dll dxil.dll
    ffmpeg.dll icudtl.dat libEGL.dll libGLESv2.dll
    resources.pak snapshot_blob.bin v8_context_snapshot.bin
    vk_swiftshader.dll vk_swiftshader_icd.json
    LICENSES.chromium.html LICENSE version
) do (
    if exist "%~dp0hwmon-electron\node_modules\electron\dist\%%F" (
        copy /Y "%~dp0hwmon-electron\node_modules\electron\dist\%%F" "%DIST_DIR%\hwmon-electron\electron\" >nul
    )
)
if exist "%~dp0hwmon-electron\node_modules\electron\dist\locales" (
    mkdir "%DIST_DIR%\hwmon-electron\electron\locales" 2>nul
    xcopy /E /Y /Q "%~dp0hwmon-electron\node_modules\electron\dist\locales" "%DIST_DIR%\hwmon-electron\electron\locales\" >nul
)
if exist "%~dp0hwmon-electron\node_modules\electron\dist\resources" (
    mkdir "%DIST_DIR%\hwmon-electron\electron\resources" 2>nul
    xcopy /E /Y /Q "%~dp0hwmon-electron\node_modules\electron\dist\resources" "%DIST_DIR%\hwmon-electron\electron\resources\" >nul
)
echo [ OK ] Electron runtime

REM --- Copy Electron app files ---
copy /Y "%~dp0hwmon-electron\main.js" "%DIST_DIR%\hwmon-electron\main.js" >nul
copy /Y "%~dp0hwmon-electron\index.html" "%DIST_DIR%\hwmon-electron\index.html" >nul
copy /Y "%~dp0hwmon-electron\package.json" "%DIST_DIR%\hwmon-electron\package.json" >nul
echo [ OK ] Electron app files

REM --- Copy src ---
mkdir "%DIST_DIR%\src"
xcopy /E /Y /Q "%~dp0src\*.rs" "%DIST_DIR%\src\" >nul
echo [ OK ] Source code

REM --- Copy other files ---
copy /Y "%~dp0Cargo.toml" "%DIST_DIR%\Cargo.toml" >nul
copy /Y "%~dp0hwmon.ico" "%DIST_DIR%\hwmon.ico" >nul
copy /Y "%~dp0install.bat" "%DIST_DIR%\install.bat" >nul
copy /Y "%~dp0README.md" "%DIST_DIR%\README.md" >nul
copy /Y "%~dp0.gitignore" "%DIST_DIR%\.gitignore" >nul
echo [ OK ] Project files

REM ---- Output ----
echo.
echo ==========================================
echo   Build and package successful!
echo ==========================================
echo.
echo   Binary:    %DIST_DIR%\hwmon.exe
echo   Electron:  %DIST_DIR%\hwmon-electron\electron\electron.exe
echo.
echo   To distribute, zip the entire dist-win\hwmon folder:
echo     powershell Compress-Archive -Path "%DIST_DIR%\*" -DestinationPath "hwmon-v0.1.0-win64.zip" -Force
echo.
echo   Or just run:  "%DIST_DIR%\hwmon.exe"
echo.

echo Closing in 3 seconds...
timeout /t 3 /nobreak >nul
exit /b
