# Tauri 降级到 v1 的影响说明

## 1. 对本项目功能的影响：**无影响**

- **Rust 侧**：当前用到的 API 在 v1 与 v2 中一致，无需改业务逻辑：
  - `tauri::Builder::default()`、`.manage()`、`.invoke_handler(tauri::generate_handler![...])`、`.run(tauri::generate_context!())`
  - `#[tauri::command]`、`tauri::State<>`、`tauri::Window`
  - 所有命令（获取机器码、授权状态、打开链接、HTTP 请求等）在 v1 中均可用。
- **前端侧**：`dist/index.html` 已同时兼容 v1 与 v2：
  - 优先使用 `window.__TAURI__.core.invoke`（v2），不存在时使用 `window.__TAURI__.invoke`（v1）。
  - 降级到 v1 后会自动走 v1 的 `invoke`，功能保持不变。
- **HTTP 服务**：warp 在独立线程中运行，与 Tauri 版本无关，不受影响。

结论：**功能行为与降级前一致，无需额外开发。**

---

## 2. 对 Windows 和 macOS 的影响：**无影响**

| 平台   | Tauri v1 支持 | 说明 |
|--------|----------------|------|
| Windows | ✅ 支持       | 仍使用系统 WebView2，安装与运行方式不变。 |
| macOS   | ✅ 支持       | 仍使用系统 WebKit，universal (x64 + arm64) 构建方式不变。 |
| Linux   | ✅ 支持       | 由 WebKit2GTK 4.1 改为 4.0，需安装 `libwebkit2gtk-4.0-dev`（CI 与文档已按此配置）。 |

- **主工作流**（`build.yml`）已更新：Linux 使用 `libwebkit2gtk-4.0-dev` + Tauri CLI 1.8，Windows/macOS 仅安装 Tauri CLI 1.8，无系统 WebView 依赖变更。
- **打包产物**：Windows 仍为 MSVC 构建 + 安装包，macOS 仍为 universal 或单架构 app，格式与使用方式不变。

结论：**Windows、macOS 行为与降级前一致；Linux 仅依赖从 4.1 改为 4.0，对功能无影响。**

---

## 3. Windows 对 32 位系统兼容吗？

- **当前 CI**：只构建 **x86_64-pc-windows-msvc**（64 位），产物为 64 位 exe，**不能在 32 位 Windows 上运行**。
- **若需要支持 32 位**：Tauri v1 + WebView2 支持 32 位（目标 `i686-pc-windows-msvc`），需在 CI 中增加 32 位构建（安装 i686 工具链、构建并打包 32 位 exe）。目前未配置，故**当前不兼容 32 位 Windows**。

---

## 4. Tauri v1 后，glibc 高于 2.31 的最新系统兼容吗？**兼容**

- Linux 二进制在**构建时**会链接当前系统的 glibc（例如在 Ubuntu 20.04 上构建则链接 glibc 2.31）。
- **向后兼容**：在 glibc 2.31 上构建的 AppImage，可以在 **glibc ≥ 2.31** 的任何系统上运行（如 Ubuntu 22.04 / 24.04、麒麟 V10 升级版、其他 glibc 2.35+ 发行版）。
- **不向后兼容**：在 glibc 2.35 上构建的二进制**不能**在 glibc 2.31 上运行。

当前麒麟/Linux 构建使用 Ubuntu 20.04（glibc 2.31），因此**既能在麒麟 V10（2.31）上运行，也能在 glibc 更高的最新系统上运行**。

