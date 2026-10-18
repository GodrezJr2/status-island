use crate::claude::parse_time;
use crate::model::{Reading, Source};
use serde_json::Value;

#[derive(Clone, Debug, PartialEq)]
pub enum CodexEvent {
    Meta {
        cwd: String,
        branch: Option<String>,
    },
    Model(String),
    TaskStarted(i64),
    TaskComplete(i64),
    Tokens {
        ts_ms: i64,
        input: u64,
        window: u64,
        total: u64,
    },
    Limits {
        ts_ms: i64,
        primary: Reading,
        secondary: Option<Reading>,
    },
}

pub fn parse_rollout_line(v: &Value) -> Vec<CodexEvent> {
    let mut out = Vec::new();
    let ts = v.get("timestamp").and_then(parse_time).unwrap_or(0);
    let p = &v["payload"];
    match v.get("type").and_then(Value::as_str) {
        Some("session_meta") => {
            if let Some(cwd) = p["cwd"].as_str() {
                out.push(CodexEvent::Meta {
                    cwd: cwd.to_owned(),
                    branch: p["git"]["branch"].as_str().map(str::to_owned),
                });
            }
        }
        Some("turn_context") => {
            if let Some(model) = p["model"].as_str() {
                out.push(CodexEvent::Model(model.to_owned()));
            }
        }
        Some("event_msg") => match p["type"].as_str() {
            Some("task_started") => out.push(CodexEvent::TaskStarted(ts)),
            Some("task_complete") => out.push(CodexEvent::TaskComplete(ts)),
            Some("token_count") => {
                let info = &p["info"];
                if let (Some(input), Some(window), Some(total)) = (
                    info["last_token_usage"]["input_tokens"].as_u64(),
                    info["model_context_window"].as_u64(),
                    info["total_token_usage"]["total_tokens"].as_u64(),
                ) {
                    out.push(CodexEvent::Tokens {
                        ts_ms: ts,
                        input,
                        window,
                        total,
                    });
                }
                let limits = &p["rate_limits"];
                if let Some(primary) = parse_limit(&limits["primary"], ts) {
                    out.push(CodexEvent::Limits {
                        ts_ms: ts,
                        primary,
                        secondary: parse_limit(&limits["secondary"], ts),
                    });
                }
            }
            _ => {}
        },
        _ => {}
    }
    out
}

fn parse_limit(v: &Value, ts: i64) -> Option<Reading> {
    let used_pct = v.get("used_percent")?.as_f64()?;
    if !(0.0..=100.0).contains(&used_pct) {
        return None;
    }
    Some(Reading {
        used_pct,
        resets_at_ms: parse_time(v.get("resets_at")?)?,
        at_ms: ts,
        source: Source::CodexRollout,
    })
}
