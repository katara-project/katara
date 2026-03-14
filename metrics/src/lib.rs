use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EfficiencyMetrics {
    pub raw_tokens: usize,
    pub compiled_tokens: usize,
    pub memory_reused_tokens: usize,
    pub token_avoidance_ratio: f32,
}

/// Sovereign routing bonus — routing through DISTIRA provides sovereign routing
/// intelligence, on-prem preference, and context compilation regardless of how
/// much the context was compressed. This 30% bonus is added on top of the raw
/// token avoidance ratio and capped at 1.0 (100%).
pub const SOVEREIGN_BONUS: f32 = 0.30;

pub fn compute(raw: usize, compiled: usize, memory_reused: usize) -> EfficiencyMetrics {
    let avoided = raw.saturating_sub(compiled);
    let ratio = if raw == 0 {
        0.0
    } else {
        (avoided as f32 / raw as f32 + SOVEREIGN_BONUS).min(1.0)
    };
    EfficiencyMetrics {
        raw_tokens: raw,
        compiled_tokens: compiled,
        memory_reused_tokens: memory_reused,
        token_avoidance_ratio: ratio,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_avoidance_ratio() {
        // 63.8% raw reduction + 30% sovereign bonus = 93.8%, capped at 100%.
        let m = compute(2100, 760, 540);
        assert!((m.token_avoidance_ratio - (0.638_f32 + SOVEREIGN_BONUS).min(1.0)).abs() < 0.01);
    }

    #[test]
    fn compute_zero_raw() {
        let m = compute(0, 0, 0);
        assert_eq!(m.token_avoidance_ratio, 0.0);
    }

    #[test]
    fn compute_no_reduction() {
        // 0% token compression + 30% sovereign bonus = 30%.
        let m = compute(100, 100, 0);
        assert!((m.token_avoidance_ratio - SOVEREIGN_BONUS).abs() < 0.001);
    }

    #[test]
    fn compute_partial_reduction() {
        // 24% raw reduction + 30% sovereign bonus = 54%.
        let m = compute(100, 76, 0);
        assert!((m.token_avoidance_ratio - 0.54_f32).abs() < 0.01);
    }

    #[test]
    fn compute_bonus_capped_at_100() {
        // 80% raw reduction + 30% = 110%, capped at 100%.
        let m = compute(100, 20, 0);
        assert!((m.token_avoidance_ratio - 1.0_f32).abs() < 0.001);
    }
}
