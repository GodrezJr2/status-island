use crate::model::{Reading, Source, Surface};
use serde_json::Value;

#[derive(Clone, Debug, PartialEq)]
pub struct ClaudeTurn {
    pub ts_ms: i64,
    pub model: String,
    pub ctx_tokens: u64,
    pub output_tokens: u64,
    pub cwd: Option<String>,
    pub branch: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LiveSession {
    pub pid: u32,
    pub session_id: String,
    pub cwd: String,
    pub surface: Surface,
    pub busy: bool,
    pub proc_start: Option<String>,
}

pub fn parse_transcript_line(v: &Value) -> Option<ClaudeTurn> {
    if v.get("type")?.as_str()? != "assistant" {
        return None;
    }
    let message = v.get("message")?;
    let usage = message.get("usage")?;
    let input = usage.get("input_tokens")?.as_u64()?;
    let cache_create = usage.get("cache_creation_input_tokens")?.as_u64()?;
    let cache_read = usage.get("cache_read_input_tokens")?.as_u64()?;
    Some(ClaudeTurn {
        ts_ms: parse_time(v.get("timestamp")?).unwrap_or_default(),
        model: message.get("model")?.as_str()?.to_owned(),
        ctx_tokens: input
            .saturating_add(cache_create)
            .saturating_add(cache_read),
        output_tokens: usage
            .get("output_tokens")
            .and_then(Value::as_u64)
            .unwrap_or(0),
        cwd: v.get("cwd").and_then(Value::as_str).map(str::to_owned),
        branch: v
            .get("gitBranch")
            .and_then(Value::as_str)
            .map(str::to_owned),
    })
}

pub fn parse_session_file(v: &Value) -> Option<LiveSession> {
    let surface = match v.get("entrypoint")?.as_str()? {
        "claude-vscode" => Surface::VsCode,
        "cli" => Surface::Terminal,
        _ => Surface::Unknown,
    };
    Some(LiveSession {
        pid: u32::try_from(v.get("pid")?.as_u64()?).ok()?,
        session_id: v.get("sessionId")?.as_str()?.to_owned(),
        cwd: v.get("cwd")?.as_str()?.to_owned(),
        surface,
        busy: v.get("status").and_then(Value::as_str) == Some("busy"),
        proc_start: v
            .get("procStart")
            .and_then(Value::as_str)
            .map(str::to_owned),
    })
}

pub fn parse_statusline(v: &Value) -> (Option<Reading>, Option<Reading>, Option<u64>) {
    let limits = &v["rate_limits"];
    let at_ms = now_ms();
    let read = |field: &str| {
        let w = &limits[field];
        Some(Reading {
            used_pct: w.get("used_percentage")?.as_f64()?,
            resets_at_ms: parse_time(w.get("resets_at")?)?,
            at_ms,
            source: Source::StatusLine,
        })
    };
    (
        read("five_hour"),
        read("seven_day"),
        v["context_window"]["context_window_size"].as_u64(),
    )
}

pub fn parse_rate_limit_event(v: &Value) -> (Option<Reading>, Option<Reading>, bool) {
    if v.get("type").and_then(Value::as_str) != Some("rate_limit_event") {
        return (None, None, false);
    }
    let info = &v["rate_limit_info"];
    let windows = &info["unifiedWindows"];
    let at_ms = now_ms();
    let read = |field: &str| {
        let w = &windows[field];
        let pct = w.get("utilization")?.as_f64()? * 100.0;
        if !(0.0..=100.0).contains(&pct) {
            return None;
        }
        Some(Reading {
            used_pct: pct,
            resets_at_ms: parse_time(w.get("resetsAt")?)?,
            at_ms,
            source: Source::Wrapper,
        })
    };
    (
        read("five_hour"),
        read("seven_day"),
        info["overageDisabledReason"].as_str() == Some("org_level_disabled"),
    )
}

pub(crate) fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX))
        .unwrap_or(0)
}

pub(crate) fn parse_time(v: &Value) -> Option<i64> {
    if let Some(n) = v.as_i64() {
        return Some(if n.abs() < 100_000_000_000 {
            n * 1000
        } else {
            n
        });
    }
    let s = v.as_str()?;
    let date = s.get(..10)?;
    let y: i64 = date.get(..4)?.parse().ok()?;
    let m: i64 = date.get(5..7)?.parse().ok()?;
    let d: i64 = date.get(8..10)?.parse().ok()?;
    if !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return None;
    }
    let h: i64 = s.get(11..13)?.parse().ok()?;
    let min: i64 = s.get(14..16)?.parse().ok()?;
    let sec: i64 = s.get(17..19)?.parse().ok()?;
    if h > 23 || min > 59 || sec > 60 {
        return None;
    }
    let frac_ms = s
        .get(19..)
        .and_then(|tail| tail.strip_prefix('.'))
        .map(|tail| tail.chars().take(3).collect::<String>())
        .and_then(|digits| format!("{digits:0<3}").parse::<i64>().ok())
        .unwrap_or(0);
    // Howard Hinnant's civil-date-to-days calculation, with the Unix epoch offset.
    let yy = y - i64::from(m <= 2);
    let era = if yy >= 0 { yy } else { yy - 399 } / 400;
    let yoe = yy - era * 400;
    let mp = m + if m > 2 { -3 } else { 9 };
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some((((era * 146097 + doe - 719468) * 86400 + h * 3600 + min * 60 + sec) * 1000) + frac_ms)
}
