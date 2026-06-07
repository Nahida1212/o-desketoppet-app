use tauri::State;

use crate::db::{self, CharacterLlmConfig, DbState};

#[tauri::command]
pub fn get_llm_config(state: State<DbState>, model_id: i64) -> Result<Option<CharacterLlmConfig>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::get_llm_config(&conn, model_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_llm_config(
    state: State<DbState>,
    model_id: i64,
    config: CharacterLlmConfig,
) -> Result<i64, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::save_llm_config(&conn, model_id, &config).map_err(|e| e.to_string())
}

/// 确保指定模型的 5 个默认表情差分存在（用于旧模型迁移）
#[tauri::command]
pub fn ensure_default_expressions(state: State<DbState>, model_id: i64) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::seed_default_expressions(&conn, model_id).map_err(|e| e.to_string())
}
