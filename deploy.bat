@echo off
chcp 65001 >nul
echo 部署机器码获取工具到Java项目...

echo.
echo 检查可执行文件...
if not exist "src-tauri\target\release\machine-code-tool.exe" (
    echo 错误: 可执行文件不存在
    echo 请先运行 build.bat 构建项目
    pause
    exit /b 1
)

echo 可执行文件检查通过

echo.
echo 创建下载目录...
if not exist "..\jc-base\jc-base-view\src\main\resources\static\downloads" (
    mkdir "..\jc-base\jc-base-view\src\main\resources\static\downloads"
)

echo.
echo 复制Windows版本...
copy "src-tauri\target\release\machine-code-tool.exe" "..\jc-base\jc-base-view\src\main\resources\static\downloads\machine-code-tool-windows.exe" >nul
if errorlevel 1 (
    echo 复制失败！
    pause
    exit /b 1
)

echo.
echo 创建占位文件（macOS和Linux版本）...
echo. > "..\jc-base\jc-base-view\src\main\resources\static\downloads\machine-code-tool-macos.dmg"
echo. > "..\jc-base\jc-base-view\src\main\resources\static\downloads\machine-code-tool-kylin.deb"

echo.
echo 更新README...
(
echo # 机器码工具下载文件
echo.
echo 请将构建好的机器码工具可执行文件放置在此目录：
echo.
echo ## Windows
echo - machine-code-tool-windows.exe ^(已部署^)
echo.
echo ## macOS  
echo - machine-code-tool-macos.dmg ^(待构建^)
echo.
echo ## 银河麒麟
echo - machine-code-tool-kylin.deb ^(待构建^)
echo.
echo ## 构建说明
echo.
echo Windows版本已经构建完成，可以直接使用。
echo.
echo macOS和Linux版本需要在对应系统上运行以下命令：
echo ```bash
echo # 进入项目目录
echo cd machine-code-tool/src-tauri
echo.
echo # 构建
echo cargo tauri build --no-bundle
echo.
echo # 复制文件
echo # macOS: cp target/release/machine-code-tool ../downloads/machine-code-tool-macos
echo # Linux: cp target/release/machine-code-tool ../downloads/machine-code-tool-kylin
echo ```
) > "..\jc-base\jc-base-view\src\main\resources\static\downloads\README.md"

echo.
echo ========================================
echo 部署完成！
echo ========================================
echo.
echo 已部署文件：
echo - Windows版本: jc-base\jc-base-view\src\main\resources\static\downloads\machine-code-tool-windows.exe
echo - macOS占位符: jc-base\jc-base-view\src\main\resources\static\downloads\machine-code-tool-macos.dmg
echo - Linux占位符: jc-base\jc-base-view\src\main\resources\static\downloads\machine-code-tool-kylin.deb
echo.
echo 下载链接：
echo - Windows: http://localhost:8080/base/machine-code/download/windows
echo - macOS: http://localhost:8080/base/machine-code/download/macos  
echo - 银河麒麟: http://localhost:8080/base/machine-code/download/kylin
echo.
echo 注意: macOS和Linux版本需要在对应系统上重新编译

pause