//! Flex-duct corrections.

/// Pressure-drop multiplier for a partially-stretched flex duct.
///
/// 100 % stretched → factor 1.0 (no correction); lower stretch → higher
/// multiplier. Curve fit (R² = 0.995) from ASHRAE Fundamentals.
#[inline]
pub fn stretch_correction_factor(diameter: f64, stretch_percentage: f64) -> f64 {
    0.557 * (100.0 - stretch_percentage) * (-4.93 * diameter).exp() + 1.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fully_stretched_is_identity() {
        assert!((stretch_correction_factor(0.2, 100.0) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn partial_stretch_increases_factor() {
        let full = stretch_correction_factor(0.2, 100.0);
        let partial = stretch_correction_factor(0.2, 60.0);
        assert!(partial > full);
    }
}
