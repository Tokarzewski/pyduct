//! Reynolds & size corrections for fitting loss coefficients (ζ).
//!
//! Published fitting data (ASHRAE Duct Fitting Database, Miller, Idelchik) is
//! **Reynolds- and size-dependent** — a constant ζ is only valid near one
//! (Re, duct-size) test point, and round elbows in particular lose slightly
//! less as Re grows. These helpers apply documented, conservative
//! multiplicative corrections to a base ζ so catalogued constants degrade
//! gracefully out of their test range.
//!
//! **Source note:** the exact coefficients of the licensed ASHRAE DB are not
//! reproduced here; the exponents are mild, physically-motivated values in the
//! direction the DB/SMACNA data trends (smaller ζ at higher Re, slightly
//! higher ζ for very small ducts), each clamped so the correction stays
//! within a few tens of percent.

use crate::physics::friction::reynolds;
use crate::Result;

/// Reynolds number of the reference test point used by catalogue constants.
pub const RE_REF: f64 = 50_000.0;

/// Nominal duct size [m] of the reference test point (200 mm).
pub const D_REF_M: f64 = 0.200;

fn clamp(v: f64, lo: f64, hi: f64) -> f64 {
    v.clamp(lo, hi)
}

/// Multiplicative Reynolds correction:
/// ``ζ(Re)/ζ_ref = (Re_ref / Re)^k``, clamped to `[0.75, 1.5]`.
///
/// `k > 0` makes higher-Re flow slightly *less* lossy (the round-elbow trend);
/// `k = 0` gives no correction. Guarded against non-positive Re.
pub fn re_correction(reynolds_number: f64, exponent: f64) -> Result<f64> {
    if reynolds_number <= 0.0 {
        return Err("reynolds_number must be positive".into());
    }
    Ok(clamp((RE_REF / reynolds_number).powf(exponent), 0.75, 1.5))
}

/// Multiplicative duct-size correction:
/// ``ζ(D)/ζ_ref = (D_ref / D)^s``, clamped to `[0.9, 1.3]`.
///
/// `s > 0` makes small ducts slightly *more* lossy (the size-effect trend in
/// the fitting data). Guarded against non-positive diameter.
pub fn size_correction(duct_diameter_m: f64, exponent: f64) -> Result<f64> {
    if duct_diameter_m <= 0.0 {
        return Err("duct_diameter_m must be positive".into());
    }
    Ok(clamp((D_REF_M / duct_diameter_m).powf(exponent), 0.9, 1.3))
}

/// Apply both the Reynolds and size corrections to a base ζ.
///
/// Convenience for any fitting whose catalogue value was measured at
/// (`RE_REF`, `D_REF_M`).
pub fn corrected_zeta(
    base_zeta: f64,
    velocity: f64,
    duct_diameter_m: f64,
    density: f64,
    dynamic_viscosity: f64,
    re_exponent: f64,
    size_exponent: f64,
) -> Result<f64> {
    let nu = dynamic_viscosity / density;
    let re = reynolds(velocity, duct_diameter_m, nu);
    let factor = re_correction(re, re_exponent)? * size_correction(duct_diameter_m, size_exponent)?;
    Ok(base_zeta * factor)
}

/// Re- and size-corrected smooth round-elbow loss coefficient.
///
/// Combines the algebraic [`crate::components::fittings_library::elbow_round`] base with
/// the mild Reynolds/size corrections, evaluated at the flow's actual velocity
/// and duct size.
///
/// # Examples
/// ```
/// use venti::re::elbow_round_loss;
/// // R/D = 1.0, 90°, 200 mm duct at 4 m/s under standard air
/// let z = elbow_round_loss(0.2, 0.2, 90.0, 4.0, 1.204, 1.825e-5).unwrap();
/// assert!(z > 0.0 && z < 0.7);
/// // higher velocity (higher Re) gives slightly lower loss, never below 75% of base
/// let z_fast = elbow_round_loss(0.2, 0.2, 90.0, 12.0, 1.204, 1.825e-5).unwrap();
/// assert!(z_fast <= z);
/// ```
pub fn elbow_round_loss(
    bend_radius: f64,
    diameter: f64,
    angle_deg: f64,
    velocity: f64,
    density: f64,
    dynamic_viscosity: f64,
) -> Result<f64> {
    let base = crate::components::fittings_library::elbow_round(bend_radius, diameter, angle_deg)?;
    let nu = dynamic_viscosity / density;
    let re = reynolds(velocity, diameter, nu);
    let factor = re_correction(re, 0.2)? * size_correction(diameter, 0.15)?;
    Ok(base * factor)
}

#[cfg(test)]
mod tests {
    use super::*;

    const RHO: f64 = 1.204;
    const MU: f64 = 1.825e-5;

    #[test]
    fn re_correction_trend_and_clamp() {
        // at the reference point factor = 1
        assert!((re_correction(RE_REF, 0.2).unwrap() - 1.0).abs() < 1e-12);
        // higher Re -> lower factor (less loss)
        assert!(re_correction(RE_REF * 10.0, 0.2).unwrap() < 1.0);
        // clamped: very high Re -> 0.75; very low Re -> 1.5
        assert!((re_correction(RE_REF * 100.0, 1.5).unwrap() - 0.75).abs() < 1e-9);
        assert!((re_correction(RE_REF / 100.0, 1.5).unwrap() - 1.5).abs() < 1e-9);
        assert!(re_correction(0.0, 0.2).is_err());
    }

    #[test]
    fn size_correction_trend_and_clamp() {
        assert!((size_correction(D_REF_M, 0.15).unwrap() - 1.0).abs() < 1e-12);
        // smaller duct -> larger factor
        assert!(size_correction(0.1, 0.15).unwrap() > 1.0);
        assert!((size_correction(0.01, 1.0).unwrap() - 1.3).abs() < 1e-9); // clamped
        assert!(size_correction(0.0, 0.1).is_err());
    }

    #[test]
    fn corrected_elbow_within_bounds_and_monotonic_re() {
        // Re ≈ 4*0.2/(1.516e-5) ≈ 5.3e4 ~ reference -> factor ~ 1
        let z = elbow_round_loss(0.2, 0.2, 90.0, 4.0, RHO, MU).unwrap();
        let base = 0.3; // elbow_round(0.2,0.2,90) = 0.21/sqrt(1.0) = 0.21? clamp min...
                        // elbow_round R/D=1 -> 0.21*1 = 0.21
        let base_actual = 0.21;
        assert!(z > base_actual * 0.75 && z < base_actual * 1.5, "z={z}");
        let _ = base;

        let re_lo = elbow_round_loss(0.2, 0.2, 90.0, 0.5, RHO, MU).unwrap();
        let re_hi = elbow_round_loss(0.2, 0.2, 90.0, 15.0, RHO, MU).unwrap();
        assert!(re_hi <= re_lo, "higher Re must not increase loss");
    }

    #[test]
    fn corrected_zeta_convenience() {
        let z = corrected_zeta(0.3, 6.0, 0.3, RHO, MU, 0.2, 0.15).unwrap();
        assert!(z > 0.2 && z < 0.5, "z={z}");
    }
}
