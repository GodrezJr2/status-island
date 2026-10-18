use si_core::estimate::{weighted_tokens, Estimator};
use si_core::live::LimitTracker;
use si_core::model::{Freshness, Reading, Source};

fn r(used_pct: f64, at_ms: i64, resets_at_ms: i64) -> Reading {
    Reading {
        used_pct,
        at_ms,
        resets_at_ms,
        source: Source::CodexRollout,
    }
}

#[test]
fn max_wins_within_window() {
    let mut t = LimitTracker::default();
    for n in [13.0, 12.0, 13.0] {
        t.ingest(r(n, 1000, 1_000_000));
    }
    assert_eq!(t.view(2000, None).unwrap().used_pct, 13.0);
}

#[test]
fn new_window_replaces() {
    let mut t = LimitTracker::default();
    t.ingest(r(90.0, 1000, 1_000_000));
    t.ingest(r(2.0, 2000, 2_000_000));
    assert_eq!(t.view(3000, None).unwrap().used_pct, 2.0);
}

#[test]
fn reset_in_past_shows_zero() {
    let mut t = LimitTracker::default();
    t.ingest(r(90.0, 1000, 2000));
    let v = t.view(3000, None).unwrap();
    assert_eq!(v.used_pct, 0.0);
    assert_eq!(v.resets_at_ms, None);
}

#[test]
fn freshness_thresholds() {
    let mut t = LimitTracker::default();
    t.ingest(r(20.0, 0, 5_000_000));
    assert_eq!(t.view(59_000, None).unwrap().freshness, Freshness::Exact);
    assert_eq!(
        t.view(61_000, Some(21.0)).unwrap().freshness,
        Freshness::Estimated
    );
    assert_eq!(
        t.view(31 * 60_000, Some(21.0)).unwrap().freshness,
        Freshness::Stale
    );
}

#[test]
fn burn_and_projection() {
    let mut t = LimitTracker::default();
    t.ingest(r(1.0, 0, 4_000_000));
    t.ingest(r(13.0, 6 * 60_000, 4_000_000));
    let v = t.view(6 * 60_000, None).unwrap();
    assert_eq!(v.burn_pts_per_min, 2.0);
    assert_eq!(v.minutes_to_empty, Some(43.5));
    let mut short = LimitTracker::default();
    short.ingest(r(1.0, 0, 400_000));
    short.ingest(r(13.0, 6 * 60_000, 400_000));
    assert_eq!(short.view(6 * 60_000, None).unwrap().minutes_to_empty, None);
}

#[test]
fn estimator_learns_and_estimates() {
    let mut e = Estimator::default();
    e.learn(&r(10.0, 0, 100_000), &r(12.0, 1000, 100_000), 100.0);
    assert_eq!(e.estimate(&r(12.0, 1000, 100_000), 50.0), Some(13.0));
    assert_eq!(
        weighted_tokens("claude-opus-5-5", 1, 2, 10, 1),
        5.0 * (1.0 + 2.5 + 1.0 + 5.0)
    );
}

#[test]
fn used_elsewhere_detected() {
    let mut t = LimitTracker::default();
    t.ingest(r(36.0, 0, 3_000_000));
    assert_eq!(
        t.view(1000, Some(30.0)).unwrap().used_elsewhere_pts,
        Some(6.0)
    );
}
