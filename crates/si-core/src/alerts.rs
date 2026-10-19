use crate::model::{AgentKind, WindowKind, WindowView};
use chrono::{Local, TimeZone};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AlertKind {
    Left50,
    Left20,
    Left10,
    LimitReached,
    FastDrain,
    ContextHigh,
    TaskFinished,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Level {
    Warn,
    Crit,
    Info,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Alert {
    pub agent: AgentKind,
    pub kind: AlertKind,
    pub title: String,
    pub body: String,
    pub priority: u8,
    pub level: Level,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Prefs {
    pub context_high: bool,
    pub task_finished: bool,
}

#[derive(Default)]
pub struct AlertEngine {
    fired: HashSet<(AgentKind, WindowKind, AlertKind, i64)>,
    last_sent: HashMap<(AgentKind, AlertKind), i64>,
    overage_disabled: HashMap<AgentKind, bool>,
    pub prefs: Prefs,
}

impl AlertEngine {
    pub fn set_overage_disabled(&mut self, agent: AgentKind, disabled: bool) {
        self.overage_disabled.insert(agent, disabled);
    }

    pub fn evaluate(
        &mut self,
        agent: AgentKind,
        window: WindowKind,
        view: &WindowView,
        project: &str,
        now_ms: i64,
    ) -> Vec<Alert> {
        let Some(reset_ms) = view.resets_at_ms else {
            return Vec::new();
        };
        let left = view.left_pct;
        let mut candidates = Vec::new();
        if left <= 50.0 {
            candidates.push(AlertKind::Left50);
        }
        if left <= 20.0 {
            candidates.push(AlertKind::Left20);
        }
        if left <= 10.0 {
            candidates.push(AlertKind::Left10);
        }
        if left <= 0.0 {
            candidates.push(AlertKind::LimitReached);
        }
        if view.burn_pts_per_min >= 2.0 && view.minutes_to_empty.is_some() {
            candidates.push(AlertKind::FastDrain);
        }
        let reset = Local
            .timestamp_millis_opt(reset_ms)
            .single()
            .map(|d| d.format("%-I:%M %p").to_string())
            .unwrap_or_else(|| "unknown".to_owned());
        let name = match agent {
            AgentKind::Claude => "Claude Code",
            AgentKind::Codex => "Codex",
            AgentKind::OpenCode => "OpenCode",
        };
        let window_label = match window {
            WindowKind::FiveHour => "5-hour",
            WindowKind::Weekly => "Weekly",
        };
        let mut alerts = Vec::new();
        for kind in candidates {
            let key = (agent, window, kind, reset_ms);
            if self.fired.contains(&key)
                || self
                    .last_sent
                    .get(&(agent, kind))
                    .is_some_and(|last| now_ms - last < 600_000)
            {
                continue;
            }
            let (title, body, priority, level) = match kind {
                AlertKind::Left50 => (
                    format!("{name} · 50% left"),
                    format!("{window_label} limit · resets {reset}"),
                    3,
                    Level::Info,
                ),
                AlertKind::Left20 => (
                    format!("{name} · 20% left"),
                    format!("{window_label} limit · resets {reset} · review model usage"),
                    4,
                    Level::Warn,
                ),
                AlertKind::Left10 => (
                    format!("{name} · 10% left"),
                    format!(
                        "{window_label} limit · resets {reset} · pause or switch to a lighter model"
                    ),
                    4,
                    Level::Crit,
                ),
                AlertKind::LimitReached => {
                    let body = if *self.overage_disabled.get(&agent).unwrap_or(&false) {
                        format!("Hard stop until {reset} (org overage off)")
                    } else {
                        format!("Resets {reset}")
                    };
                    (format!("{name} · limit reached"), body, 5, Level::Crit)
                }
                AlertKind::FastDrain => {
                    let eta = view.minutes_to_empty.unwrap_or(0.0).round() as u32;
                    (
                        format!("{name} · draining fast"),
                        format!(
                            "~{:.1}%/min on {project} · ~{eta} min left at this pace",
                            view.burn_pts_per_min
                        ),
                        4,
                        Level::Warn,
                    )
                }
                AlertKind::ContextHigh | AlertKind::TaskFinished => continue,
            };
            self.fired.insert(key);
            self.last_sent.insert((agent, kind), now_ms);
            alerts.push(Alert {
                agent,
                kind,
                title,
                body,
                priority,
                level,
            });
        }
        alerts
    }
}
