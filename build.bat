@echo off
echo 构建机器码获取工具...
echo.

echo 检查Rust环境...
cargo --version >nul 2>&1
if errorlevel 1 (
    echo 错误: 未找到Rust环境
    echo 请访问 https://rustup.rs/ 安装Rust
    pause
    exit /b 1
)

echo 检查Rust版本（Windows 7 兼容需 1.77.2，1.78+ 会在 Win7 上报 ProcessPrng 错误）...
for /f "tokens=2" %%v in ('rustc --version 2^>nul') do set RUST_VER=%%v
echo 当前 Rust 版本: %RUST_VER%
echo %RUST_VER% | findstr /B "1.77.2" >nul
if errorlevel 1 (
    echo.
    echo 警告: 当前 Rust 版本不是 1.77.2，构建产物可能无法在 Windows 7 上运行。
    echo 请执行: rustup toolchain install 1.77.2 ^&^& rustup override set 1.77.2
    echo 或在项目根目录已配置 rust-toolchain.toml，进入本目录后 rustup 应自动切换。
    echo.
)

echo Rust环境检查通过
echo.

echo 尝试直接构建Rust项目...
cd src-tauri
rem 静态链接 MSVC 运行时，避免客户机缺少 VCRUNTIME140_1.dll
set RUSTFLAGS=-C target-feature=+crt-static
if exist "Cargo.lock" (
    cargo build --locked --release
) else (
    cargo build --release
)

if errorlevel 1 (
    echo 构建失败，尝试安装 Tauri CLI（需 Rust stable，不能用 1.77.2）...
    rustup toolchain install stable >nul 2>&1
    cargo +stable install tauri-cli --version 1.5.14 --locked
    if errorlevel 1 (
        echo Tauri CLI 安装失败
        cd ..
        pause
        exit /b 1
    )
    
    echo 使用 Tauri CLI 构建...
    cd ..
    cargo +1.77.2 tauri build
    
    if errorlevel 1 (
        echo 构建失败
        pause
        exit /b 1
    )
) else (
    cd ..
    echo 构建成功！
    echo 可执行文件位置: src-tauri\target\release\machine-code-tool.exe
)

echo.
echo 创建发布目录...
if not exist "release" mkdir release

echo 复制可执行文件...
if exist "src-tauri\target\release\machine-code-tool.exe" (
    copy "src-tauri\target\release\machine-code-tool.exe" "release\machine-code-tool-windows.exe"
    echo 发布文件已准备就绪: release\machine-code-tool-windows.exe
) else (
    echo 警告: 未找到可执行文件
    
    echo 查找Tauri构建产物...
    for /r "src-tauri\target\release\bundle\" %%f in (*.exe *.msi) do (
        copy "%%f" "release\machine-code-tool-windows.exe"
        echo 发布文件已准备就绪: release\machine-code-tool-windows.exe
        goto :found
    )
    
    :found
)
echo.
pause