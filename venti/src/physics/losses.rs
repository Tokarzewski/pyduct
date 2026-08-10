//! Pressure-loss primitives (Darcy–Weisbach straight and local fittings).

/// Darcy–Weisbach straight-duct pressure drop [Pa].
///
/// ```text
/// dp = f * (L / D_h) * (rho * v^2 / 2)
/// ```
#[inline]
pub fn straight_pressure_drop(
    friction_factor: f64,
    length: f64,
    hydraulic_diameter: f64,
    velocity: f64,
    density: f64,
) -> f64 {
    friction_factor * (length / hydraulic_diameter) * (density * velocity * velocity * 0.5)
}

/// Local-fitting pressure drop ``dp = zeta * (rho * v^2 / 2)`` [Pa].
#[inline]
pub fn local_pressure_drop(zeta: f64, velocity: f64, density: f64) -> f64 {
    zeta * density * velocity * velocity * 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn straight_drop_closed_form() {
        let dp = straight_pressure_drop(0.02, 10.0, 0.2, 4.0, 1.204);
        // dp = 0.02 * (10/0.2) * (1.204*16/2) = 0.02*50*9.632 = 9.632
        assert!((dp - 9.632).abs() < 1e-12, "dp = {dp}");
    }

    #[test]
    fn local_drop_closed_form() {
        let dp = local_pressure_drop(1.0, 4.0, 1.204);
        assert!((dp - 1.204 * 16.0 * 0.5).abs() < 1e-12);
    }
}
