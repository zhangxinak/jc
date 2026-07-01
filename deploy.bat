@echo off
setlocal
chcp 65001 >nul

set "WIN7_TARGET=x86_64-win7-windows-msvc"
set "SOURCE_EXE=src-tauri\target\%WIN7_TARGET%\release\machine-code-tool.exe"
set "DEST_DIR=..\jc-base\jc-base-view\src\main\resources\static\downloads"

echo Deploying machine-code-tool to Java project...
echo Source: %SOURCE_EXE%
echo.

if not exist "%SOURCE_EXE%" (
    echo Error: Windows 7 compatible executable was not found.
    echo Run build.bat first.
    pause
    exit /b 1
)

if not exist "%DEST_DIR%" mkdir "%DEST_DIR%"

copy /Y "%SOURCE_EXE%" "%DEST_DIR%\machine-code-tool-windows.exe" >nul
if errorlevel 1 (
    echo Error: failed to copy Windows executable.
    pause
    exit /b 1
)

if not exist "%DEST_DIR%\machine-code-tool-macos.dmg" echo. > "%DEST_DIR%\machine-code-tool-macos.dmg"
if not exist "%DEST_DIR%\machine-code-tool-kylin.deb" echo. > "%DEST_DIR%\machine-code-tool-kylin.deb"

echo # Machine Code Tool downloads> "%DEST_DIR%\README.md"
echo.>> "%DEST_DIR%\README.md"
echo Windows executable has been built from the Windows 7 compatible Rust target.>> "%DEST_DIR%\README.md"
echo.>> "%DEST_DIR%\README.md"
echo - machine-code-tool-windows.exe>> "%DEST_DIR%\README.md"
echo - machine-code-tool-macos.dmg placeholder>> "%DEST_DIR%\README.md"
echo - machine-code-tool-kylin.deb placeholder>> "%DEST_DIR%\README.md"

echo.
echo Deploy succeeded.
echo - %DEST_DIR%\machine-code-tool-windows.exe
pause
