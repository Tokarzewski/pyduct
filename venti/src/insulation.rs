//! Thermal insulation of ducts — calculation & selection (izolacja termiczna).
//!
//! Issues #39: computing and choosing duct insulation thickness for two common
//! criteria (per EN ISO 12241 / ASHRAE practice):
//!
//! 1. **Condensation prevention** — enough insulation that the outer surface
//!    stays above the ambient **dew point** (cold supply air in a warm, humid
//!    space is the classic case);
//! 2. **Heat-loss / heat-gain limit** — enough insulation that the heat
//!    transfer per metre stays under a target.
//!
//! The thermal model is a steady-state cylindrical resistance network per
//! metre of duct (inner air film + cylindrical insulation + outer air film),
//! solved by growing the thickness until the criterion is met:
//!
//! ```text
//! R_in  = 1 / (h_i · π · D)                      inner air film
//! R_ins = ln(D_o / D) / (2π·λ)                   cylindrical insulation
//! R_out = 1 / (h_e · π · D_o)                    outer air film
//! q     = (T_air − T_amb) / (R_in + R_ins + R_out)     [W/m]
//! T_s   = T_amb + q · R_out                       outer surface temp [°C]
//! ```
//!
//! Condensation is avoided when `T_s ≥ T_dew`. All thicknesses are returned in
//! **metres** (library SI convention); [`select_thickness`] snaps to standard
//! millimetre steps.

use crate::Result;

/// Inner (air-side) heat-transfer coefficient default [W/m²K] — forced duct
/// airflow.
pub const DEFAULT_INNER_HTC: f64 = 10.0;
/// Outer (ambient-side) heat-transfer coefficient default [W/m²K] — still
/// indoor air.
pub const DEFAULT_OUTER_HTC: f64 = 8.0;

/// Largest insulation thickness [m] the solvers will consider.
pub const MAX_THICKNESS_M: f64 = 0.250;

/// Common standard insulation thicknesses [mm] used for selection.
pub const STANDARD_THICKNESS_MM: &[f64] = &[20.0, 30.0, 40.0, 50.0, 60.0, 80.0, 100.0, 120.0];

/// A thermal insulation material: name + thermal conductivity λ [W/(m·K)].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InsulationMaterial {
    pub name: &'static str,
    /// Thermal conductivity λ [W/(m·K)].
    pub conductivity: f64,
}

/// Typical duct insulants and their conductivities λ [W/(m·K)].
pub const MATERIALS: &[InsulationMaterial] = &[
    InsulationMaterial {
        name: "mineral_wool",
        conductivity: 0.035,
    },
    InsulationMaterial {
        name: "pe_foam",
        conductivity: 0.040,
    },
    InsulationMaterial {
        name: "epdm_nbr",
        conductivity: 0.038,
    },
    InsulationMaterial {
        name: "pir",
        conductivity: 0.024,
    },
    InsulationMaterial {
        name: "pu_foam",
        conductivity: 0.028,
    },
];

/// Look up a material's conductivity by name (case-insensitive), e.g.
/// `"mineral_wool"` → `0.035`.
pub fn material_conductivity(name: &str) -> Option<f64> {
    MATERIALS
        .iter()
        .find(|m| m.name.eq_ignore_ascii_case(name))
        .map(|m| m.conductivity)
}

/// Cylindrical insulation resistance per metre for outer diameter `d_o`.
fn r_ins(cond: f64, d: f64, d_o: f64) -> f64 {
    (d_o / d).ln() / (2.0 * std::f64::consts::PI * cond)
}

/// Steady-state per-metre heat flow through a duct with insulation thickness
/// `t` [m]. Returns q [W/m]; positive = outwards (hot duct), negative =
/// inwards (cold duct).
fn heat_flow(
    air_temp_c: f64,
    ambient_temp_c: f64,
    cond: f64,
    d_m: f64,
    t_m: f64,
    inner_htc: f64,
    outer_htc: f64,
) -> f64 {
    let d_o = d_m + 2.0 * t_m;
    let r_in = 1.0 / (inner_htc * std::f64::consts::PI * d_m);
    let r_ins = r_ins(cond, d_m, d_o);
    let r_out = 1.0 / (outer_htc * std::f64::consts::PI * d_o);
    (air_temp_c - ambient_temp_c) / (r_in + r_ins + r_out)
}

/// Outer surface temperature [°C] of the insulation for thickness `t` [m].
fn surface_temp(
    air_temp_c: f64,
    ambient_temp_c: f64,
    cond: f64,
    d_m: f64,
    t_m: f64,
    inner_htc: f64,
    outer_htc: f64,
) -> f64 {
    let q = heat_flow(
        air_temp_c,
        ambient_temp_c,
        cond,
        d_m,
        t_m,
        inner_htc,
        outer_htc,
    );
    let d_o = d_m + 2.0 * t_m;
    let r_out = 1.0 / (outer_htc * std::f64::consts::PI * d_o);
    ambient_temp_c + q * r_out
}

/// Smallest insulation thickness [m] so the outer surface stays at or above
/// the dew point (condensation prevention).
///
/// # Examples
/// ```
/// use venti::insulation::required_thickness_condensation;
/// // Cold supply air (8 °C) in a 24 °C room at 60% RH (dew point ≈ 15.8 °C),
/// // mineral wool λ=0.035, 200 mm duct, indoor film coefficients.
/// let t = required_thickness_condensation(
///     8.0, 15.8, 24.0, 0.035, 0.2, 10.0, 8.0).unwrap();
/// assert!(t > 0.0 && t < 0.1);
/// ```
pub fn required_thickness_condensation(
    air_temp_c: f64,
    dew_point_c: f64,
    ambient_temp_c: f64,
    conductivity: f64,
    duct_outer_diameter_m: f64,
    inner_htc: f64,
    outer_htc: f64,
) -> Result<f64> {
    validate(duct_outer_diameter_m, conductivity, inner_htc, outer_htc)?;
    // Already condensation-safe at t = 0?
    if surface_temp(
        air_temp_c,
        ambient_temp_c,
        conductivity,
        duct_outer_diameter_m,
        0.0,
        inner_htc,
        outer_htc,
    ) >= dew_point_c
    {
        return Ok(0.0);
    }
    // Grow thickness until the surface warms above the dew point.
    let step = 0.001; // 1 mm
    let mut t = 0.0;
    while t <= MAX_THICKNESS_M {
        t += step;
        if surface_temp(
            air_temp_c,
            ambient_temp_c,
            conductivity,
            duct_outer_diameter_m,
            t,
            inner_htc,
            outer_htc,
        ) >= dew_point_c
        {
            return Ok(t);
        }
    }
    Err("dew point cannot be reached within the maximum considered thickness".into())
}

/// Smallest insulation thickness [m] so the per-metre heat transfer magnitude
/// is at most `target_w_per_m` (heat-loss / heat-gain limit).
///
/// # Examples
/// ```
/// use venti::insulation::required_thickness_heat_loss;
/// let t = required_thickness_heat_loss(
///     60.0, 20.0, 10.0, 0.035, 0.2, 10.0, 8.0).unwrap();
/// assert!(t > 0.0);
/// ```
pub fn required_thickness_heat_loss(
    air_temp_c: f64,
    ambient_temp_c: f64,
    target_w_per_m: f64,
    conductivity: f64,
    duct_outer_diameter_m: f64,
    inner_htc: f64,
    outer_htc: f64,
) -> Result<f64> {
    validate(duct_outer_diameter_m, conductivity, inner_htc, outer_htc)?;
    if target_w_per_m <= 0.0 {
        return Err("target_w_per_m must be positive".into());
    }
    let at_zero = heat_flow(
        air_temp_c,
        ambient_temp_c,
        conductivity,
        duct_outer_diameter_m,
        0.0,
        inner_htc,
        outer_htc,
    )
    .abs();
    if at_zero <= target_w_per_m {
        return Ok(0.0);
    }
    let step = 0.001;
    let mut t = 0.0;
    while t <= MAX_THICKNESS_M {
        t += step;
        let q = heat_flow(
            air_temp_c,
            ambient_temp_c,
            conductivity,
            duct_outer_diameter_m,
            t,
            inner_htc,
            outer_htc,
        )
        .abs();
        if q <= target_w_per_m {
            return Ok(t);
        }
    }
    Err("heat-loss target cannot be met within the maximum considered thickness".into())
}

/// Per-metre heat transfer [W/m] through insulation of thickness `t` [m]
/// (positive = outwards). Useful to report the resulting loss after selection.
pub fn heat_loss_with_insulation(
    air_temp_c: f64,
    ambient_temp_c: f64,
    conductivity: f64,
    duct_outer_diameter_m: f64,
    thickness_m: f64,
    inner_htc: f64,
    outer_htc: f64,
) -> Result<f64> {
    validate(duct_outer_diameter_m, conductivity, inner_htc, outer_htc)?;
    if thickness_m < 0.0 {
        return Err("thickness_m must be non-negative".into());
    }
    Ok(heat_flow(
        air_temp_c,
        ambient_temp_c,
        conductivity,
        duct_outer_diameter_m,
        thickness_m,
        inner_htc,
        outer_htc,
    ))
}

/// Round a required thickness [m] up to the smallest standard step
/// ([`STANDARD_THICKNESS_MM`]). Errors when the requirement exceeds the
/// largest standard step.
///
/// # Examples
/// ```
/// use venti::insulation::{select_thickness, STANDARD_THICKNESS_MM};
/// let sel = select_thickness(0.045).unwrap();
/// assert_eq!(sel, 0.05); // 45 mm -> 50 mm step
/// let _ = STANDARD_THICKNESS_MM;
/// ```
pub fn select_thickness(required_m: f64) -> Result<f64> {
    if required_m < 0.0 {
        return Err("required_m must be non-negative".into());
    }
    let required_mm = required_m * 1000.0;
    let max = *STANDARD_THICKNESS_MM.last().unwrap();
    if required_mm > max {
        return Err(format!(
            "required thickness {required_mm:.0} mm exceeds the largest standard step {max:.0} mm"
        )
        .into());
    }
    for s in STANDARD_THICKNESS_MM {
        if *s >= required_mm {
            return Ok(s / 1000.0);
        }
    }
    unreachable!("largest standard step covers the required range")
}

fn validate(d_m: f64, cond: f64, inner_htc: f64, outer_htc: f64) -> Result<()> {
    if d_m <= 0.0 {
        return Err("duct_outer_diameter_m must be positive".into());
    }
    if cond <= 0.0 {
        return Err("conductivity must be positive".into());
    }
    if inner_htc <= 0.0 || outer_htc <= 0.0 {
        return Err("heat-transfer coefficients must be positive".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINERAL_WOOL: f64 = 0.035;
    const D: f64 = 0.2;
    const HI: f64 = 10.0;
    const HE: f64 = 8.0;

    #[test]
    fn no_insulation_when_no_condensation_risk() {
        // hot duct in cold ambient: surface stays above dew point at t=0
        let t = required_thickness_condensation(40.0, 5.0, 15.0, MINERAL_WOOL, D, HI, HE).unwrap();
        assert_eq!(t, 0.0);
    }

    #[test]
    fn condensation_thickness_grows_with_humidity() {
        let dry =
            required_thickness_condensation(8.0, 12.0, 24.0, MINERAL_WOOL, D, HI, HE).unwrap();
        let humid =
            required_thickness_condensation(8.0, 17.0, 24.0, MINERAL_WOOL, D, HI, HE).unwrap();
        assert!(
            humid >= dry,
            "higher dew point needs at least as much insulation"
        );
    }

    #[test]
    fn condensation_meets_criterion() {
        let t = required_thickness_condensation(8.0, 15.8, 24.0, MINERAL_WOOL, D, HI, HE).unwrap();
        let ts = surface_temp(8.0, 24.0, MINERAL_WOOL, D, t, HI, HE);
        assert!(ts >= 15.8 - 1e-6, "surface {ts} must be >= dew point");
        assert!(t > 0.0 && t < 0.1);
    }

    #[test]
    fn heat_loss_thickness_monotonic_with_target() {
        let strict =
            required_thickness_heat_loss(60.0, 20.0, 10.0, MINERAL_WOOL, D, HI, HE).unwrap();
        let loose =
            required_thickness_heat_loss(60.0, 20.0, 50.0, MINERAL_WOOL, D, HI, HE).unwrap();
        assert!(strict >= loose);
        assert!(required_thickness_heat_loss(60.0, 20.0, 0.0, MINERAL_WOOL, D, HI, HE).is_err());
    }

    #[test]
    fn heat_loss_meets_target() {
        let t = required_thickness_heat_loss(60.0, 20.0, 25.0, MINERAL_WOOL, D, HI, HE).unwrap();
        let q = heat_loss_with_insulation(60.0, 20.0, MINERAL_WOOL, D, t, HI, HE)
            .unwrap()
            .abs();
        assert!(q <= 25.0 + 1e-9);
    }

    #[test]
    fn heat_loss_with_insulation_decreases_with_thickness() {
        let thin = heat_loss_with_insulation(60.0, 20.0, MINERAL_WOOL, D, 0.02, HI, HE)
            .unwrap()
            .abs();
        let thick = heat_loss_with_insulation(60.0, 20.0, MINERAL_WOOL, D, 0.06, HI, HE)
            .unwrap()
            .abs();
        assert!(thick < thin);
    }

    #[test]
    fn select_thickness_rounds_up() {
        assert_eq!(select_thickness(0.020).unwrap(), 0.02);
        assert_eq!(select_thickness(0.045).unwrap(), 0.05);
        assert_eq!(select_thickness(0.061).unwrap(), 0.08);
        assert_eq!(select_thickness(0.001).unwrap(), 0.02); // 1 mm -> min step
        assert!(select_thickness(0.200).is_err()); // above max 120 mm
    }

    #[test]
    fn material_lookup() {
        assert_eq!(material_conductivity("mineral_wool"), Some(0.035));
        assert_eq!(material_conductivity("PIR"), Some(0.024));
        assert_eq!(material_conductivity("bogus"), None);
        assert_eq!(MATERIALS.len(), 5);
    }

    #[test]
    fn validation() {
        assert!(required_thickness_condensation(8.0, 15.0, 24.0, 0.0, D, HI, HE).is_err());
        assert!(
            required_thickness_condensation(8.0, 15.0, 24.0, MINERAL_WOOL, 0.0, HI, HE).is_err()
        );
        assert!(heat_loss_with_insulation(60.0, 20.0, MINERAL_WOOL, D, -0.01, HI, HE).is_err());
    }
}
