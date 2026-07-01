@echo off
chcp 65001 >nul
echo 部署GUI版本机器码获取工具...

echo.
echo 检查GUI版本可执行文件...
if not exist "src-tauri\target\release\machine-code-tool.exe" (
    echo 错误: GUI版本可执行文件不存在
    echo 请先运行构建命令
    pause
    exit /b 1
)

echo GUI版本可执行文件检查通过

echo.
echo 创建下载目录...
if not exist "..\jc-base\jc-base-view\src\main\resources\static\downloads" (
    mkdir "..\jc-base\jc-base-view\src\main\resources\static\downloads"
)

echo.
echo 部署GUI版本到下载目录...
copy "src-tauri\target\release\machine-code-tool.exe" "..\jc-base\jc-base-view\src\main\resources\static\downloads\machine-code-tool-gui-windows.exe" >nul
if errorlevel 1 (
    echo 复制失败！
    pause
    exit /b 1
)

echo.
echo 更新命令行版本（保持兼容性）...
copy "src-tauri\target\release\machine-code-tool.exe" "..\jc-base\jc-base-view\src\main\resources\static\downloads\machine-code-tool-windows.exe" >nul

echo.
echo ========================================
echo GUI版本部署完成！
echo ========================================
echo.
echo 已部署文件：
echo - GUI版本: jc-base\jc-base-view\src\main\resources\static\downloads\machine-code-tool-gui-windows.exe
echo - 兼容版本: jc-base\jc-base-view\src\main\resources\static\downloads\machine-code-tool-windows.exe
echo.
echo 功能特点：
echo ✅ 图形用户界面（如您的截图所示）
echo ✅ HTTP API服务（端口18888）
echo ✅ 机器码获取（MAC、主板、CPU、硬盘）
echo ✅ 授权状态管理
echo ✅ 用户协议和隐私政策链接
echo.
echo 下载链接：
echo - GUI版本: http://localhost:8080/base/machine-code/download/gui-windows
echo - 标准版本: http://localhost:8080/base/machine-code/download/windows
echo.
echo 注意: GUI版本包含您截图中显示的所有界面功能！

pause