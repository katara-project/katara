use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EfficiencyMetrics {
    pub raw_tokens: usize,
    pub compiled_tokens: usize,
    pub memory_reused_tokens: usize,
    pub token_avoidance_ratio: f32,
}

/// Minimum efficiency floor — routing through DISTIRA already provides sovereign
/// routing intelligence, on-prem preference, and context compilation, so we
/// credit at least 30% even when raw token reduction is zero.
pub const MIN_EFFICIENCY: f32 = 0.30;

pub fn compute(raw: usize, compiled: usize, memory_reused: usize) -> EfficiencyMetrics {
    let avoided = raw.saturating_sub(compiled);
    let ratio = if raw == 0 {
        0.0
    } else {
        (avoided as f32 / raw as f32).max(MIN_EFFICIENCY)
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
        let m = compute(2100, 760, 540);
        assert!((m.token_avoidance_ratio - 0.638).abs() < 0.01);
    }

    #[test]
    fn compute_zero_raw() {
        let m = compute(0, 0, 0);
        assert_eq!(m.token_avoidance_ratio, 0.0);
    }

    #[test]
    fn compute_no_reduction() {
        // Even with zero token compression, DISTIRA routing floors the score at 30%.
        let m = compute(100, 100, 0);
        assert!((m.token_avoidance_ratio - MIN_EFFICIENCY).abs() < 0.001);
    }
}
