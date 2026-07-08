use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

fn log_path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("machine-code-tool")
        .join("startup.log")
}

pub fn append_log(message: &str) {
    let path = log_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        let _ = writeln!(file, "{}", message);
    }
}

pub fn log_path_display() -> String {
    log_path().display().to_string()
}

#[cfg(windows)]
pub fn show_fatal_error(message: &str) {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use winapi::um::winuser::{MessageBoxW, MB_ICONERROR, MB_OK};

    let text: Vec<u16> = OsStr::new(message).encode_wide().chain(Some(0)).collect();
    let title: Vec<u16> = OsStr::new("机器码获取工具")
        .encode_wide()
        .chain(Some(0))
        .collect();
    unsafe {
        MessageBoxW(
            std::ptr::null_mut(),
            text.as_ptr(),
            title.as_ptr(),
            MB_OK | MB_ICONERROR,
        );
    }
}

#[cfg(not(windows))]
pub fn show_fatal_error(message: &str) {
    eprintln!("{}", message);
}

#[cfg(windows)]
pub fn is_webview2_installed() -> bool {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use winapi::shared::minwindef::HKEY;
    use winapi::um::winnt::KEY_READ;
    use winapi::um::winreg::{RegCloseKey, RegOpenKeyExW, HKEY_LOCAL_MACHINE};

    const WEBVIEW2_CLIENT_KEY: &str =
        r"SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}";

    unsafe {
        let subkey: Vec<u16> = OsStr::new(WEBVIEW2_CLIENT_KEY)
            .encode_wide()
            .chain(Some(0))
            .collect();
        let mut hkey: HKEY = std::ptr::null_mut();
        if RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            subkey.as_ptr(),
            0,
            KEY_READ,
            &mut hkey,
        ) == 0
        {
            RegCloseKey(hkey);
            return true;
        }
    }

    [
        r"C:\Program Files (x86)\Microsoft\EdgeWebView\Application",
        r"C:\Program Files\Microsoft\EdgeWebView\Application",
    ]
    .iter()
    .any(|path| Path::new(path).exists())
}

#[cfg(not(windows))]
pub fn is_webview2_installed() -> bool {
    true
}

pub fn install_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let message = format!(
            "程序异常退出: {}\n\n详细日志: {}",
            info,
            log_path_display()
        );
        append_log(&message);
        show_fatal_error(&message);
    }));
}

pub fn preflight_checks() -> Result<(), String> {
    append_log("程序启动中...");

    #[cfg(windows)]
    if !is_webview2_installed() {
        let message = "未检测到 WebView2 运行时，界面无法启动。\n\n\
            请先安装 Microsoft Edge WebView2 运行时（Evergreen 引导程序），再重新运行本程序。\n\n\
            下载地址:\nhttps://go.microsoft.com/fwlink/p/?LinkId=2124703\n\n\
            若已安装仍闪退，请查看日志:\n"
            .to_string()
            + &log_path_display();
        append_log(&message);
        return Err(message);
    }

    Ok(())
}
