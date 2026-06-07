use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

/// 角色模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterModel {
    pub id: Option<i64>,
    pub name: String,
    /// 模型启动文件地址
    pub model_start_path: String,
    pub ref_audio_path: String,
    pub prompt_text: String,
    pub prompt_lang: String,
    pub text_lang: String,
    pub gpt_model: String,
    pub sovits_model: String,
    pub api_base_url: String,
    pub config_path: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

/// 角色差分 / 表情配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterExpression {
    pub id: Option<i64>,
    pub model_id: i64,
    /// 表情 key，如 "default", "happy", "sad"
    pub name: String,
    /// 显示名称，如 "常态", "开心", "难过"
    pub display_name: String,
    pub ref_audio_path: String,
    pub prompt_text: String,
    pub prompt_lang: String,
    pub text_lang: String,
    /// 立绘图片路径
    pub illustration_path: String,
    pub sort_order: i32,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

/// LLM 角色提示词配置（一对一关联 character_models）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterLlmConfig {
    pub id: Option<i64>,
    pub model_id: i64,
    pub system_prompt: String,
    pub user_prompt_template: String,
    pub greeting_message: String,
    pub temperature: f64,
    pub max_tokens: i32,
    pub top_p: f64,
    pub presence_penalty: f64,
    pub frequency_penalty: f64,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

/// 数据库状态，由 Tauri 管理
pub struct DbState {
    pub conn: Mutex<Connection>,
}

impl DbState {
    /// 打开/创建数据库并初始化表结构
    pub fn new(db_path: &str) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(db_path)?;

        // 启用 WAL 模式提升并发性能
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;

        // 建表
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS character_models (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                name            TEXT NOT NULL,
                model_start_path TEXT NOT NULL DEFAULT '',
                ref_audio_path  TEXT NOT NULL DEFAULT '',
                prompt_text     TEXT NOT NULL DEFAULT '',
                prompt_lang     TEXT NOT NULL DEFAULT 'zh',
                text_lang       TEXT NOT NULL DEFAULT 'zh',
                gpt_model       TEXT NOT NULL DEFAULT '',
                sovits_model    TEXT NOT NULL DEFAULT '',
                api_base_url    TEXT NOT NULL DEFAULT '',
                config_path     TEXT NOT NULL DEFAULT '',
                created_at      TEXT NOT NULL DEFAULT (datetime('now','localtime')),
                updated_at      TEXT NOT NULL DEFAULT (datetime('now','localtime'))
            );

            CREATE TABLE IF NOT EXISTS character_expressions (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                model_id        INTEGER NOT NULL,
                name            TEXT NOT NULL,
                display_name    TEXT NOT NULL DEFAULT '',
                ref_audio_path  TEXT NOT NULL DEFAULT '',
                prompt_text     TEXT NOT NULL DEFAULT '',
                prompt_lang     TEXT NOT NULL DEFAULT 'zh',
                text_lang       TEXT NOT NULL DEFAULT 'zh',
                illustration_path TEXT NOT NULL DEFAULT '',
                sort_order      INTEGER NOT NULL DEFAULT 0,
                created_at      TEXT NOT NULL DEFAULT (datetime('now','localtime')),
                updated_at      TEXT NOT NULL DEFAULT (datetime('now','localtime')),
                FOREIGN KEY (model_id) REFERENCES character_models(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS character_llm_configs (
                id                  INTEGER PRIMARY KEY AUTOINCREMENT,
                model_id            INTEGER NOT NULL UNIQUE,
                system_prompt       TEXT NOT NULL DEFAULT '',
                user_prompt_template TEXT NOT NULL DEFAULT '',
                greeting_message    TEXT NOT NULL DEFAULT '',
                temperature         REAL NOT NULL DEFAULT 0.7,
                max_tokens          INTEGER NOT NULL DEFAULT 1024,
                top_p               REAL NOT NULL DEFAULT 0.9,
                presence_penalty    REAL NOT NULL DEFAULT 0.0,
                frequency_penalty   REAL NOT NULL DEFAULT 0.0,
                created_at          TEXT NOT NULL DEFAULT (datetime('now','localtime')),
                updated_at          TEXT NOT NULL DEFAULT (datetime('now','localtime')),
                FOREIGN KEY (model_id) REFERENCES character_models(id) ON DELETE CASCADE
            );",
        )?;

        // 迁移：为旧版 character_expressions 表添加 illustration_path 列
        conn.execute_batch(
            "ALTER TABLE character_expressions ADD COLUMN illustration_path TEXT NOT NULL DEFAULT '';",
        ).ok();

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }
}

// ────────────────────────────── CRUD ──────────────────────────────

/// 创建角色模型
pub fn create_model(conn: &Connection, model: &CharacterModel) -> Result<i64, rusqlite::Error> {
    conn.execute(
        "INSERT INTO character_models (name, model_start_path, ref_audio_path, prompt_text,
         prompt_lang, text_lang, gpt_model, sovits_model, api_base_url, config_path)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            model.name,
            model.model_start_path,
            model.ref_audio_path,
            model.prompt_text,
            model.prompt_lang,
            model.text_lang,
            model.gpt_model,
            model.sovits_model,
            model.api_base_url,
            model.config_path,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// 查询所有角色模型
pub fn list_models(conn: &Connection) -> Result<Vec<CharacterModel>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, name, model_start_path, ref_audio_path, prompt_text,
                prompt_lang, text_lang, gpt_model, sovits_model,
                api_base_url, config_path, created_at, updated_at
         FROM character_models
         ORDER BY id DESC",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(CharacterModel {
            id: Some(row.get(0)?),
            name: row.get(1)?,
            model_start_path: row.get(2)?,
            ref_audio_path: row.get(3)?,
            prompt_text: row.get(4)?,
            prompt_lang: row.get(5)?,
            text_lang: row.get(6)?,
            gpt_model: row.get(7)?,
            sovits_model: row.get(8)?,
            api_base_url: row.get(9)?,
            config_path: row.get(10)?,
            created_at: row.get(11)?,
            updated_at: row.get(12)?,
        })
    })?;

    let mut models = Vec::new();
    for row in rows {
        models.push(row?);
    }
    Ok(models)
}

/// 根据 ID 查询单个角色模型
pub fn get_model(conn: &Connection, id: i64) -> Result<Option<CharacterModel>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, name, model_start_path, ref_audio_path, prompt_text,
                prompt_lang, text_lang, gpt_model, sovits_model,
                api_base_url, config_path, created_at, updated_at
         FROM character_models
         WHERE id = ?1",
    )?;

    let mut rows = stmt.query_map(params![id], |row| {
        Ok(CharacterModel {
            id: Some(row.get(0)?),
            name: row.get(1)?,
            model_start_path: row.get(2)?,
            ref_audio_path: row.get(3)?,
            prompt_text: row.get(4)?,
            prompt_lang: row.get(5)?,
            text_lang: row.get(6)?,
            gpt_model: row.get(7)?,
            sovits_model: row.get(8)?,
            api_base_url: row.get(9)?,
            config_path: row.get(10)?,
            created_at: row.get(11)?,
            updated_at: row.get(12)?,
        })
    })?;

    match rows.next() {
        Some(Ok(model)) => Ok(Some(model)),
        _ => Ok(None),
    }
}

/// 更新角色模型
pub fn update_model(conn: &Connection, id: i64, model: &CharacterModel) -> Result<bool, rusqlite::Error> {
    let affected = conn.execute(
        "UPDATE character_models SET
            name = ?1, model_start_path = ?2, ref_audio_path = ?3,
            prompt_text = ?4, prompt_lang = ?5, text_lang = ?6,
            gpt_model = ?7, sovits_model = ?8, api_base_url = ?9,
            config_path = ?10, updated_at = datetime('now','localtime')
         WHERE id = ?11",
        params![
            model.name,
            model.model_start_path,
            model.ref_audio_path,
            model.prompt_text,
            model.prompt_lang,
            model.text_lang,
            model.gpt_model,
            model.sovits_model,
            model.api_base_url,
            model.config_path,
            id,
        ],
    )?;
    Ok(affected > 0)
}

/// 删除角色模型
pub fn delete_model(conn: &Connection, id: i64) -> Result<bool, rusqlite::Error> {
    let affected = conn.execute("DELETE FROM character_models WHERE id = ?1", params![id])?;
    Ok(affected > 0)
}

/// 仅更新 config_path 字段（创建模型后自动生成配置文件时使用）
pub fn update_config_path(conn: &Connection, id: i64, config_path: &str) -> Result<bool, rusqlite::Error> {
    let affected = conn.execute(
        "UPDATE character_models SET config_path = ?1, updated_at = datetime('now','localtime') WHERE id = ?2",
        params![config_path, id],
    )?;
    Ok(affected > 0)
}

// ────────────────────── CharacterExpressions CRUD ──────────────────────

/// 查询角色所有差分
pub fn list_expressions(conn: &Connection, model_id: i64) -> Result<Vec<CharacterExpression>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, model_id, name, display_name, ref_audio_path,
                prompt_text, prompt_lang, text_lang, illustration_path, sort_order,
                created_at, updated_at
         FROM character_expressions
         WHERE model_id = ?1
         ORDER BY sort_order ASC, id ASC",
    )?;
    let rows = stmt.query_map(params![model_id], |row| {
        Ok(CharacterExpression {
            id: Some(row.get(0)?),
            model_id: row.get(1)?,
            name: row.get(2)?,
            display_name: row.get(3)?,
            ref_audio_path: row.get(4)?,
            prompt_text: row.get(5)?,
            prompt_lang: row.get(6)?,
            text_lang: row.get(7)?,
            illustration_path: row.get(8)?,
            sort_order: row.get(9)?,
            created_at: row.get(10)?,
            updated_at: row.get(11)?,
        })
    })?;
    let mut list = Vec::new();
    for row in rows {
        list.push(row?);
    }
    Ok(list)
}

/// 创建差分
pub fn create_expression(conn: &Connection, expr: &CharacterExpression) -> Result<i64, rusqlite::Error> {
    conn.execute(
        "INSERT INTO character_expressions (model_id, name, display_name, ref_audio_path,
         prompt_text, prompt_lang, text_lang, illustration_path, sort_order)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            expr.model_id,
            expr.name,
            expr.display_name,
            expr.ref_audio_path,
            expr.prompt_text,
            expr.prompt_lang,
            expr.text_lang,
            expr.illustration_path,
            expr.sort_order,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// 更新差分
pub fn update_expression(conn: &Connection, id: i64, expr: &CharacterExpression) -> Result<bool, rusqlite::Error> {
    let affected = conn.execute(
        "UPDATE character_expressions SET
            name = ?1, display_name = ?2, ref_audio_path = ?3,
            prompt_text = ?4, prompt_lang = ?5, text_lang = ?6,
            illustration_path = ?7, sort_order = ?8,
            updated_at = datetime('now','localtime')
         WHERE id = ?9",
        params![
            expr.name,
            expr.display_name,
            expr.ref_audio_path,
            expr.prompt_text,
            expr.prompt_lang,
            expr.text_lang,
            expr.illustration_path,
            expr.sort_order,
            id,
        ],
    )?;
    Ok(affected > 0)
}

/// 为指定模型创建 5 个默认表情差分（如果尚未存在）
pub fn seed_default_expressions(conn: &Connection, model_id: i64) -> Result<(), rusqlite::Error> {
    let defaults = [
        ("default", "常态", 0),
        ("happy", "高兴", 1),
        ("angry", "生气", 2),
        ("sad", "悲伤", 3),
        ("surprised", "惊讶", 4),
    ];
    for (name, display_name, sort_order) in &defaults {
        // 检查是否已存在同名差分
        let exists: bool = conn.query_row(
            "SELECT COUNT(*) > 0 FROM character_expressions WHERE model_id = ?1 AND name = ?2",
            params![model_id, name],
            |row| row.get(0),
        )?;
        if !exists {
            conn.execute(
                "INSERT INTO character_expressions (model_id, name, display_name, ref_audio_path,
                 prompt_text, prompt_lang, text_lang, illustration_path, sort_order)
                 VALUES (?1, ?2, ?3, '', '', 'zh', 'zh', '', ?4)",
                params![model_id, name, display_name, sort_order],
            )?;
        }
    }
    Ok(())
}

/// 删除差分
pub fn delete_expression(conn: &Connection, id: i64) -> Result<bool, rusqlite::Error> {
    let affected = conn.execute("DELETE FROM character_expressions WHERE id = ?1", params![id])?;
    Ok(affected > 0)
}

// ────────────────────── CharacterLlmConfig CRUD ──────────────────────

/// 获取 LLM 提示词配置（一对一）
pub fn get_llm_config(conn: &Connection, model_id: i64) -> Result<Option<CharacterLlmConfig>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, model_id, system_prompt, user_prompt_template, greeting_message,
                temperature, max_tokens, top_p, presence_penalty, frequency_penalty,
                created_at, updated_at
         FROM character_llm_configs
         WHERE model_id = ?1",
    )?;
    let mut rows = stmt.query_map(params![model_id], |row| {
        Ok(CharacterLlmConfig {
            id: Some(row.get(0)?),
            model_id: row.get(1)?,
            system_prompt: row.get(2)?,
            user_prompt_template: row.get(3)?,
            greeting_message: row.get(4)?,
            temperature: row.get(5)?,
            max_tokens: row.get(6)?,
            top_p: row.get(7)?,
            presence_penalty: row.get(8)?,
            frequency_penalty: row.get(9)?,
            created_at: row.get(10)?,
            updated_at: row.get(11)?,
        })
    })?;
    match rows.next() {
        Some(Ok(cfg)) => Ok(Some(cfg)),
        _ => Ok(None),
    }
}

/// 保存 LLM 提示词配置（INSERT OR REPLACE）
pub fn save_llm_config(conn: &Connection, model_id: i64, cfg: &CharacterLlmConfig) -> Result<i64, rusqlite::Error> {
    // 先尝试更新
    let existing = conn.execute(
        "UPDATE character_llm_configs SET
            system_prompt = ?1, user_prompt_template = ?2, greeting_message = ?3,
            temperature = ?4, max_tokens = ?5, top_p = ?6,
            presence_penalty = ?7, frequency_penalty = ?8,
            updated_at = datetime('now','localtime')
         WHERE model_id = ?9",
        params![
            cfg.system_prompt,
            cfg.user_prompt_template,
            cfg.greeting_message,
            cfg.temperature,
            cfg.max_tokens,
            cfg.top_p,
            cfg.presence_penalty,
            cfg.frequency_penalty,
            model_id,
        ],
    )?;
    if existing > 0 {
        // 更新成功，返回现有记录的 id
        let mut stmt = conn.prepare("SELECT id FROM character_llm_configs WHERE model_id = ?1")?;
        let id: i64 = stmt.query_row(params![model_id], |row| row.get(0))?;
        Ok(id)
    } else {
        // 不存在，插入新记录
        conn.execute(
            "INSERT INTO character_llm_configs (model_id, system_prompt, user_prompt_template,
             greeting_message, temperature, max_tokens, top_p, presence_penalty, frequency_penalty)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                model_id,
                cfg.system_prompt,
                cfg.user_prompt_template,
                cfg.greeting_message,
                cfg.temperature,
                cfg.max_tokens,
                cfg.top_p,
                cfg.presence_penalty,
                cfg.frequency_penalty,
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }
}

/// 删除 LLM 配置
pub fn delete_llm_config(conn: &Connection, model_id: i64) -> Result<bool, rusqlite::Error> {
    let affected = conn.execute(
        "DELETE FROM character_llm_configs WHERE model_id = ?1",
        params![model_id],
    )?;
    Ok(affected > 0)
}
