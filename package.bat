@echo off
setlocal
title hwmon Package Script
chcp 65001 >nul 2>&1

cd /d "%~dp0"

set "DIST_DIR=%~dp0dist-win\hwmon"
if exist "%DIST_DIR%" rmdir /s /q "%DIST_DIR%"
mkdir "%DIST_DIR%"
mkdir "%DIST_DIR%\hwmon-electron"
mkdir "%DIST_DIR%\hwmon-electron\electron"

echo [1/5] Copying hwmon.exe...
copy /Y "%~dp0target\release\hwmon.exe" "%DIST_DIR%\hwmon.exe" >nul

echo [2/5] Copying Electron runtime...
copy /Y "%~dp0hwmon-electron\node_modules\electron\dist\electron.exe" "%DIST_DIR%\hwmon-electron\electron\" >nul
set "EDIST=%~dp0hwmon-electron\node_modules\electron\dist"
for %%F in (
    chrome_100_percent.pak chrome_200_percent.pak
    d3dcompiler_47.dll dxcompiler.dll dxil.dll
    ffmpeg.dll icudtl.dat libEGL.dll libGLESv2.dll
    resources.pak snapshot_blob.bin v8_context_snapshot.bin
    vk_swiftshader.dll vk_swiftshader_icd.json
    LICENSES.chromium.html LICENSE version
) do (
    if exist "%EDIST%\%%F" (
        copy /Y "%EDIST%\%%F" "%DIST_DIR%\hwmon-electron\electron\" >nul
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

echo [3/5] Copying Electron app files...
copy /Y "%~dp0hwmon-electron\main.js" "%DIST_DIR%\hwmon-electron\main.js" >nul
copy /Y "%~dp0hwmon-electron\index.html" "%DIST_DIR%\hwmon-electron\index.html" >nul
copy /Y "%~dp0hwmon-electron\package.json" "%DIST_DIR%\hwmon-electron\package.json" >nul

echo [4/5] Copying source code...
mkdir "%DIST_DIR%\src"
xcopy /E /Y /Q "%~dp0src\*.rs" "%DIST_DIR%\src\" >nul
copy /Y "%~dp0Cargo.toml" "%DIST_DIR%\Cargo.toml" >nul
copy /Y "%~dp0hwmon.ico" "%DIST_DIR%\hwmon.ico" >nul
copy /Y "%~dp0install.bat" "%DIST_DIR%\install.bat" >nul
copy /Y "%~dp0README.md" "%DIST_DIR%\README.md" >nul
copy /Y "%~dp0.gitignore" "%DIST_DIR%\.gitignore" >nul

echo [5/5] Creating zip package...
for /f "tokens=2 delims== " %%V in ('findstr /b "version" "%~dp0Cargo.toml"') do set "VERSION=%%V"
set "VERSION=%VERSION:"=%"
powershell -NoProfile -Command "Compress-Archive -Path '%DIST_DIR%\*' -DestinationPath '%~dp0hwmon-v%VERSION%-win64.zip' -Force"

echo.
echo ==========================================
echo   Package complete!
echo ==========================================
dir "%~dp0hwmon-v%VERSION%-win64.zip"
echo.
pause
