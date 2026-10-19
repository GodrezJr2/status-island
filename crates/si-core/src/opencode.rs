use crate::model::Spend;
use rusqlite::{
    hooks::{AuthAction, Authorization},
    Connection, OpenFlags,
};
use std::path::Path;

#[derive(Clone, Debug, PartialEq)]
pub struct OcSession {
    pub id: String,
    pub directory: String,
    pub model: Option<String>,
    pub cost: f64,
    pub tokens: u64,
    pub updated_ms: i64,
    pub idle_ms: Option<i64>,
}

/// OpenCode stores the model as JSON (`{"id":"deepseek-flash","providerID":"deepseek"}`);
/// return its `id`. Anything else passes through unchanged.
pub fn model_id(raw: &str) -> String {
    serde_json::from_str::<serde_json::Value>(raw)
        .ok()
        .and_then(|v| v.get("id").and_then(|id| id.as_str()).map(str::to_owned))
        .unwrap_or_else(|| raw.to_owned())
}

pub fn read_opencode(db: &Path) -> rusqlite::Result<(Vec<OcSession>, Spend)> {
    let conn = Connection::open_with_flags(
        db,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;
    conn.authorizer(Some(
        |context: rusqlite::hooks::AuthContext<'_>| match context.action {
            AuthAction::Select => Authorization::Allow,
            AuthAction::Read {
                table_name: "session_v2" | "session" | "sqlite_master" | "sqlite_schema",
                ..
            } if context.database_name == Some("main") => Authorization::Allow,
            AuthAction::Function { function_name }
                if ["coalesce", "sum", "strftime"]
                    .contains(&function_name.to_ascii_lowercase().as_str()) =>
            {
                Authorization::Allow
            }
            _ => Authorization::Deny,
        },
    ));
    // OpenCode versions differ: some keep sessions in `session_v2`, others in `session`
    // (which has no `time_idle`). The table name only ever comes from this fixed list.
    let table = conn.query_row(
        "SELECT name FROM sqlite_master WHERE type = 'table' AND name IN ('session_v2', 'session') ORDER BY name = 'session_v2' DESC LIMIT 1",
        [],
        |r| r.get::<_, String>(0),
    )?;
    let table = if table == "session_v2" {
        "session_v2"
    } else {
        "session"
    };
    let select = |idle: &str| {
        format!("SELECT id, directory, model, cost, tokens_input + tokens_output + tokens_reasoning + tokens_cache_read + tokens_cache_write, time_updated, {idle} FROM {table} WHERE time_archived IS NULL ORDER BY time_updated DESC LIMIT 5")
    };
    let mut stmt = match conn.prepare(&select("time_idle")) {
        Ok(stmt) => stmt,
        Err(_) => conn.prepare(&select("NULL"))?,
    };
    let sessions = stmt
        .query_map([], |r| {
            let tokens: i64 = r.get(4)?;
            Ok(OcSession {
                id: r.get(0)?,
                directory: r.get(1)?,
                model: r.get::<_, Option<String>>(2)?.map(|m| model_id(&m)),
                cost: r.get(3)?,
                tokens: tokens.max(0) as u64,
                updated_ms: r.get(5)?,
                idle_ms: r.get(6)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let today = conn.query_row(&format!("SELECT COALESCE(SUM(cost),0), COALESCE(SUM(tokens_input + tokens_output + tokens_reasoning + tokens_cache_read + tokens_cache_write),0) FROM {table} WHERE time_archived IS NULL AND time_updated >= CAST(strftime('%s','now','localtime','start of day','utc') AS INTEGER) * 1000"), [], |r| {
        let tokens: i64 = r.get(1)?;
        Ok(Spend { session_usd: 0.0, today_usd: r.get(0)?, session_tokens: tokens.max(0) as u64 })
    })?;
    Ok((sessions, today))
}
