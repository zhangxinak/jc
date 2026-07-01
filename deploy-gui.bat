@echo off
setlocal
chcp 65001 >nul

set "WIN7_TARGET=x86_64-win7-windows-msvc"
set "SOURCE_EXE=src-tauri\target\%WIN7_TARGET%\release\machine-code-tool.exe"
set "DEST_DIR=..\jc-base\jc-base-view\src\main\resources\static\downloads"

echo Deploying GUI machine-code-tool...
echo Source: %SOURCE_EXE%
echo.

if not exist "%SOURCE_EXE%" (
    echo Error: Windows 7 compatible GUI executable was not found.
    echo Run build.bat first.
    pause
    exit /b 1
)

if not exist "%DEST_DIR%" mkdir "%DEST_DIR%"

copy /Y "%SOURCE_EXE%" "%DEST_DIR%\machine-code-tool-gui-windows.exe" >nul
if errorlevel 1 (
    echo Error: failed to copy GUI executable.
    pause
    exit /b 1
)

copy /Y "%SOURCE_EXE%" "%DEST_DIR%\machine-code-tool-windows.exe" >nul
if errorlevel 1 (
    echo Error: failed to copy compatibility executable.
    pause
    exit /b 1
)

echo.
echo Deploy succeeded.
echo - %DEST_DIR%\machine-code-tool-gui-windows.exe
echo - %DEST_DIR%\machine-code-tool-windows.exe
pause
