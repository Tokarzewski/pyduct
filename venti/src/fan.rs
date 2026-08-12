//! Fan selection: vendor fan curves (wentylatory) and duty-point picking.
//!
//! A ductwork system is designed for a target **duty point**: a design flow
//! (m³/s) and the static pressure the fan must develop at that flow (Pa) to
//! push air through the sized network. This module models a fan by its
//! **static pressure curve** — the static pressure the fan delivers as a
//! function of flow, tabulated as a polyline of points copied from a vendor
//! catalogue. Selection is the classic "fan curve vs. system curve"
//! procedure, inverted: instead of plotting the fan curve against the system
//! curve, we ask which fan, at the design flow, still delivers **at least**
//! the required static pressure (a non-negative pressure margin).
//!
//! All quantities are SI: flow in m³/s, static pressure in Pa, shaft power in
//! W.
//!
//! This module is **dependency-free** — pure `f64` math on standard Rust.

use crate::Result;

/// A single point on a vendor fan curve.
///
/// `flow_m3s` is the volumetric flow through the fan [m³/s] and
/// `static_pressure_pa` is the static pressure the fan develops at that
/// flow [Pa].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FanPoint {
    pub flow_m3s: f64,
    pub static_pressure_pa: f64,
}

/// A fan pressure curve: a polyline through [`FanPoint`]s tabulated from a
/// vendor catalogue.
///
/// The curve is stored as an ordered list of `(flow, static pressure)`
/// points with **strictly increasing** flow (so a flow unambiguously maps to
/// a curve segment). The pressure delivered at an arbitrary flow is obtained
/// by piecewise-linear interpolation between the bracketing points; outside
/// the tabulated flow range the curve is undefined and queries error.
#[derive(Debug, Clone)]
pub struct FanCurve {
    pub name: String,
    pub points: Vec<FanPoint>,
}

impl FanCurve {
    /// Build a fan curve from its name and tabulated points.
    ///
    /// # Errors
    ///
    /// Returns an error when:
    ///
    /// * fewer than 2 points are given (a curve needs at least a segment),
    /// * flows are not strictly increasing (equal or decreasing flow makes
    ///   the flow → pressure mapping ambiguous), or
    /// * any value is non-finite (NaN / infinity breaks interpolation).
    pub fn new(name: &str, points: Vec<FanPoint>) -> Result<Self> {
        if points.len() < 2 {
            return Err(format!(
                "fan curve '{name}' needs at least 2 points, got {}",
                points.len()
            )
            .into());
        }
        for w in points.windows(2) {
            let (a, b) = (&w[0], &w[1]);
            if !a.flow_m3s.is_finite()
                || !a.static_pressure_pa.is_finite()
                || !b.flow_m3s.is_finite()
                || !b.static_pressure_pa.is_finite()
            {
                return Err(format!("fan curve '{name}' contains a non-finite point").into());
            }
            if a.flow_m3s >= b.flow_m3s {
                return Err(format!(
                    "fan curve '{name}' flows must be strictly increasing \
                     ({} m³/s then {} m³/s)",
                    a.flow_m3s, b.flow_m3s
                )
                .into());
            }
        }
        Ok(Self {
            name: name.to_string(),
            points,
        })
    }

    /// Static pressure [Pa] delivered by the fan at `flow_m3s` [m³/s].
    ///
    /// The vendor curve is interpolated piecewise-linearly between the two
    /// points bracketing the requested flow.
    ///
    /// # Examples
    ///
    /// ```
    /// use venti::{FanCurve, FanPoint};
    ///
    /// // Vendor points (flow m³/s, static pressure Pa) for a small duct fan.
    /// let fan = FanCurve::new(
    ///     "TD-350",
    ///     vec![
    ///         FanPoint { flow_m3s: 0.00, static_pressure_pa: 220.0 },
    ///         FanPoint { flow_m3s: 0.10, static_pressure_pa: 150.0 },
    ///         FanPoint { flow_m3s: 0.20, static_pressure_pa: 0.0 },
    ///     ],
    /// )
    /// .unwrap();
    ///
    /// // Exact at the knots …
    /// assert!((fan.static_pressure_at(0.10).unwrap() - 150.0).abs() < 1e-9);
    /// // … and linear at the midpoint between them:
    /// assert!((fan.static_pressure_at(0.15).unwrap() - 75.0).abs() < 1e-9);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error when `flow_m3s` lies outside the tabulated curve
    /// range (the catalogue data is not defined beyond its measured points),
    /// or when `flow_m3s` is not finite.
    pub fn static_pressure_at(&self, flow_m3s: f64) -> Result<f64> {
        if !flow_m3s.is_finite() {
            return Err(format!("fan '{}': flow {flow_m3s} m³/s is not finite", self.name).into());
        }
        let first = &self.points[0];
        let last = &self.points[self.points.len() - 1];
        if flow_m3s < first.flow_m3s || flow_m3s > last.flow_m3s {
            return Err(format!(
                "fan '{}': flow {flow_m3s} m³/s outside curve range \
                 [{} m³/s … {} m³/s]",
                self.name, first.flow_m3s, last.flow_m3s
            )
            .into());
        }
        // `flow_m3s` is finite and inside [first, last], so exactly one
        // segment brackets it (including the knot values on its boundaries).
        for w in self.points.windows(2) {
            let (a, b) = (&w[0], &w[1]);
            if flow_m3s >= a.flow_m3s && flow_m3s <= b.flow_m3s {
                let t = (flow_m3s - a.flow_m3s) / (b.flow_m3s - a.flow_m3s);
                return Ok(a.static_pressure_pa + t * (b.static_pressure_pa - a.static_pressure_pa));
            }
        }
        // Defensive: unreachable for finite in-range flows on a validated curve.
        Err(format!(
            "fan '{}': no segment brackets flow {flow_m3s} m³/s",
            self.name
        )
        .into())
    }
}

/// Shaft power [W] the fan must develop for a duty point (fan power input
/// to the fluid, sometimes called "air power" before motor/impeller
/// efficiencies are folded in).
///
/// Closed form:
///
/// ```text
/// P = Q · p / η
/// ```
///
/// with `Q` = flow [m³/s], `p` = static pressure [Pa] and `η` = the total
/// (impeller × motor) efficiency in `(0, 1]`.
///
/// # Examples
///
/// ```
/// use venti::fan_power;
///
/// // 0.10 m³/s against 500 Pa at 60 % efficiency:
/// let p = fan_power(0.10, 500.0, 0.60).unwrap();
/// assert!((p - 83.333).abs() < 0.001);
/// ```
///
/// # Errors
///
/// Returns an error when `efficiency` is outside `(0, 1]`, when flow or
/// pressure are negative, or when any input is non-finite.
pub fn fan_power(flow_m3s: f64, pressure_pa: f64, efficiency: f64) -> Result<f64> {
    if !flow_m3s.is_finite() || !pressure_pa.is_finite() || !efficiency.is_finite() {
        return Err("fan_power: flow, pressure and efficiency must be finite".into());
    }
    if flow_m3s < 0.0 {
        return Err(format!("fan_power: flow must be >= 0 m³/s, got {flow_m3s}").into());
    }
    if pressure_pa < 0.0 {
        return Err(format!("fan_power: pressure must be >= 0 Pa, got {pressure_pa}").into());
    }
    if efficiency <= 0.0 || efficiency > 1.0 {
        return Err(format!("fan_power: efficiency must be in (0, 1], got {efficiency}").into());
    }
    Ok(flow_m3s * pressure_pa / efficiency)
}

/// Pressure margin [Pa] of `fan` at the design flow: the static pressure the
/// fan's curve delivers at `design_flow_m3s` minus `required_static_pa`.
///
/// A positive margin means the fan beats the requirement at the duty point;
/// zero means it delivers exactly the required pressure; a negative margin
/// means it falls short. Returns `Ok(None)` when `design_flow_m3s` is
/// outside the fan's tabulated curve range — the margin is undefined there,
/// so that fan cannot be compared at this duty point.
///
/// # Errors
///
/// Returns an error for a negative or non-finite `design_flow_m3s` or
/// `required_static_pa`.
pub fn margin_pa(
    fan: &FanCurve,
    design_flow_m3s: f64,
    required_static_pa: f64,
) -> Result<Option<f64>> {
    if !design_flow_m3s.is_finite() || design_flow_m3s < 0.0 {
        return Err(format!(
            "margin_pa: design flow must be a finite value >= 0 m³/s, got {design_flow_m3s}"
        )
        .into());
    }
    if !required_static_pa.is_finite() || required_static_pa < 0.0 {
        return Err(format!(
            "margin_pa: required pressure must be a finite value >= 0 Pa, \
             got {required_static_pa}"
        )
        .into());
    }
    Ok(fan
        .static_pressure_at(design_flow_m3s)
        .ok()
        .map(|curve_pressure| curve_pressure - required_static_pa))
}

/// Pick the first fan from `curves` that delivers the required static
/// pressure at the design flow.
///
/// For each fan in catalogue order the curve static pressure at
/// `design_flow_m3s` is compared with `required_static_pa`; the index of the
/// first fan whose margin (`curve pressure − required`) is `>= 0` is
/// returned. Fans whose curve does not cover the design flow (margin
/// undefined) are skipped, as are fans that fall short. `Ok(None)` means no
/// fan in the catalogue meets the duty point.
///
/// # Errors
///
/// Returns an error for a negative or non-finite `design_flow_m3s` or
/// `required_static_pa`.
pub fn pick_fan(
    curves: &[FanCurve],
    design_flow_m3s: f64,
    required_static_pa: f64,
) -> Result<Option<usize>> {
    for (i, fan) in curves.iter().enumerate() {
        if let Some(margin) = margin_pa(fan, design_flow_m3s, required_static_pa)? {
            if margin >= 0.0 {
                return Ok(Some(i));
            }
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn point(flow_m3s: f64, static_pressure_pa: f64) -> FanPoint {
        FanPoint {
            flow_m3s,
            static_pressure_pa,
        }
    }

    /// (0, 300) → (0.1, 250) → (0.25, 100) → (0.4, 0) Pa.
    fn sample_curve() -> FanCurve {
        FanCurve::new(
            "sample",
            vec![
                point(0.0, 300.0),
                point(0.1, 250.0),
                point(0.25, 100.0),
                point(0.4, 0.0),
            ],
        )
        .unwrap()
    }

    #[test]
    fn interpolation_is_exact_at_knots() {
        let fan = sample_curve();
        for p in &fan.points {
            let got = fan.static_pressure_at(p.flow_m3s).unwrap();
            assert!((got - p.static_pressure_pa).abs() < 1e-12);
        }
    }

    #[test]
    fn interpolation_is_linear_at_midpoints() {
        let fan = sample_curve();
        // Between (0.0, 300) and (0.1, 250) at halfway.
        assert!((fan.static_pressure_at(0.05).unwrap() - 275.0).abs() < 1e-9);
        // Between (0.1, 250) and (0.25, 100) at the quarter point (t = 0.25).
        assert!((fan.static_pressure_at(0.1375).unwrap() - 212.5).abs() < 1e-9);
        // Between (0.25, 100) and (0.4, 0) at halfway.
        assert!((fan.static_pressure_at(0.325).unwrap() - 50.0).abs() < 1e-9);
    }

    #[test]
    fn interpolation_outside_range_is_error() {
        let fan = sample_curve();
        assert!(fan.static_pressure_at(-0.01).is_err());
        assert!(fan.static_pressure_at(0.41).is_err());
        assert!(fan.static_pressure_at(f64::NAN).is_err());
        // Boundaries are still valid.
        assert!(fan.static_pressure_at(0.0).is_ok());
        assert!(fan.static_pressure_at(0.4).is_ok());
    }

    #[test]
    fn fan_power_closed_form() {
        // P = Q·p/η = 1.5 · 800 / 0.4 = 3000 W.
        assert!((fan_power(1.5, 800.0, 0.4).unwrap() - 3000.0).abs() < 1e-9);
        // Zero flow → zero power regardless of pressure.
        assert!((fan_power(0.0, 800.0, 0.4).unwrap() - 0.0).abs() < 1e-12);
        // η = 1 is the perfect, loss-free lower bound P = Q·p.
        assert!((fan_power(1.5, 800.0, 1.0).unwrap() - 1200.0).abs() < 1e-9);
    }

    #[test]
    fn fan_power_rejects_invalid_inputs() {
        assert!(fan_power(1.0, 100.0, 0.0).is_err()); // η = 0
        assert!(fan_power(1.0, 100.0, -0.1).is_err()); // η < 0
        assert!(fan_power(1.0, 100.0, 1.5).is_err()); // η > 1
        assert!(fan_power(-1.0, 100.0, 0.5).is_err()); // negative flow
        assert!(fan_power(1.0, -100.0, 0.5).is_err()); // negative pressure
    }

    #[test]
    fn pick_fan_selects_first_adequate_fan() {
        let weak = FanCurve::new("weak", vec![point(0.0, 300.0), point(0.5, 150.0)]).unwrap();
        let strong = FanCurve::new("strong", vec![point(0.0, 500.0), point(0.5, 400.0)]).unwrap();
        let mid = FanCurve::new("mid", vec![point(0.0, 450.0), point(0.5, 300.0)]).unwrap();

        // Required 250 Pa at 0.3 m³/s: weak delivers 210, mid 330 → mid wins.
        let curves = [weak.clone(), mid.clone(), strong.clone()];
        let idx = pick_fan(&curves, 0.3, 250.0).unwrap();
        assert_eq!(idx, Some(1));

        // Required 200 Pa at 0.3 m³/s: weak delivers 210 → first fan wins.
        assert_eq!(pick_fan(&curves, 0.3, 200.0).unwrap(), Some(0));

        // Exact match (margin 0) still counts: 330 Pa at 0.3 m³/s → mid.
        assert_eq!(pick_fan(&curves, 0.3, 330.0).unwrap(), Some(1));
    }

    #[test]
    fn pick_fan_returns_none_when_nothing_meets_duty() {
        let small = FanCurve::new("small", vec![point(0.0, 200.0), point(0.3, 100.0)]).unwrap();
        let curves = [small];
        // Required far above anything on any curve.
        assert_eq!(pick_fan(&curves, 0.15, 5000.0).unwrap(), None);
        // Empty catalogue.
        assert_eq!(pick_fan(&[], 0.15, 100.0).unwrap(), None);
    }

    #[test]
    fn pick_fan_skips_fans_outside_their_curve_range() {
        // Narrow fan only tabulated up to 0.2 m³/s; design flow 0.4 m³/s is
        // outside its range, so it cannot be compared and must be skipped
        // in favour of the wide fan.
        let narrow = FanCurve::new("narrow", vec![point(0.0, 400.0), point(0.2, 100.0)]).unwrap();
        let wide = FanCurve::new("wide", vec![point(0.0, 400.0), point(0.6, 250.0)]).unwrap();
        let idx = pick_fan(&[narrow, wide], 0.4, 300.0).unwrap();
        assert_eq!(idx, Some(1));
    }

    #[test]
    fn margin_sign_and_undefined_range() {
        let fan = sample_curve();
        // 0.3 m³/s → 66.667 Pa on the curve (t = ⅓ on the last segment):
        // 66.667 − 40 = +26.667 Pa margin.
        let curve_at_03 = fan.static_pressure_at(0.3).unwrap();
        assert!((curve_at_03 - 200.0 / 3.0).abs() < 1e-9);
        assert!((margin_pa(&fan, 0.3, 40.0).unwrap().unwrap() - 26.66666666).abs() < 1e-7);
        // Below requirement → negative margin.
        assert!(margin_pa(&fan, 0.3, 100.0).unwrap().unwrap() < 0.0);
        // Exact requirement → zero margin.
        assert!(margin_pa(&fan, 0.3, 200.0 / 3.0).unwrap().unwrap().abs() < 1e-9);
        // Outside curve range → margin undefined (None, not an error).
        assert_eq!(margin_pa(&fan, 0.9, 40.0).unwrap(), None);
        // Invalid arguments are errors.
        assert!(margin_pa(&fan, -0.1, 40.0).is_err());
        assert!(margin_pa(&fan, 0.3, -1.0).is_err());
    }

    #[test]
    fn fan_curve_new_validation() {
        // Fewer than 2 points.
        assert!(FanCurve::new("one", vec![point(0.1, 100.0)]).is_err());
        // Equal flows.
        assert!(FanCurve::new("dup", vec![point(0.1, 100.0), point(0.1, 90.0)]).is_err());
        // Decreasing flow.
        assert!(FanCurve::new("down", vec![point(0.2, 100.0), point(0.1, 90.0)]).is_err());
        // Non-finite values.
        assert!(FanCurve::new("nan", vec![point(0.1, f64::NAN), point(0.2, 90.0)]).is_err());
        assert!(FanCurve::new("inf", vec![point(f64::INFINITY, 100.0), point(0.2, 90.0)]).is_err());
        // Valid curve still builds.
        assert!(FanCurve::new("ok", vec![point(0.0, 0.0), point(0.5, 100.0)]).is_ok());
    }
}
