// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;
use tokio::sync::Mutex;
use warp::Filter;

mod hardware;
mod config;

use hardware::MachineInfo;
use config::AppConfig;

#[derive(Debug, Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub machine_info: Option<MachineInfo>,
    pub authorized: bool,
}

impl AppState {
    pub fn new() -> Self {
        let mut config = AppConfig::load();
        // 每次启动都默认为未授权状态
        config.authorized = false;
        let authorized = false;
        Self {
            config,
            machine_info: None,
            authorized,
        }
    }
}

// Tauri命令处理函数
#[tauri::command]
async fn get_machine_info_gui(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
) -> Result<serde_json::Value, String> {
    let mut app_state = state.lock().await;
    
    // 检查授权状态
    if !app_state.authorized {
        return Ok(serde_json::json!({
            "success": false,
            "message": "未开启授权，请先开启授权后再获取机器信息"
        }));
    }
    
    if app_state.machine_info.is_none() {
        match hardware::get_machine_info().await {
            Ok(info) => app_state.machine_info = Some(info),
            Err(e) => {
                   // eprintln!("获取机器信息失败: {}", e);
                return Ok(serde_json::json!({
                    "success": false,
                    "message": format!("获取机器信息失败: {}", e)
                }));
            }
        }
    }
    
    if let Some(ref info) = app_state.machine_info {
        Ok(serde_json::json!({
            "success": true,
            "data": info
        }))
    } else {
        Ok(serde_json::json!({
            "success": false,
            "message": "无法获取机器信息"
        }))
    }
}

#[tauri::command]
async fn get_auth_status_gui(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
) -> Result<serde_json::Value, String> {
    let app_state = state.lock().await;
    Ok(serde_json::json!({
        "authorized": app_state.authorized
    }))
}

#[derive(serde::Deserialize)]
struct AuthRequest {
    authorized: bool,
}

#[tauri::command]
async fn set_auth_status_gui(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
    request: AuthRequest,
) -> Result<serde_json::Value, String> {
    let mut app_state = state.lock().await;
    app_state.authorized = request.authorized;
    app_state.config.authorized = request.authorized;
    
    if let Err(e) = app_state.config.save() {
        return Ok(serde_json::json!({
            "success": false,
            "message": format!("保存配置失败: {}", e)
        }));
    }
    
    Ok(serde_json::json!({
        "success": true,
        "authorized": request.authorized
    }))
}

#[tauri::command]
async fn open_user_agreement() -> Result<(), String> {
    if let Err(e) = webbrowser::open("https://www.glodon.com/user-agreement") {
        return Err(format!("打开用户协议失败: {}", e));
    }
    Ok(())
}

#[tauri::command]
async fn open_privacy_policy() -> Result<(), String> {
    if let Err(e) = webbrowser::open("https://www.glodon.com/privacy-policy") {
        return Err(format!("打开隐私政策失败: {}", e));
    }
    Ok(())
}

#[tauri::command]
async fn toggle_devtools(window: tauri::Window) -> Result<(), String> {
    window.open_devtools();
    Ok(())
}

#[tauri::command]
async fn fetch_remote_content(url: String) -> Result<serde_json::Value, String> {
    let client = reqwest::Client::builder()
        .user_agent("Machine-Code-Tool/2.1.0")
        .build()
        .map_err(|e| format!("创建HTTP客户端失败: {}", e))?;
    // findConfByKey 类接口一般为 GET
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("网络请求失败: {}", e))?;
    if !response.status().is_success() {
        return Err(format!(
            "HTTP {} {}",
            response.status().as_u16(),
            response.status().canonical_reason().unwrap_or("Unknown")
        ));
    }
    let text = response
        .text()
        .await
        .map_err(|e| format!("读取响应失败: {}", e))?;
    serde_json::from_str::<serde_json::Value>(&text)
        .map_err(|e| format!("JSON解析失败: {}", e))
}

// 返回带完整 CORS 头的 OPTIONS 预检响应（直接用 http::Response，避免 warp reply 链未生效）
fn cors_preflight_response() -> warp::http::Response<Vec<u8>> {
    warp::http::Response::builder()
        .status(warp::http::StatusCode::NO_CONTENT)
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
        // 允许前端任意请求头（含 jc-gw-sign 等自定义头），避免预检被拒；非凭证请求可用 *
        .header("Access-Control-Allow-Headers", "*")
        .header("Access-Control-Max-Age", "86400")
        // 允许“公网页面”访问“本地 localhost”（Chrome Private Network Access），解决麒麟/浏览器侧请求 18888 被拦截
        .header("Access-Control-Allow-Private-Network", "true")
        .body(Vec::new())
        .expect("CORS preflight response build")
}

// HTTP服务器相关函数
async fn start_http_server(state: Arc<Mutex<AppState>>) {
    use warp::http::StatusCode;

    // CORS：允许跨域（GET/POST 响应也需 CORS 头，由 cors 层添加）
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["content-type", "Content-Type", "Origin", "Accept", "Access-Control-Request-Method", "Access-Control-Request-Headers"])
        .allow_methods(vec!["GET", "POST", "OPTIONS"])
        .build();

    let state_filter = warp::any().map(move || state.clone());

    // OPTIONS 预检：直接返回带 CORS 头的 204，不经过 warp CORS 层（确保预检一定带上 Access-Control-Allow-Origin）
    let opt_machine = warp::path!("api" / "machine-code").and(warp::options()).map(cors_preflight_response);
    let opt_auth = warp::path!("api" / "auth-status").and(warp::options()).map(cors_preflight_response);
    let opt_set = warp::path!("api" / "set-auth").and(warp::options()).map(cors_preflight_response);
    let opt_health = warp::path!("health").and(warp::options()).map(cors_preflight_response);

    let machine_code = warp::path!("api" / "machine-code")
        .and(warp::get())
        .and(state_filter.clone())
        .and_then(get_machine_code_handler);

    let auth_status = warp::path!("api" / "auth-status")
        .and(warp::get())
        .and(state_filter.clone())
        .and_then(get_auth_status_handler);

    let set_auth = warp::path!("api" / "set-auth")
        .and(warp::post())
        .and(warp::body::json())
        .and(state_filter.clone())
        .and_then(set_auth_handler);

    let health = warp::path!("health")
        .and(warp::get())
        .map(|| warp::reply::json(&"OK"));

    // 添加UTF-8编码头
    let utf8_header = warp::reply::with::header("content-type", "application/json; charset=utf-8");
    // 允许公网页面访问本地 18888（Chrome Private Network Access），与预检中的 Access-Control-Allow-Private-Network 一致
    let pna_header = warp::reply::with::header("Access-Control-Allow-Private-Network", "true");

    // 重要：OPTIONS 预检不经过 warp 的 cors 层，直接返回我们自己的带 CORS 头的 204，避免 warp 对 OPTIONS 的拦截/覆盖导致缺少 Access-Control-Allow-Origin
    let opt_routes = opt_machine.or(opt_auth).or(opt_set).or(opt_health);
    let api_routes = machine_code
        .or(auth_status)
        .or(set_auth)
        .or(health)
        .with(cors)
        .with(pna_header)
        .with(utf8_header);
    let routes = opt_routes.or(api_routes);

    // HTTP服务已启动: http://localhost:18888
    warp::serve(routes)
        .run(([127, 0, 0, 1], 18888))
        .await;
}

async fn get_machine_code_handler(
    state: Arc<Mutex<AppState>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut app_state = state.lock().await;
    
    // 检查授权状态
    if !app_state.authorized {
        return Ok(warp::reply::json(&serde_json::json!({
            "error": "未开启授权",
            "message": "请先开启授权后再获取机器信息",
            "authorized": false
        })));
    }
    
    if app_state.machine_info.is_none() {
        match hardware::get_machine_info().await {
            Ok(info) => app_state.machine_info = Some(info),
            Err(e) => {
                   // eprintln!("获取机器信息失败: {}", e);
                return Ok(warp::reply::json(&serde_json::json!({
                    "error": "获取机器信息失败",
                    "message": format!("获取机器信息失败: {}", e)
                })));
            }
        }
    }
    
    if let Some(ref info) = app_state.machine_info {
        Ok(warp::reply::json(info))
    } else {
        Ok(warp::reply::json(&serde_json::json!({
            "error": "无法获取机器信息"
        })))
    }
}

async fn get_auth_status_handler(
    state: Arc<Mutex<AppState>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let app_state = state.lock().await;
    Ok(warp::reply::json(&serde_json::json!({
        "authorized": app_state.authorized
    })))
}


async fn set_auth_handler(
    request: AuthRequest,
    state: Arc<Mutex<AppState>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut app_state = state.lock().await;
    app_state.authorized = request.authorized;
    app_state.config.authorized = request.authorized;
    
    if let Err(e) = app_state.config.save() {
        return Ok(warp::reply::json(&serde_json::json!({
            "success": false,
            "message": format!("保存配置失败: {}", e)
        })));
    }
    
    Ok(warp::reply::json(&serde_json::json!({
        "success": true,
        "authorized": request.authorized
    })))
}

#[tokio::main]
async fn main() {
    env_logger::init();
    
    let state = Arc::new(Mutex::new(AppState::new()));
    
    // 启动HTTP服务器
    let http_state = state.clone();
    tokio::spawn(async move {
        start_http_server(http_state).await;
    });

    // 启动Tauri GUI
    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            get_machine_info_gui,
            get_auth_status_gui,
            set_auth_status_gui,
            open_user_agreement,
            open_privacy_policy,
            toggle_devtools,
            fetch_remote_content
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}