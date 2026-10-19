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

pub fn read_opencode(db: &Path) -> rusqlite::Result<(Vec<OcSession>, Spend)> {
    let conn = Connection::open_with_flags(
        db,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;
    conn.authorizer(Some(
        |context: rusqlite::hooks::AuthContext<'_>| match context.action {
            AuthAction::Select => Authorization::Allow,
            AuthAction::Read {
                table_name: "session_v2",
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
    let mut stmt = conn.prepare("SELECT id, directory, model, cost, tokens_input + tokens_output + tokens_reasoning + tokens_cache_read + tokens_cache_write, time_updated, time_idle FROM session_v2 WHERE time_archived IS NULL ORDER BY time_updated DESC LIMIT 5")?;
    let sessions = stmt
        .query_map([], |r| {
            let tokens: i64 = r.get(4)?;
            Ok(OcSession {
                id: r.get(0)?,
                directory: r.get(1)?,
                model: r.get(2)?,
                cost: r.get(3)?,
                tokens: tokens.max(0) as u64,
                updated_ms: r.get(5)?,
                idle_ms: r.get(6)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let today = conn.query_row("SELECT COALESCE(SUM(cost),0), COALESCE(SUM(tokens_input + tokens_output + tokens_reasoning + tokens_cache_read + tokens_cache_write),0) FROM session_v2 WHERE time_archived IS NULL AND time_updated >= CAST(strftime('%s','now','localtime','start of day','utc') AS INTEGER) * 1000", [], |r| {
        let tokens: i64 = r.get(1)?;
        Ok(Spend { session_usd: 0.0, today_usd: r.get(0)?, session_tokens: tokens.max(0) as u64 })
    })?;
    Ok((sessions, today))
}
