@echo off
setlocal

set "WINDOWS_TARGET=x86_64-pc-windows-msvc"
set "RUST_TOOLCHAIN=1.77.2"
set "EXE_NAME=machine-code-tool.exe"
set "TARGET_EXE=src-tauri\target\%WINDOWS_TARGET%\release\%EXE_NAME%"
set "RELEASE_EXE=release\machine-code-tool-windows.exe"

echo Building machine-code-tool for Windows 7 compatibility...
echo Rust toolchain: %RUST_TOOLCHAIN%
echo Target: %WINDOWS_TARGET%
echo.

where cargo >nul 2>&1
if errorlevel 1 (
    echo Error: cargo was not found. Install Rust from https://rustup.rs/
    pause
    exit /b 1
)

where rustup >nul 2>&1
if errorlevel 1 (
    echo Error: rustup was not found.
    pause
    exit /b 1
)

echo Installing Rust %RUST_TOOLCHAIN% and target...
rustup toolchain install %RUST_TOOLCHAIN%
if errorlevel 1 (
    echo Error: failed to install Rust %RUST_TOOLCHAIN%.
    pause
    exit /b 1
)

rustup target add %WINDOWS_TARGET% --toolchain %RUST_TOOLCHAIN%
if errorlevel 1 (
    echo Error: failed to install target %WINDOWS_TARGET%.
    pause
    exit /b 1
)

set "RUSTUP_TOOLCHAIN=%RUST_TOOLCHAIN%"

echo.
echo Building release executable...
pushd src-tauri
cargo build --release --target %WINDOWS_TARGET%
set "BUILD_EXIT=%ERRORLEVEL%"
popd

if not "%BUILD_EXIT%"=="0" (
    echo Error: build failed.
    pause
    exit /b %BUILD_EXIT%
)

if not exist "%TARGET_EXE%" (
    echo Error: expected executable was not found:
    echo %TARGET_EXE%
    pause
    exit /b 1
)

if not exist "release" mkdir release
copy /Y "%TARGET_EXE%" "%RELEASE_EXE%" >nul
if errorlevel 1 (
    echo Error: failed to copy release executable.
    pause
    exit /b 1
)

echo.
echo Build succeeded.
echo Release file: %RELEASE_EXE%
echo.
echo This build uses Rust %RUST_TOOLCHAIN% and the legacy getrandom backend for Windows 7 compatibility.
pause
