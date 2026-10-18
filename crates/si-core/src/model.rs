//! Shared data model. Serialised as camelCase JSON for the front end.

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum AgentKind { Claude, Codex, OpenCode }

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Surface { Terminal, VsCode, Unknown }

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Activity { Working, Waiting, Idle }

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum WindowKind { FiveHour, Weekly }

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Source { StatusLine, Wrapper, CodexRollout }

/// One exact limit reading from an official source.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Reading {
    pub used_pct: f64,
    pub resets_at_ms: i64,
    pub at_ms: i64,
    pub source: Source,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Freshness { Exact, Estimated, Stale, Unknown }

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WindowView {
    pub used_pct: f64,
    pub left_pct: f64,
    pub resets_at_ms: Option<i64>,
    pub freshness: Freshness,
    pub age_ms: i64,
    pub burn_pts_per_min: f64,
    pub minutes_to_empty: Option<f64>,
    pub used_elsewhere_pts: Option<f64>,
    pub estimated: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Spend {
    pub session_usd: f64,
    pub today_usd: f64,
    pub session_tokens: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SessionView {
    pub id: String,
    pub kind: AgentKind,
    pub model_label: String,
    pub project: String,
    pub branch: Option<String>,
    pub surface: Surface,
    pub activity: Activity,
    pub last_event_ms: i64,
    pub ctx_tokens: Option<u64>,
    pub ctx_pct: Option<u8>,
    pub tokens_per_min: u64,
    pub spend: Option<Spend>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct AccountLimits {
    pub five_hour: Option<WindowView>,
    pub weekly: Option<WindowView>,
    pub overage_disabled: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub now_ms: i64,
    pub sessions: Vec<SessionView>,
    pub claude: AccountLimits,
    pub codex: AccountLimits,
    pub opencode_today: Option<Spend>,
}

/// Human label for a model id: "claude-opus-5-5" -> "Opus 5.5", "gpt-6-sol" -> "GPT 6 sol".
pub fn model_label(id: &str) -> String {
    let id = id.trim();
    if id.is_empty() {
        return "—".to_string();
    }
    let parts: Vec<&str> = id.split('-').collect();
    match parts.as_slice() {
        ["claude", family, major, minor] if is_num(major) && is_num(minor) => {
            format!("{} {}.{}", title(family), major, minor)
        }
        // OpenAI names keep their own casing after the prefix.
        ["gpt", rest @ ..] => std::iter::once("GPT").chain(rest.iter().copied()).collect::<Vec<_>>().join(" "),
        _ => parts.iter().map(|p| brand(p)).collect::<Vec<_>>().join(" "),
    }
}

fn is_num(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_digit())
}

fn brand(word: &str) -> String {
    match word {
        "deepseek" => "DeepSeek".to_string(),
        "opencode" => "OpenCode".to_string(),
        _ => title(word),
    }
}

fn title(word: &str) -> String {
    let mut c = word.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_label_formats_known_ids() {
        assert_eq!(model_label("claude-opus-5-5"), "Opus 5.5");
        assert_eq!(model_label("claude-sonnet-5-5"), "Sonnet 5.5");
        assert_eq!(model_label("gpt-6-sol"), "GPT 6 sol");
        assert_eq!(model_label("deepseek-flash"), "DeepSeek Flash");
        assert_eq!(model_label(""), "—");
    }
}
