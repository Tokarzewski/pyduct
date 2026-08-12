//! Duct sizing methods: velocity, equal-friction, pressure-drop budget,
//! noise-limit, aspect-ratio, and batch velocity sizing.
//!
//! Mirrors `python/wenta/sizing.py` (the generic `Shape`-based entry points)
//! and `wentamojo/sizing.mojo` (the round/rectangular-specific kernels).

use crate::core::fluid::{Fluid, STANDARD_AIR};
use crate::core::geometry::{CrossSection, Rectangular, Round};
use crate::data::standard_sizes::{STANDARD_RECTANGULAR_DUCT_SIZES, STANDARD_ROUND_DUCT_SIZES};
use crate::physics::friction::{friction_factor, relative_roughness, reynolds};
use crate::Result;

/// Duct cross-section shape selector for the generic sizing functions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    Round,
    Rectangular,
}

/// ASHRAE-style maximum air velocity by space type (m/s).
///
/// Lower values keep flow noise below the typical NC target for that room.
pub const NOISE_LIMITS_M_S: &[(&str, f64)] = &[
    ("studio", 2.5), // recording / broadcast
    ("bedroom", 3.0),
    ("office", 4.0),
    ("classroom", 4.5),
    ("retail", 5.0),
    ("industrial", 7.5),
];

fn noise_limit(space_type: &str) -> Result<f64> {
    for (k, v) in NOISE_LIMITS_M_S {
        if *k == space_type {
            return Ok(*v);
        }
    }
    Err((format!(
        "unknown space_type {space_type:?}; expected one of studio|bedroom|office|classroom|retail|industrial"
    )).into())
}

/// Generic velocity-method sizing for any shape.
///
/// Returns `(cross_section, actual_velocity)`.
pub fn velocity_method(
    flowrate: f64,
    shape: Shape,
    target_velocity: f64,
    _absolute_roughness: f64,
    _fluid: &Fluid,
) -> Result<(CrossSection, f64)> {
    if flowrate <= 0.0 {
        return Err((format!("flowrate must be positive, got {flowrate}")).into());
    }
    if target_velocity <= 0.0 {
        return Err((format!("target_velocity must be positive, got {target_velocity}")).into());
    }

    // Velocity depends only on area, so a straight pass over the chosen
    // shape's sections is exactly what the reference does.
    let (section, v) = match shape {
        Shape::Round => {
            let mut last: Option<(CrossSection, f64)> = None;
            for d in STANDARD_ROUND_DUCT_SIZES {
                let s = CrossSection::Round(Round::new(f64::from(d) / 1000.0)?);
                let v = flowrate / s.area();
                last = Some((s, v));
                if v <= target_velocity {
                    return Ok((s, v));
                }
            }
            last.ok_or("no round standard sections")?
        }
        Shape::Rectangular => {
            let mut last: Option<(CrossSection, f64)> = None;
            for (w, h) in STANDARD_RECTANGULAR_DUCT_SIZES {
                let s = CrossSection::Rectangular(Rectangular::new(
                    f64::from(w) / 1000.0,
                    f64::from(h) / 1000.0,
                )?);
                let v = flowrate / s.area();
                last = Some((s, v));
                if v <= target_velocity {
                    return Ok((s, v));
                }
            }
            last.ok_or("no rectangular standard sections")?
        }
    };
    let _ = &_absolute_roughness;
    let _ = _fluid;
    Ok((section, v))
}

// ---------------------------------------------------------------------------
// Equal-friction helpers
// ---------------------------------------------------------------------------

fn dp_per_m(
    section_area: f64,
    section_d_h: f64,
    flowrate: f64,
    absolute_roughness: f64,
    fluid: &Fluid,
) -> f64 {
    let v = flowrate / section_area;
    let f = friction_factor(
        reynolds(v, section_d_h, fluid.kinematic_viscosity),
        relative_roughness(absolute_roughness, section_d_h),
    );
    f / section_d_h * (fluid.density * v * v) * 0.5
}

/// Generic equal-friction sizing for any shape.
///
/// Returns `(cross_section, velocity, pressure_drop_per_meter)`.
pub fn equal_friction_method(
    flowrate: f64,
    target_pressure_drop_per_meter: f64,
    shape: Shape,
    absolute_roughness: f64,
    fluid: &Fluid,
) -> Result<(CrossSection, f64, f64)> {
    if flowrate <= 0.0 {
        return Err((format!("flowrate must be positive, got {flowrate}")).into());
    }
    if target_pressure_drop_per_meter <= 0.0 {
        return Err((format!(
            "target_pressure_drop_per_meter must be positive, got {target_pressure_drop_per_meter}"
        ))
        .into());
    }
    let fluid = if fluid.density > 0.0 {
        fluid
    } else {
        &STANDARD_AIR
    };

    let (section, r) = match shape {
        Shape::Round => {
            let mut last: Option<(CrossSection, f64)> = None;
            for d in STANDARD_ROUND_DUCT_SIZES {
                let s = CrossSection::Round(Round::new(f64::from(d) / 1000.0)?);
                let r = dp_per_m(
                    s.area(),
                    s.hydraulic_diameter(),
                    flowrate,
                    absolute_roughness,
                    fluid,
                );
                last = Some((s, r));
                if r <= target_pressure_drop_per_meter {
                    return Ok((s, flowrate / s.area(), r));
                }
            }
            last.ok_or("no round standard sections")?
        }
        Shape::Rectangular => {
            let mut last: Option<(CrossSection, f64)> = None;
            for (w, h) in STANDARD_RECTANGULAR_DUCT_SIZES {
                let s = CrossSection::Rectangular(Rectangular::new(
                    f64::from(w) / 1000.0,
                    f64::from(h) / 1000.0,
                )?);
                let r = dp_per_m(
                    s.area(),
                    s.hydraulic_diameter(),
                    flowrate,
                    absolute_roughness,
                    fluid,
                );
                last = Some((s, r));
                if r <= target_pressure_drop_per_meter {
                    return Ok((s, flowrate / s.area(), r));
                }
            }
            last.ok_or("no rectangular standard sections")?
        }
    };
    Ok((section, flowrate / section.area(), r))
}

/// Size a duct so total pressure drop across `length` <= `budget_pa`.
pub fn pressure_drop_budget(
    flowrate: f64,
    length: f64,
    budget_pa: f64,
    shape: Shape,
    absolute_roughness: f64,
    fluid: &Fluid,
) -> Result<(CrossSection, f64, f64)> {
    if length <= 0.0 {
        return Err((format!("length must be positive, got {length}")).into());
    }
    if budget_pa <= 0.0 {
        return Err((format!("budget_pa must be positive, got {budget_pa}")).into());
    }
    equal_friction_method(
        flowrate,
        budget_pa / length,
        shape,
        absolute_roughness,
        fluid,
    )
}

/// Noise-limit sizing: velocity constrained by the NC target for `space_type`.
pub fn noise_limit_method(
    flowrate: f64,
    space_type: &str,
    shape: Shape,
    absolute_roughness: f64,
    fluid: &Fluid,
) -> Result<(CrossSection, f64)> {
    let target = noise_limit(space_type)?;
    velocity_method(flowrate, shape, target, absolute_roughness, fluid)
}

// ---------------------------------------------------------------------------
// Mojo-parity round / rectangular kernels
// ---------------------------------------------------------------------------

/// Smallest EN-standard round duct whose velocity <= `target_velocity`.
///
/// Returns `(section, actual_velocity)`. If no standard size meets the
/// target, the largest size is returned with its velocity.
///
/// # Examples
/// ```
/// use venti::velocity_method_round;
/// let (section, v) = velocity_method_round(0.1, 4.0).unwrap();
/// assert!(v <= 4.0); // chunk of the target velocity
/// let _ = section;
/// ```
pub fn velocity_method_round(flowrate: f64, target_velocity: f64) -> Result<(CrossSection, f64)> {
    if flowrate <= 0.0 {
        return Err("flowrate must be positive".into());
    }
    if target_velocity <= 0.0 {
        return Err("target_velocity must be positive".into());
    }
    let sizes = &STANDARD_ROUND_DUCT_SIZES;
    let n = sizes.len();
    let last = Round::new(f64::from(sizes[n - 1]) * 0.001)?;
    let mut last_v = flowrate / last.area;

    for d in sizes {
        let section = Round::new(f64::from(*d) * 0.001)?;
        let v = flowrate / section.area;
        if v <= target_velocity {
            return Ok((CrossSection::Round(section), v));
        }
        last_v = v;
    }
    Ok((CrossSection::Round(last), last_v))
}

/// Smallest EN-standard round duct with linear ΔP ≤ target.
///
/// Returns `(section, velocity_m_s, pressure_drop_per_meter)`.
pub fn equal_friction_method_round(
    flowrate: f64,
    target_pressure_drop_per_meter: f64,
    absolute_roughness: f64,
    fluid: &Fluid,
) -> Result<(CrossSection, f64, f64)> {
    if flowrate <= 0.0 {
        return Err("flowrate must be positive".into());
    }
    if target_pressure_drop_per_meter <= 0.0 {
        return Err("target_pressure_drop_per_meter must be positive".into());
    }
    let f = if fluid.density > 0.0 {
        fluid
    } else {
        &STANDARD_AIR
    };
    let sizes = &STANDARD_ROUND_DUCT_SIZES;
    let n = sizes.len();
    let last = Round::new(f64::from(sizes[n - 1]) * 0.001)?;
    let last_r = dp_per_m(
        last.area,
        last.hydraulic_diameter,
        flowrate,
        absolute_roughness,
        f,
    );

    for d in sizes {
        let section = Round::new(f64::from(*d) * 0.001)?;
        let r = dp_per_m(
            section.area,
            section.hydraulic_diameter,
            flowrate,
            absolute_roughness,
            f,
        );
        if r <= target_pressure_drop_per_meter {
            return Ok((CrossSection::Round(section), flowrate / section.area, r));
        }
    }
    Ok((CrossSection::Round(last), flowrate / last.area, last_r))
}

/// Size a round duct so total ΔP across `length` ≤ `budget_pa`.
pub fn pressure_drop_budget_round(
    flowrate: f64,
    length: f64,
    budget_pa: f64,
    absolute_roughness: f64,
    fluid: &Fluid,
) -> Result<(CrossSection, f64, f64)> {
    if length <= 0.0 {
        return Err("length must be positive".into());
    }
    if budget_pa <= 0.0 {
        return Err("budget_pa must be positive".into());
    }
    equal_friction_method_round(flowrate, budget_pa / length, absolute_roughness, fluid)
}

/// Size a rectangular duct so total ΔP across `length` ≤ `budget_pa`.
pub fn pressure_drop_budget_rectangular(
    flowrate: f64,
    length: f64,
    budget_pa: f64,
    absolute_roughness: f64,
    fluid: &Fluid,
) -> Result<(CrossSection, f64, f64)> {
    if length <= 0.0 {
        return Err("length must be positive".into());
    }
    if budget_pa <= 0.0 {
        return Err("budget_pa must be positive".into());
    }
    equal_friction_method_rectangular(flowrate, budget_pa / length, absolute_roughness, fluid)
}

/// Smallest EN-standard rectangular duct whose velocity ≤ `target_velocity`.
pub fn velocity_method_rectangular(
    flowrate: f64,
    target_velocity: f64,
) -> Result<(CrossSection, f64)> {
    if flowrate <= 0.0 {
        return Err("flowrate must be positive".into());
    }
    if target_velocity <= 0.0 {
        return Err("target_velocity must be positive".into());
    }
    let sizes = &STANDARD_RECTANGULAR_DUCT_SIZES;
    let n = sizes.len();
    let (wl, hl) = sizes[n - 1];
    let last = Rectangular::new(f64::from(wl) * 0.001, f64::from(hl) * 0.001)?;
    let mut last_v = flowrate / last.area;

    for (w, h) in sizes {
        let section = Rectangular::new(f64::from(*w) * 0.001, f64::from(*h) * 0.001)?;
        let v = flowrate / section.area;
        if v <= target_velocity {
            return Ok((CrossSection::Rectangular(section), v));
        }
        last_v = v;
    }
    Ok((CrossSection::Rectangular(last), last_v))
}

/// Smallest EN-standard rectangular duct with linear ΔP ≤ target.
pub fn equal_friction_method_rectangular(
    flowrate: f64,
    target_pressure_drop_per_meter: f64,
    absolute_roughness: f64,
    fluid: &Fluid,
) -> Result<(CrossSection, f64, f64)> {
    if flowrate <= 0.0 {
        return Err("flowrate must be positive".into());
    }
    if target_pressure_drop_per_meter <= 0.0 {
        return Err("target_pressure_drop_per_meter must be positive".into());
    }
    let f = if fluid.density > 0.0 {
        fluid
    } else {
        &STANDARD_AIR
    };
    let sizes = &STANDARD_RECTANGULAR_DUCT_SIZES;
    let n = sizes.len();
    let (wl, hl) = sizes[n - 1];
    let last = Rectangular::new(f64::from(wl) * 0.001, f64::from(hl) * 0.001)?;
    let last_r = dp_per_m(
        last.area,
        last.hydraulic_diameter,
        flowrate,
        absolute_roughness,
        f,
    );

    for (w, h) in sizes {
        let section = Rectangular::new(f64::from(*w) * 0.001, f64::from(*h) * 0.001)?;
        let r = dp_per_m(
            section.area,
            section.hydraulic_diameter,
            flowrate,
            absolute_roughness,
            f,
        );
        if r <= target_pressure_drop_per_meter {
            return Ok((
                CrossSection::Rectangular(section),
                flowrate / section.area,
                r,
            ));
        }
    }
    Ok((
        CrossSection::Rectangular(last),
        flowrate / last.area,
        last_r,
    ))
}

/// Size a rectangular duct at a target velocity and minimum aspect ratio.
pub fn aspect_ratio_method(
    flowrate: f64,
    target_velocity: f64,
    aspect_ratio: f64,
) -> Result<(CrossSection, f64)> {
    if flowrate <= 0.0 {
        return Err("flowrate must be positive".into());
    }
    if target_velocity <= 0.0 {
        return Err("target_velocity must be positive".into());
    }
    if aspect_ratio < 1.0 {
        return Err("aspect_ratio must be >= 1".into());
    }

    // Gather qualifying (w, h) pairs, sort by area ascending.
    let mut qualifying: Vec<Rectangular> = Vec::new();
    for (w, h) in STANDARD_RECTANGULAR_DUCT_SIZES.iter() {
        let long = f64::max(f64::from(*w), f64::from(*h));
        let short = f64::min(f64::from(*w), f64::from(*h));
        if long / short >= aspect_ratio {
            let wm = f64::from(*w) * 0.001;
            let hm = f64::from(*h) * 0.001;
            qualifying.push(Rectangular::new(wm, hm)?);
        }
    }
    if qualifying.is_empty() {
        return Err("no standard rectangular size meets the aspect_ratio".into());
    }
    qualifying.sort_by(|a, b| a.area.partial_cmp(&b.area).unwrap());

    let n = qualifying.len();
    let last_section = qualifying[n - 1];
    let mut last_v = flowrate / last_section.area;
    for s in &qualifying {
        let v = flowrate / s.area;
        if v <= target_velocity {
            return Ok((CrossSection::Rectangular(*s), v));
        }
        last_v = v;
    }
    Ok((CrossSection::Rectangular(last_section), last_v))
}

/// Batch-size N round ducts for an array of flowrates.
///
/// Returns `(diameters_mm, velocities)` vectors — one pass over the standard
/// sizes per flowrate, mirroring the Mojo `velocity_method_round_batch`.
pub fn velocity_method_batch<'a>(
    flowrates: impl IntoIterator<Item = &'a f64>,
    target_velocity: f64,
) -> Result<(Vec<f64>, Vec<f64>)> {
    let mut diameters = Vec::new();
    let mut velocities = Vec::new();
    for &q in flowrates {
        let (section, v) = velocity_method_round(q, target_velocity)?;
        let d = match section {
            CrossSection::Round(r) => r.diameter * 1000.0, // -> mm
            CrossSection::Rectangular(_) => 0.0,
        };
        diameters.push(d);
        velocities.push(v);
    }
    Ok((diameters, velocities))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn velocity_method_round_basic() {
        // 0.1 m^3/s at target 4 m/s -> smallest round duct with v <= 4.
        let (section, v) = velocity_method_round(0.1, 4.0).unwrap();
        assert!(v <= 4.0);
        if let CrossSection::Round(r) = section {
            // area = pi*(d/2)^2, v = q/area <= 4 -> d >= sqrt(4q/(pi*4))
            let min_d = (4.0 * 0.1 / (std::f64::consts::PI * 4.0)).sqrt();
            assert!(r.diameter >= min_d - 0.001);
        } else {
            panic!("expected round");
        }
    }

    #[test]
    fn equal_friction_round_basic() {
        let (section, v, r) = equal_friction_method_round(0.1, 1.0, 0.0001, &STANDARD_AIR).unwrap();
        assert!(r <= 1.0, "r = {r}");
        assert!(v > 0.0);
        assert!(matches!(section, CrossSection::Round(_)));
    }

    #[test]
    fn aspect_ratio_filters() {
        let (section, v) = aspect_ratio_method(0.1, 4.0, 2.0).unwrap();
        if let CrossSection::Rectangular(r) = section {
            let long = f64::max(r.width, r.height);
            let short = f64::min(r.width, r.height);
            assert!(long / short >= 2.0);
        } else {
            panic!("expected rectangular");
        }
        assert!(v <= 4.0);
    }

    #[test]
    fn noise_limit_resolves_velocity() {
        let target = noise_limit("office").unwrap();
        assert_eq!(target, 4.0);
        assert!(noise_limit("bogus").is_err());
    }

    #[test]
    fn velocity_method_batch_matches_loops() {
        let flows = [0.05, 0.1, 0.3, 0.8];
        let (ds, vs) = velocity_method_batch(flows.iter(), 4.0).unwrap();
        assert_eq!(ds.len(), 4);
        for (i, &q) in flows.iter().enumerate() {
            let (sec, v) = velocity_method_round(q, 4.0).unwrap();
            let d = match sec {
                CrossSection::Round(r) => r.diameter * 1000.0,
                CrossSection::Rectangular(_) => 0.0,
            };
            assert!((ds[i] - d).abs() < 1e-9);
            assert!((vs[i] - v).abs() < 1e-9);
        }
    }
}
