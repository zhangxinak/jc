@echo off
setlocal

set "WIN7_TARGET=x86_64-win7-windows-msvc"
set "EXE_NAME=machine-code-tool.exe"
set "TARGET_EXE=src-tauri\target\%WIN7_TARGET%\release\%EXE_NAME%"
set "RELEASE_EXE=release\machine-code-tool-windows.exe"

echo Building machine-code-tool for Windows 7 compatibility...
echo Target: %WIN7_TARGET%
echo.

where cargo >nul 2>&1
if errorlevel 1 (
    echo Error: cargo was not found. Install Rust from https://rustup.rs/
    pause
    exit /b 1
)

where rustup >nul 2>&1
if errorlevel 1 (
    echo Error: rustup was not found. The Windows 7 target requires nightly + rust-src.
    pause
    exit /b 1
)

echo Installing/updating nightly toolchain with rust-src...
rustup toolchain install nightly --component rust-src
if errorlevel 1 (
    echo Error: failed to install nightly toolchain or rust-src.
    pause
    exit /b 1
)

set "RUSTUP_TOOLCHAIN=nightly"

echo Checking Tauri CLI...
cargo tauri --version >nul 2>&1
if errorlevel 1 (
    echo Installing Tauri CLI v1...
    cargo install --git https://github.com/tauri-apps/tauri --branch 1.x tauri-cli
    if errorlevel 1 (
        echo Error: failed to install Tauri CLI v1.
        pause
        exit /b 1
    )
)

echo.
echo Building release executable...
pushd src-tauri
cargo tauri build --target %WIN7_TARGET% --bundles none --verbose -- -Z build-std=std,panic_abort
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
echo This build uses the Rust Windows 7 target to avoid importing ProcessPrng from bcryptprimitives.dll.
pause
