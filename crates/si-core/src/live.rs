use crate::model::{Freshness, Reading, WindowView};

#[derive(Clone, Debug, Default)]
pub struct LimitTracker {
    latest: Option<Reading>,
    history: Vec<Reading>,
}

impl LimitTracker {
    pub fn ingest(&mut self, r: Reading) {
        if !r.used_pct.is_finite() || !(0.0..=100.0).contains(&r.used_pct) {
            return;
        }
        match &self.latest {
            Some(old) if r.resets_at_ms < old.resets_at_ms => return,
            Some(old) if r.resets_at_ms > old.resets_at_ms => self.history.clear(),
            _ => {}
        }
        if let Some(old) = &self.latest {
            if old.resets_at_ms == r.resets_at_ms && r.used_pct < old.used_pct {
                return;
            }
        }
        self.history.push(r.clone());
        self.history.sort_by_key(|x| x.at_ms);
        self.latest = Some(r);
    }

    pub fn view(&self, now_ms: i64, estimate: Option<f64>) -> Option<WindowView> {
        let last = self.latest.as_ref()?;
        let age_ms = (now_ms - last.at_ms).max(0);
        if last.resets_at_ms <= now_ms {
            return Some(WindowView {
                used_pct: 0.0,
                left_pct: 100.0,
                resets_at_ms: None,
                freshness: Freshness::Exact,
                age_ms: 0,
                burn_pts_per_min: 0.0,
                minutes_to_empty: None,
                used_elsewhere_pts: None,
                estimated: false,
            });
        }
        let freshness = if age_ms > 30 * 60_000 {
            Freshness::Stale
        } else if age_ms > 60_000 && estimate.is_some() {
            Freshness::Estimated
        } else {
            Freshness::Exact
        };
        let estimated = freshness == Freshness::Estimated;
        let used_pct = if estimated {
            estimate.unwrap_or(last.used_pct).clamp(0.0, 100.0)
        } else {
            last.used_pct
        };
        let left_pct = (100.0 - used_pct).max(0.0);
        let earliest = self
            .history
            .iter()
            .filter(|r| r.at_ms <= last.at_ms)
            .min_by_key(|r| r.at_ms);
        let burn_pts_per_min = earliest
            .map(|first| {
                let minutes = ((last.at_ms - first.at_ms) as f64 / 60_000.0).max(1.0);
                ((last.used_pct - first.used_pct) / minutes).max(0.0)
            })
            .unwrap_or(0.0);
        let projection = if burn_pts_per_min > 0.0 {
            Some(left_pct / burn_pts_per_min)
        } else {
            None
        };
        let minutes_to_empty =
            projection.filter(|mins| *mins * 60_000.0 < (last.resets_at_ms - now_ms) as f64);
        let used_elsewhere_pts = estimate.and_then(|e| {
            let diff = last.used_pct - e;
            (diff >= 2.0).then_some(diff)
        });
        Some(WindowView {
            used_pct,
            left_pct,
            resets_at_ms: Some(last.resets_at_ms),
            freshness,
            age_ms,
            burn_pts_per_min,
            minutes_to_empty,
            used_elsewhere_pts,
            estimated,
        })
    }
}
