use tauri::{AppHandle, State};

use crate::config;
use crate::db::{self, CharacterModel, DbState};

#[tauri::command]
pub fn create_model(
    app: AppHandle,
    state: State<DbState>,
    model: CharacterModel,
) -> Result<i64, String> {
    // 1. 先插入数据库，拿到 ID
    let id = {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        db::create_model(&conn, &model).map_err(|e| e.to_string())?
    };

    // 2. 自动生成 TTS 配置文件
    let configs_dir = config::get_configs_dir(&app);
    let config_path = config::generate_model_config(
        &model.name,
        id,
        &model.gpt_model,
        &model.sovits_model,
        &model.prompt_text,
        &model.prompt_lang,
        &model.text_lang,
        &model.ref_audio_path,
        &configs_dir,
    )?;

    // 3. 回写 config_path 到数据库
    {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        db::update_config_path(&conn, id, &config_path.to_string_lossy())
            .map_err(|e| e.to_string())?;
    }

    // 4. 自动创建 5 个默认表情差分
    {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        db::seed_default_expressions(&conn, id).map_err(|e| e.to_string())?;
    }

    Ok(id)
}

#[tauri::command]
pub fn list_models(state: State<DbState>) -> Result<Vec<CharacterModel>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::list_models(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_model(state: State<DbState>, id: i64) -> Result<Option<CharacterModel>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::get_model(&conn, id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_model(
    app: AppHandle,
    state: State<DbState>,
    id: i64,
    model: CharacterModel,
) -> Result<bool, String> {
    // 1. 重新生成 TTS 配置文件
    let configs_dir = config::get_configs_dir(&app);
    let config_path = config::generate_model_config(
        &model.name,
        id,
        &model.gpt_model,
        &model.sovits_model,
        &model.prompt_text,
        &model.prompt_lang,
        &model.text_lang,
        &model.ref_audio_path,
        &configs_dir,
    )?;

    // 2. 更新数据库（带上新的 config_path）
    let mut updated = model;
    updated.config_path = config_path.to_string_lossy().to_string();
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::update_model(&conn, id, &updated).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_model(state: State<DbState>, id: i64) -> Result<bool, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    // 先拿到 config_path 再删除
    let model = db::get_model(&conn, id).map_err(|e| e.to_string())?;
    if let Some(m) = model {
        if !m.config_path.is_empty() {
            config::delete_config_file(&m.config_path).ok();
        }
    }
    db::delete_model(&conn, id).map_err(|e| e.to_string())
}
