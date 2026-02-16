pub const WILSON_Z_95: f64 = 1.96;

pub fn wilson_score(positive: u64, total: u64) -> f64 {
    wilson_score_with_z(positive, total, WILSON_Z_95)
}

pub fn wilson_score_with_z(positive: u64, total: u64, z: f64) -> f64 {
    if total == 0 {
        return 0.0;
    }

    let clamped_positive = positive.min(total);
    let n = total as f64;
    let p_hat = clamped_positive as f64 / n;
    let z2 = z * z;

    let denominator = 1.0 + z2 / n;
    let center = p_hat + z2 / (2.0 * n);
    let margin = z * ((p_hat * (1.0 - p_hat) + z2 / (4.0 * n)) / n).sqrt();
    ((center - margin) / denominator).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f64 = 1e-6;

    fn assert_approx(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() < EPS,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn returns_zero_for_empty_sample() {
        assert_eq!(wilson_score(0, 0), 0.0);
    }

    #[test]
    fn known_values_match_reference_outputs() {
        assert_approx(wilson_score(10, 10), 0.7224598312333834);
        assert_approx(wilson_score(80, 100), 0.7111690380734976);
        assert_approx(wilson_score(1, 2), 0.09452865480086611);
    }

    #[test]
    fn clamps_positive_votes_to_total() {
        assert_eq!(wilson_score(12, 10), wilson_score(10, 10));
    }
}
