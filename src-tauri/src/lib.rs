mod config;
mod db;
mod logging;
mod process;
mod commands;

use std::path::PathBuf;
use tauri::Manager;

use crate::db::DbState;
use crate::process::ProcessManager;

/// 获取数据库文件路径：{应用数据目录}/desktoppet.db
fn get_db_path(app: &tauri::AppHandle) -> PathBuf {
    let app_dir = app
        .path()
        .app_data_dir()
        .expect("failed to resolve app data dir");
    std::fs::create_dir_all(&app_dir).ok();
    app_dir.join("desktoppet.db")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let db_path = get_db_path(&app.handle());
            let db_state = DbState::new(
                db_path.to_str().expect("invalid db path"),
            )
            .expect("failed to initialize database");
            app.manage(db_state);
            app.manage(ProcessManager::new());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::create_model,
            commands::list_models,
            commands::get_model,
            commands::update_model,
            commands::delete_model,
            commands::start_model,
            commands::stop_model,
            commands::get_model_status,
            commands::list_expressions,
            commands::create_expression,
            commands::update_expression,
            commands::delete_expression,
            commands::get_llm_config,
            commands::save_llm_config,
            commands::ensure_default_expressions,
            commands::open_pet_window,
            commands::close_pet_window,
            commands::open_pet_devtools,
            commands::read_image_as_data_url,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
