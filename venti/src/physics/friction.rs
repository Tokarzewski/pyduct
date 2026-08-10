//! Friction-related correlations for duct flow.

use std::f64::consts::LN_10;

/// Below this Reynolds number flow is treated as laminar (f = 64/Re).
pub const LAMINAR_RE_LIMIT: f64 = 2300.0;

/// Reynolds number Re = v * D_h / nu.
#[inline]
pub fn reynolds(velocity: f64, hydraulic_diameter: f64, kinematic_viscosity: f64) -> f64 {
    velocity * hydraulic_diameter / kinematic_viscosity
}

/// Relative roughness epsilon / D_h.
#[inline]
pub fn relative_roughness(absolute_roughness: f64, hydraulic_diameter: f64) -> f64 {
    absolute_roughness / hydraulic_diameter
}

/// Darcy friction factor (Swamee–Jain explicit approximation).
///
/// Falls back to laminar `64 / Re` for Re < 2300.
///
/// # Examples
/// ```
/// use venti::physics::friction::friction_factor;
/// let f = friction_factor(50_000.0, 0.0009);
/// assert!((0.015..0.05).contains(&f));
/// assert!((friction_factor(1_000.0, 0.001) - 0.064).abs() < 1e-12); // laminar
/// ```
#[inline]
pub fn friction_factor(reynolds_number: f64, rel_roughness: f64) -> f64 {
    if reynolds_number < LAMINAR_RE_LIMIT {
        return 64.0 / reynolds_number;
    }
    let arg = 0.234 * rel_roughness.powf(1.1007) - 60.525 / reynolds_number.powf(1.1105)
        + 56.291 / reynolds_number.powf(1.0712);
    let l = arg.ln();
    1.613 / (l * l)
}

/// Darcy friction factor from the implicit Colebrook–White equation.
///
/// Fixed-point iteration seeded from the Swamee–Jain estimate.
pub fn friction_factor_colebrook(
    reynolds_number: f64,
    rel_roughness: f64,
    tol: f64,
    max_iter: usize,
) -> f64 {
    if reynolds_number < LAMINAR_RE_LIMIT {
        return 64.0 / reynolds_number;
    }
    let mut f = friction_factor(reynolds_number, rel_roughness);
    for _ in 0..max_iter {
        // log10(x) = ln(x)/ln(10)
        let rhs = -2.0 * (rel_roughness / 3.71 + 2.51 / (reynolds_number * f.sqrt())).ln() / LN_10;
        let f_new = 1.0 / (rhs * rhs);
        if (f_new - f).abs() < tol {
            return f_new;
        }
        f = f_new;
    }
    f
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Closed-form: laminar Re < 2300 -> f = 64/Re exactly.
    #[test]
    fn laminar_falloff() {
        assert!((friction_factor(1000.0, 0.001) - 64.0 / 1000.0).abs() < 1e-12);
        assert!((friction_factor_colebrook(1000.0, 0.001, 1e-12, 100) - 0.064).abs() < 1e-12);
    }

    /// Swamee–Jain for a typical HVAC case stays in a physical range.
    #[test]
    fn turbulent_friction_is_reasonable() {
        // Re = 5e4, eps/D = 0.0009 -> f ~ 0.022
        let f = friction_factor(50_000.0, 0.0009);
        assert!((f - 0.022).abs() < 0.005, "f = {f}");
    }

    #[test]
    fn colebrook_agrees_with_swamee_jain() {
        let sj = friction_factor(100_000.0, 0.001);
        let cb = friction_factor_colebrook(100_000.0, 0.001, 1e-12, 100);
        // Colebrook is the more accurate implicit solve; should be close.
        assert!((sj - cb).abs() < 0.002, "sj={sj} cb={cb}");
    }

    #[test]
    fn reynolds_direct() {
        assert!((reynolds(4.0, 0.2, 1.5e-5) - 4.0 * 0.2 / 1.5e-5).abs() < 1e-12);
    }

    #[test]
    fn relative_roughness_direct() {
        assert!((relative_roughness(0.0001, 0.2) - 0.0005).abs() < 1e-12);
    }
}
