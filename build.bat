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

echo Rust环境检查通过
echo.

echo 尝试直接构建Rust项目...
cd src-tauri
cargo build --release

if errorlevel 1 (
    echo 构建失败，尝试安装Tauri CLI...
    cargo install tauri-cli
    
    if errorlevel 1 (
        echo Tauri CLI安装失败
        cd ..
        pause
        exit /b 1
    )
    
    echo 使用Tauri CLI构建...
    cd ..
    cargo tauri build
    
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