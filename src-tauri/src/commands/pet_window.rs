use std::path::Path;
use tauri::{AppHandle, Manager};
use tauri::window::Color;

use crate::logging::log_msg;

/// 打开宠物窗口，展示角色立绘
/// 使用 run_on_main_thread 确保窗口创建在主线程完成
/// window_url: 可选的完整 URL（来自前端），避免 WebviewUrl::App 在子窗口的解析问题
#[tauri::command]
pub async fn open_pet_window(app: AppHandle, model_id: i64, window_url: Option<String>) -> Result<(), String> {
    let label = format!("pet-{}", model_id);

    // 如果窗口已存在，直接聚焦
    if let Some(window) = app.get_webview_window(&label) {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
        log_msg(&app, "INFO", &format!("宠物窗口已存在，聚焦: {}", label));
        return Ok(());
    }

    log_msg(&app, "INFO", &format!("创建宠物窗口: {}", label));

    // 构造 URL：优先使用前端传入的完整 URL，否则回退到 WebviewUrl::App
    let webview_url = if let Some(ref url_str) = window_url {
        log_msg(&app, "INFO", &format!("使用前端 URL: {}", url_str));
        let parsed = url_str.parse::<tauri::Url>()
            .map_err(|e| format!("URL 解析失败: {}", e))?;
        tauri::WebviewUrl::External(parsed)
    } else {
        let path = format!("/pet-window/{}", model_id);
        log_msg(&app, "INFO", &format!("使用 App URL: {}", path));
        tauri::WebviewUrl::App(path.into())
    };

    // 通过 channel 获取窗口创建结果
    let (tx, rx) = std::sync::mpsc::channel::<Result<(), String>>();
    let app_clone = app.clone();
    let label_clone = label.clone();

    app.run_on_main_thread(move || {
        match tauri::WebviewWindowBuilder::new(
            &app_clone,
            &label_clone,
            webview_url,
        )
        .title("")
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .inner_size(400.0, 600.0)
        .build()
        {
            Ok(window) => {
                if let Err(e) = window.set_background_color(Some(Color(0, 0, 0, 0))) {
                    log_msg(&app_clone, "WARN", &format!("设置背景透明失败: {}", e));
                }
                let _ = window.show();
                let _ = window.set_focus();
                log_msg(&app_clone, "INFO", &format!("宠物窗口创建成功: {}", label_clone));
                let _ = tx.send(Ok(()));
            }
            Err(e) => {
                let msg = format!("创建宠物窗口失败: {}", e);
                log_msg(&app_clone, "ERROR", &msg);
                let _ = tx.send(Err(msg));
            }
        }
    })
    .map_err(|e| format!("分发到主线程失败: {}", e))?;

    rx.recv().map_err(|e| format!("接收窗口创建结果失败: {}", e))?
}

/// 关闭宠物窗口
#[tauri::command]
pub fn close_pet_window(app: AppHandle, model_id: i64) -> Result<(), String> {
    let label = format!("pet-{}", model_id);
    log_msg(&app, "INFO", &format!("关闭宠物窗口: {}", label));
    if let Some(window) = app.get_webview_window(&label) {
        window.close().map_err(|e| e.to_string())?;
    } else {
        log_msg(&app, "WARN", &format!("宠物窗口不存在: {}", label));
    }
    Ok(())
}

/// 读取本地图片并返回 base64 data URL（避免 asset 协议配置问题）
#[tauri::command]
pub fn read_image_as_data_url(path: String) -> Result<String, String> {
    let data = std::fs::read(&path).map_err(|e| format!("读取图片失败: {}", e))?;
    let ext = Path::new(&path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png")
        .to_lowercase();
    let mime = match ext.as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        _ => "image/png",
    };
    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&data);
    Ok(format!("data:{};base64,{}", mime, b64))
}

/// 打开宠物窗口的开发者工具
#[tauri::command]
pub fn open_pet_devtools(app: AppHandle, model_id: i64) -> Result<(), String> {
    let label = format!("pet-{}", model_id);
    if let Some(window) = app.get_webview_window(&label) {
        #[cfg(debug_assertions)]
        {
            log_msg(&app, "INFO", &format!("打开开发者工具: {}", label));
            window.open_devtools();
        }
        #[cfg(not(debug_assertions))]
        {
            log_msg(&app, "WARN", "开发者工具仅支持调试模式");
            return Err("开发者工具仅支持调试模式 (tauri dev)".to_string());
        }
        Ok(())
    } else {
        Err(format!("宠物窗口 {} 不存在", label))
    }
}
