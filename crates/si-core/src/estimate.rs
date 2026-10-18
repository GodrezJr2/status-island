use crate::model::Reading;

#[derive(Clone, Debug, Default)]
pub struct Estimator {
    rate: Option<f64>,
}

impl Estimator {
    pub fn learn(&mut self, prev: &Reading, next: &Reading, weighted_tokens: f64) {
        if prev.resets_at_ms != next.resets_at_ms || weighted_tokens <= 0.0 {
            return;
        }
        let delta = next.used_pct - prev.used_pct;
        if delta <= 0.0 {
            return;
        }
        let observed = delta / weighted_tokens;
        self.rate = Some(self.rate.map_or(observed, |old| old * 0.7 + observed * 0.3));
    }

    pub fn estimate(&self, last: &Reading, weighted_tokens_since: f64) -> Option<f64> {
        self.rate
            .map(|rate| (last.used_pct + rate * weighted_tokens_since.max(0.0)).clamp(0.0, 100.0))
    }
}

pub fn weighted_tokens(
    model: &str,
    input: u64,
    cache_create: u64,
    cache_read: u64,
    output: u64,
) -> f64 {
    let model = model.to_ascii_lowercase();
    let multiplier = if model.contains("opus") || model.contains("fable") {
        5.0
    } else if model.contains("sonnet") {
        3.0
    } else if model.contains("haiku") {
        1.0
    } else {
        3.0
    };
    multiplier
        * (input as f64
            + 1.25 * cache_create as f64
            + 0.1 * cache_read as f64
            + 5.0 * output as f64)
}
