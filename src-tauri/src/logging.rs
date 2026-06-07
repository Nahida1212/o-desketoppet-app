use std::fs::OpenOptions;
use std::io::Write;
use chrono::Local;
use tauri::Manager;

/// 简单文件日志：写入 {app_data_dir}/logs/app.log，同时打印到 stdout
pub fn log_msg(app: &tauri::AppHandle, level: &str, msg: &str) {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    let line = format!("[{}] {}: {}", timestamp, level, msg);
    // stdout（tauri dev 终端可见）
    println!("{}", line);
    // 写入日志文件
    if let Ok(mut log_dir) = app.path().app_data_dir() {
        log_dir.push("logs");
        std::fs::create_dir_all(&log_dir).ok();
        log_dir.push("app.log");
        if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&log_dir) {
            let _ = writeln!(f, "{}", line);
            let _ = f.flush();
        }
    }
}
