# 机器码获取工具

## 概述

跨平台硬件信息获取应用，用于配合广联达 SRM 系统投标时的设备身份验证。支持 Windows、macOS、银河麒麟（Linux）。

## 功能特性

- **跨平台**: Windows（32/64 位）、macOS（Intel + Apple Silicon）、银河麒麟 / Linux（AppImage）
- **硬件信息**: MAC 地址、主板序列号、CPU 序列号、硬盘序列号
- **本地 API**: HTTP 接口 `http://localhost:18888` 供业务系统调用
- **授权控制**: 用户可控制是否允许信息获取

## 系统要求

- **Windows**: 7 SP1 及以上，32/64 位
- **macOS**: 10.13 及以上
- **银河麒麟 / Linux**: 麒麟 V10 或 Ubuntu 20.04 等，glibc 2.31+，x86_64。AppImage 单文件免安装，通常无需额外装 WebKit。

## 获取安装包

通过 **GitHub Actions** 构建，不发布 Release，仅保留各平台构建产物（Artifacts）：

1. 打开本仓库 **Actions** → 选择 workflow「构建机器码获取工具」
2. **Run workflow**（或 push 到 main/master 自动触发）
3. 运行完成后进入该次 run → **Artifacts** 下载对应平台：
   - **machine-code-tool-kylin**：银河麒麟 / Linux（AppImage + 使用说明）
   - **machine-code-tool-macos**：macOS 通用版
   - **machine-code-tool-windows**：Windows 64 位
   - **machine-code-tool-windows-32**：Windows 32 位

本工程使用 **Tauri 1.x**，请勿用本地 Tauri 2.0 构建；推荐仅通过上述 Actions 获取安装包。

## 使用说明

- **Windows**: 双击 `.exe` 运行
- **macOS**: 双击通用二进制运行；若提示“来自身份不明的开发者”，右键 → 打开
- **银河麒麟 / Linux**: `chmod +x *.AppImage` 后双击或 `./machine-code-tool-kylin.AppImage`。若终端出现 AT-SPI 警告，可忽略或使用 `NO_AT_BRIDGE=1 ./machine-code-tool-kylin.AppImage`

## API 接口

- 获取机器码: `GET http://localhost:18888/api/machine-code`
- 授权状态: `GET http://localhost:18888/api/auth-status`
- 设置授权: `POST http://localhost:18888/api/set-auth`，Body: `{"authorized": true}`
- 健康检查: `GET http://localhost:18888/health`
- 用户协议：请修改index.html 中 openUserAgreement 的配置获取地址
- 隐私策略：请修改index.html 中 openPrivacyPolicy 的配置获取地址

## 配置文件

- Windows: `%APPDATA%\machine-code-tool\config.json`
- macOS: `~/Library/Application Support/machine-code-tool/config.json`
- Linux: `~/.config/machine-code-tool/config.json`

## 技术栈

- **Tauri 1.x**: 桌面壳（非 2.0）
- **Rust**: 系统信息、HTTP 服务、加密
- **warp**: HTTP 服务
- **sysinfo / 平台 API**: 硬件信息

## 故障排除

- **Windows 7 报「无法定位程序输入点 ProcessPrng 于 bcryptprimitives.dll」**：构建时使用了 Rust **1.78 及以上**。Rust 1.78 起不再支持 Windows 7。请使用项目固定的 **1.77.2** 重新构建（根目录 `rust-toolchain.toml` 已配置；本地可执行 `rustup override set 1.77.2`），或通过 GitHub Actions 重新下载产物。
- **CI 报 `edition2024`（如 `getrandom-0.4.3`、`time-0.3.53`、`ignore-0.4.26`）**：无 `Cargo.lock` 时 cargo 会拉 crates.io 最新依赖，许多 crate 已切 edition2024，Cargo 1.77.2 无法解析。项目已通过 `Cargo.toml` + `src-tauri/Cargo.lock` 锁定兼容版本；**必须提交 `Cargo.lock`**。根因通常是 `uuid`/`tempfile` 新版本引入 `getrandom 0.4`，已在 `scripts/edition2021-pins.txt` 统一维护 pin 列表。本地可执行 `bash scripts/pin-rust1772-deps.sh` 重新生成 lockfile。
- **端口 18888 被占用**：修改配置文件中的 `port` 或关闭占用程序
- **获取硬件信息失败**：确认已点击「开启授权」；Windows 可尝试管理员权限
- **麒麟 / Linux 报 GLIBC_2.xx not found**：当前 Linux 包在 Ubuntu 20.04 环境构建，需 glibc 2.31+；麒麟 V10 一般满足

## 版权与支持

© 广联达科技股份有限公司  
技术支持：service@glodon.com / tech-support@glodon.com
