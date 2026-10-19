use si_core::alerts::{AlertEngine, AlertKind, Prefs};
use si_core::model::{AgentKind, Freshness, WindowKind, WindowView};

fn view(left: f64, reset: i64, burn: f64, eta: Option<f64>) -> WindowView {
    WindowView {
        used_pct: 100.0 - left,
        left_pct: left,
        resets_at_ms: Some(reset),
        freshness: Freshness::Exact,
        age_ms: 0,
        burn_pts_per_min: burn,
        minutes_to_empty: eta,
        used_elsewhere_pts: None,
        estimated: false,
    }
}

#[test]
fn threshold_fires_once_and_rearms_on_reset() {
    let mut e = AlertEngine::default();
    let first = e.evaluate(
        AgentKind::Claude,
        WindowKind::FiveHour,
        &view(20.0, 100_000_000, 0.0, None),
        "Demo",
        0,
    );
    assert!(first.iter().any(|a| a.kind == AlertKind::Left20));
    assert!(!e
        .evaluate(
            AgentKind::Claude,
            WindowKind::FiveHour,
            &view(20.0, 100_000_000, 0.0, None),
            "Demo",
            1000
        )
        .iter()
        .any(|a| a.kind == AlertKind::Left20));
    assert!(e
        .evaluate(
            AgentKind::Claude,
            WindowKind::FiveHour,
            &view(20.0, 200_000_000, 0.0, None),
            "Demo",
            11 * 60_000
        )
        .iter()
        .any(|a| a.kind == AlertKind::Left20));
}

#[test]
fn fast_drain_needs_projection_before_reset() {
    let mut e = AlertEngine::default();
    assert!(!e
        .evaluate(
            AgentKind::Codex,
            WindowKind::FiveHour,
            &view(40.0, 100_000_000, 2.0, None),
            "Demo",
            0
        )
        .iter()
        .any(|a| a.kind == AlertKind::FastDrain));
    assert!(e
        .evaluate(
            AgentKind::Codex,
            WindowKind::FiveHour,
            &view(40.0, 100_000_000, 2.0, Some(20.0)),
            "Demo",
            1
        )
        .iter()
        .any(|a| a.kind == AlertKind::FastDrain));
}

#[test]
fn cooldown_suppresses_repeat() {
    let mut e = AlertEngine::default();
    assert!(!e
        .evaluate(
            AgentKind::Codex,
            WindowKind::FiveHour,
            &view(20.0, 100_000_000, 0.0, None),
            "Demo",
            0
        )
        .is_empty());
    assert!(e
        .evaluate(
            AgentKind::Codex,
            WindowKind::Weekly,
            &view(20.0, 200_000_000, 0.0, None),
            "Demo",
            1000
        )
        .is_empty());
}

#[test]
fn copy_matches_spec() {
    let mut e = AlertEngine::default();
    e.prefs = Prefs::default();
    let reset = 1_792_329_000_000;
    let left20 = e.evaluate(
        AgentKind::Claude,
        WindowKind::FiveHour,
        &view(20.0, reset, 0.0, None),
        "Demo",
        0,
    );
    let a = left20.iter().find(|a| a.kind == AlertKind::Left20).unwrap();
    assert_eq!(a.title, "Claude Code · 20% left");
    assert!(a.body.starts_with("5-hour limit · resets "));
    assert!(a.body.ends_with(" · review model usage"));
    let fast = e.evaluate(
        AgentKind::Codex,
        WindowKind::FiveHour,
        &view(50.0, reset, 2.0, Some(25.0)),
        "VWMS",
        0,
    );
    let a = fast
        .iter()
        .find(|a| a.kind == AlertKind::FastDrain)
        .unwrap();
    assert_eq!(a.title, "Codex · draining fast");
    assert!(a.body.contains(" on VWMS · ~25 min left at this pace"));
    e.set_overage_disabled(AgentKind::Claude, true);
    let reached = e.evaluate(
        AgentKind::Claude,
        WindowKind::FiveHour,
        &view(0.0, reset + 10_000, 0.0, None),
        "Demo",
        11 * 60_000,
    );
    assert!(reached
        .iter()
        .find(|a| a.kind == AlertKind::LimitReached)
        .unwrap()
        .body
        .starts_with("Hard stop until "));
}

// Wording approved by the owner on 2026-10-19 (answers to docs/questions.md).
#[test]
fn owner_approved_copy() {
    let reset = 1_792_329_000_000;
    let mut e = AlertEngine::default();
    let left = e.evaluate(
        AgentKind::Claude,
        WindowKind::FiveHour,
        &view(10.0, reset, 0.0, None),
        "Demo",
        0,
    );
    let fifty = left.iter().find(|a| a.kind == AlertKind::Left50).unwrap();
    assert_eq!(fifty.title, "Claude Code · 50% left");
    assert!(fifty.body.starts_with("5-hour limit · resets "));
    assert!(!fifty.body.contains("review model usage"));
    let ten = left.iter().find(|a| a.kind == AlertKind::Left10).unwrap();
    assert_eq!(ten.title, "Claude Code · 10% left");
    assert!(ten.body.ends_with(" · pause or switch to a lighter model"));

    let reached = e.evaluate(
        AgentKind::Claude,
        WindowKind::FiveHour,
        &view(0.0, reset, 0.0, None),
        "Demo",
        11 * 60_000,
    );
    let r = reached
        .iter()
        .find(|a| a.kind == AlertKind::LimitReached)
        .unwrap();
    assert_eq!(r.title, "Claude Code · limit reached");
    assert!(r.body.starts_with("Resets "), "got {}", r.body);

    let fast = e.evaluate(
        AgentKind::Codex,
        WindowKind::FiveHour,
        &view(50.0, reset, 2.4, Some(20.0)),
        "VWMS",
        0,
    );
    let f = fast
        .iter()
        .find(|a| a.kind == AlertKind::FastDrain)
        .unwrap();
    assert_eq!(f.body, "~2.4%/min on VWMS · ~20 min left at this pace");

    let weekly = e.evaluate(
        AgentKind::Codex,
        WindowKind::Weekly,
        &view(20.0, reset, 0.0, None),
        "VWMS",
        0,
    );
    assert!(weekly
        .iter()
        .find(|a| a.kind == AlertKind::Left20)
        .unwrap()
        .body
        .starts_with("Weekly limit · resets "));
}
