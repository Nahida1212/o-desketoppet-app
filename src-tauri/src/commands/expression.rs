use tauri::State;

use crate::db::{self, CharacterExpression, DbState};

#[tauri::command]
pub fn list_expressions(state: State<DbState>, model_id: i64) -> Result<Vec<CharacterExpression>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::list_expressions(&conn, model_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_expression(
    state: State<DbState>,
    expression: CharacterExpression,
) -> Result<i64, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::create_expression(&conn, &expression).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_expression(
    state: State<DbState>,
    id: i64,
    expression: CharacterExpression,
) -> Result<bool, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::update_expression(&conn, id, &expression).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_expression(state: State<DbState>, id: i64) -> Result<bool, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::delete_expression(&conn, id).map_err(|e| e.to_string())
}
