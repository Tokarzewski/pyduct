//! Loss-coefficient correlations for common HVAC fittings.
//!
//! Mirrors `wentamojo/components/fittings_library.mojo` and the Python
//! `wenta.components.fittings_library`. Coefficients from ASHRAE Fundamentals
//! and ductwork design guides (Hendiger, Idelchik).

use crate::Result;
/// Smooth-radius rectangular elbow (Idelchik §6) with aspect correction.
pub fn rectangular_elbow(width: f64, height: f64, bend_radius: f64, angle_deg: f64) -> Result<f64> {
    let smallest = width.min(height);
    if smallest <= 0.0 || bend_radius <= 0.0 {
        return Err("width, height and bend_radius must be positive".into());
    }
    if angle_deg <= 0.0 || angle_deg > 180.0 {
        return Err("angle_deg must be in (0, 180]".into());
    }
    let r_over_w = bend_radius / width;
    let floor = r_over_w.max(0.1);
    let mut zeta_90 = 0.21 / floor.sqrt();
    if zeta_90 > 1.5 {
        zeta_90 = 1.5;
    }
    let aspect_correction = (height / width).powf(0.25);
    Ok(zeta_90 * aspect_correction * (angle_deg / 90.0))
}

/// Round reducer loss coefficient (ASHRAE/Swamee–Jain style), referenced to
/// the outlet velocity.
pub fn reducer_round(d_inlet: f64, d_outlet: f64, angle_deg: f64) -> Result<f64> {
    if d_outlet > d_inlet {
        return Err("outlet diameter must be <= inlet".into());
    }
    if d_outlet <= 0.0 {
        return Err("outlet diameter must be positive".into());
    }
    let area_ratio = (d_outlet / d_inlet).powi(2);
    let zeta = 0.04 + 0.37 * (1.0 - area_ratio);
    let angle_factor = if angle_deg < 45.0 {
        0.8 + 0.004 * (45.0 - angle_deg)
    } else {
        1.0
    };
    Ok(zeta * angle_factor)
}

/// Round expander / diffuser loss coefficient (Borda–Carnot baseline),
/// referenced to the inlet velocity.
pub fn expander_round(d_inlet: f64, d_outlet: f64, angle_deg: f64) -> Result<f64> {
    if d_inlet > d_outlet {
        return Err("inlet diameter must be <= outlet".into());
    }
    if d_inlet <= 0.0 {
        return Err("inlet diameter must be positive".into());
    }
    let area_ratio = (d_inlet / d_outlet).powi(2);
    let zeta_sudden = (1.0 - area_ratio).powi(2);
    let diffuser_factor = if angle_deg <= 10.0 {
        0.5
    } else if angle_deg <= 20.0 {
        0.6
    } else if angle_deg <= 45.0 {
        0.8
    } else {
        1.0
    };
    Ok(diffuser_factor * zeta_sudden)
}

/// Splitting-tee loss coefficients `(zeta_main, zeta_branch)`.
pub fn junction_tee_branch(
    d_main: f64,
    d_branch: f64,
    flowrate_main: f64,
    flowrate_branch: f64,
) -> Result<(f64, f64)> {
    if flowrate_main < 0.0 || flowrate_branch < 0.0 {
        return Err("flowrates must be non-negative".into());
    }
    let total = flowrate_main + flowrate_branch;
    if total <= 0.0 {
        return Err("at least one flowrate must be positive".into());
    }
    let split = flowrate_branch / total;
    let area = if d_main > 0.0 {
        (d_branch / d_main).powi(2)
    } else {
        0.0
    };
    Ok((
        0.08 * split + 0.05 * area,
        0.3 + 0.5 * (1.0 - area) + 0.4 * split,
    ))
}

/// Combining-tee loss coefficients `(zeta_main, zeta_branch)`.
pub fn junction_tee_combine(
    d_main: f64,
    d_branch: f64,
    flowrate_main: f64,
    flowrate_branch: f64,
) -> Result<(f64, f64)> {
    let total = flowrate_main + flowrate_branch;
    if total <= 0.0 {
        return Err("at least one flowrate must be positive".into());
    }
    let split = flowrate_branch / total;
    let area = if d_main > 0.0 {
        (d_branch / d_main).powi(2)
    } else {
        0.0
    };
    Ok((
        0.1 + 0.15 * split + 0.08 * area,
        0.4 + 0.6 * (1.0 - area) + 0.3 * split,
    ))
}

/// Butterfly-damper loss coefficient (~0.1 fully open, rises steeply closed).
pub fn damper_butterfly(open_percentage: f64) -> Result<f64> {
    if !(0.0..=100.0).contains(&open_percentage) {
        return Err("open_percentage must be in [0, 100]".into());
    }
    if open_percentage >= 95.0 {
        return Ok(0.1);
    }
    let closed_frac = 1.0 - open_percentage / 100.0;
    Ok(0.1 + closed_frac * closed_frac * 10.0)
}

/// Ceiling-diffuser face-velocity loss coefficient.
pub fn diffuser_ceiling(area_throw: f64) -> Result<f64> {
    if area_throw <= 0.0 {
        return Err("area_throw must be positive".into());
    }
    Ok(0.4 / area_throw)
}

/// Return-grille face-velocity loss coefficient.
pub fn grille_return(blockage_factor: f64) -> Result<f64> {
    if !(0.0..=1.0).contains(&blockage_factor) {
        return Err("blockage_factor must be in [0, 1]".into());
    }
    Ok(0.25 * (1.0 + blockage_factor))
}

/// Sharp-corner mitered elbow loss coefficient; `vaned=True` cuts it to ~40 %.
pub fn mitered_elbow(angle_deg: f64, vaned: bool) -> Result<f64> {
    if angle_deg <= 0.0 || angle_deg > 180.0 {
        return Err("angle_deg must be in (0, 180]".into());
    }
    let a = angle_deg / 90.0;
    let zeta_unvaned = 0.55 * a + 0.65 * a * a;
    Ok(zeta_unvaned * if vaned { 0.4 } else { 1.0 })
}

/// Two-port taper transition (ASHRAE F23 / Idelchik §4). Blends reducer and
/// expander behaviour: when `d_outlet > d_inlet` it is a diffuser (Borda–Carnot
/// baseline referenced to the inlet velocity), otherwise a reducer (contraction
/// referenced to the outlet velocity). All loss is driven by the inlet/outlet
/// area ratio and the included (cone) angle; smaller included angles give a
/// longer, smoother transition and lower loss.
pub fn taper_transition(d_inlet: f64, d_outlet: f64, angle_deg: f64) -> Result<f64> {
    if d_inlet <= 0.0 || d_outlet <= 0.0 {
        return Err("d_inlet and d_outlet must be positive".into());
    }
    if angle_deg <= 0.0 || angle_deg > 90.0 {
        return Err("angle_deg must be in (0, 90]".into());
    }
    // Smaller included angle = gentler, longer cone = smoother flow = less loss.
    let angle_factor = if angle_deg <= 10.0 {
        0.25
    } else if angle_deg <= 20.0 {
        0.4
    } else if angle_deg <= 45.0 {
        0.6
    } else {
        1.0
    };
    let area_ratio = (d_outlet / d_inlet).powi(2);
    if d_outlet >= d_inlet {
        // Expander / diffuser, referenced to the inlet velocity.
        let zeta_borda = (1.0 - 1.0 / area_ratio).powi(2);
        Ok(angle_factor * zeta_borda)
    } else {
        // Reducer / contraction, referenced to the outlet velocity (ASHRAE).
        let zeta_contraction = 0.05 + 0.5 * (1.0 - area_ratio);
        Ok(angle_factor * zeta_contraction)
    }
}

/// 4-way cross junction loss coefficients `(zeta_main, zeta_branch)` (ASHRAE
/// F25 / Miller). `flow_ratio` is the fraction of the total flow that leaves
/// through the branch leg (0 = straight-through only, 1 = all diverted). The
/// main/straight leg loss is modest and rises with the amount diverted, while
/// the branch leg carries a larger loss that also penalises a small branch
/// area relative to the main.
pub fn cross_fitting(d_main: f64, d_branch: f64, flow_ratio: f64) -> Result<(f64, f64)> {
    if d_main <= 0.0 || d_branch <= 0.0 {
        return Err("d_main and d_branch must be positive".into());
    }
    if !(0.0..=1.0).contains(&flow_ratio) {
        return Err("flow_ratio must be in [0, 1]".into());
    }
    let area = (d_branch / d_main).powi(2);
    let zeta_main = 0.12 + 0.3 * flow_ratio + 0.1 * area;
    let zeta_branch = 0.5 + 0.8 * (1.0 - area) + 0.5 * flow_ratio;
    Ok((zeta_main, zeta_branch))
}

/// Fire-damper housing/section loss coefficient (HVAC design guides, e.g.
/// ASHRAE/SMACNA damper data). The open housing adds a small base loss; as the
/// damper closes (`open_percentage` below ~95 %) a sharp quadratic penalty
/// captures the blade/obstruction losses.
pub fn fire_damper(open_percentage: f64) -> Result<f64> {
    if !(0.0..=100.0).contains(&open_percentage) {
        return Err("open_percentage must be in [0, 100]".into());
    }
    const BASE: f64 = 0.18; // typical fully-open fire-damper section zeta
    if open_percentage >= 95.0 {
        return Ok(BASE);
    }
    let closed_frac = 1.0 - open_percentage / 100.0;
    Ok(BASE + closed_frac * closed_frac * 30.0)
}

/// Branded fire-damper loss coefficient — catalogue-driven selection.
///
/// Looks the fully-open housing section zeta for a specific vendor brand and
/// nominal duct size (mm) up in [`crate::catalog::reference_catalog`] under
/// the key `damper.fire.<brand>.<size_mm>` (lower-cased brand), then applies
/// the same closing correlation as [`fire_damper`]: below 95 % open,
/// `zeta = base + 30·(1 − open/100)²`, otherwise the bare housing value.
/// Returns an error (a `String`-style message, `venti::Error`) if the
/// brand/size combination is not catalogued or an input is invalid.
///
/// # Examples
/// ```
/// use venti::components::fittings_library::fire_damper_branded;
/// let z = fire_damper_branded("trox", 200.0, 100.0).unwrap();
/// assert!((z - 0.20).abs() < 1e-12);
/// assert!(fire_damper_branded("acme", 200.0, 100.0).is_err());
/// ```
pub fn fire_damper_branded(brand: &str, size_mm: f64, open_percentage: f64) -> Result<f64> {
    if size_mm <= 0.0 {
        return Err("size_mm must be positive".into());
    }
    if !(0.0..=100.0).contains(&open_percentage) {
        return Err("open_percentage must be in [0, 100]".into());
    }
    let key = format!("damper.fire.{}.{}", brand.trim().to_lowercase(), size_mm);
    let base = crate::catalog::reference_catalog()
        .lookup(&key)
        .ok_or_else(|| {
            format!(
                "no catalogue entry for fire damper: brand {brand:?} at {size_mm} mm (key {key})"
            )
        })?;
    if open_percentage >= 95.0 {
        return Ok(base);
    }
    let closed_frac = 1.0 - open_percentage / 100.0;
    Ok(base + closed_frac * closed_frac * 30.0)
}

/// Attenuator / silencer insertion loss — the pressure-loss coefficient of a
/// duct silencer section as a function of the open (free-area) fraction of its
/// perforated lining. An open silencer contributes a small base section loss;
/// as the open fraction drops, the frontal-area restriction raises the loss.
pub fn attenuator_open(open_fraction: f64) -> Result<f64> {
    if !(0.0..=1.0).contains(&open_fraction) {
        return Err("open_fraction must be in [0, 1]".into());
    }
    const BASE: f64 = 0.35; // typical fully-open silencer section zeta
    if open_fraction >= 0.95 {
        return Ok(BASE);
    }
    let closed_frac = 1.0 - open_fraction;
    Ok(BASE + closed_frac * closed_frac * 8.0)
}

/// Alias for [`attenuator_open`].
pub fn attenuator(open_fraction: f64) -> Result<f64> {
    attenuator_open(open_fraction)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rectangular_elbow_closed_form() {
        // width=0.3, height=0.2, R=0.15 -> r/W=0.5 -> zeta90=0.21/sqrt(0.5)
        let z = rectangular_elbow(0.3, 0.2, 0.15, 90.0).unwrap();
        let expected = (0.21 / 0.5f64.sqrt()) * (0.2f64 / 0.3).powf(0.25);
        assert!((z - expected).abs() < 1e-9, "z = {z}");
    }

    #[test]
    fn reducer_round_monotonic() {
        let full = reducer_round(0.4, 0.4, 45.0).unwrap(); // no reduction
        let reduced = reducer_round(0.4, 0.2, 45.0).unwrap();
        assert!(reduced > full);
    }

    #[test]
    fn damper_fully_open_is_small() {
        assert!((damper_butterfly(100.0).unwrap() - 0.1).abs() < 1e-12);
        assert!(damper_butterfly(0.0).unwrap() > 9.0);
    }

    #[test]
    fn grille_scales_with_blockage() {
        assert!((grille_return(0.0).unwrap() - 0.25).abs() < 1e-12);
        assert!((grille_return(1.0).unwrap() - 0.5).abs() < 1e-12);
    }

    #[test]
    fn mitered_vaned_cuts_loss() {
        let unvaned = mitered_elbow(90.0, false).unwrap();
        let vaned = mitered_elbow(90.0, true).unwrap();
        assert!((vaned - unvaned * 0.4).abs() < 1e-12);
    }

    #[test]
    fn tee_branch_split() {
        let (zm, zb) = junction_tee_branch(0.3, 0.2, 0.2, 0.1).unwrap();
        assert!(zb > zm);
    }

    #[test]
    fn taper_transition_reducer_expander_blend_closed_form() {
        // Equal diameters -> no transition -> zero loss for both branches.
        assert_eq!(taper_transition(0.3, 0.3, 30.0).unwrap(), 0.0);
        // Reducer: deterministic closed form at angle_deg > 45.
        let red = taper_transition(0.4, 0.2, 50.0).unwrap();
        let expected = 1.0 * (0.05 + 0.5 * (1.0 - (0.2f64 / 0.4).powi(2)));
        assert!((red - expected).abs() < 1e-12);
        // Expander: Borda-Carnot at angle_deg > 45.
        let exp = taper_transition(0.2, 0.4, 50.0).unwrap();
        let expected_exp = 1.0 * (1.0 - 1.0 / (0.4f64 / 0.2).powi(2)).powi(2);
        assert!((exp - expected_exp).abs() < 1e-12);
    }

    #[test]
    fn taper_transition_gentler_angle_is_smoother() {
        // All else equal, a smaller included angle reduces the loss.
        let steep = taper_transition(0.2, 0.4, 50.0).unwrap();
        let gentle = taper_transition(0.2, 0.4, 10.0).unwrap();
        assert!(gentle < steep);
    }

    #[test]
    fn cross_fitting_closed_form_and_monotone() {
        let (zm, zb) = cross_fitting(0.3, 0.2, 0.4).unwrap();
        let area = (0.2f64 / 0.3).powi(2);
        assert!((zm - (0.12 + 0.3 * 0.4 + 0.1 * area)).abs() < 1e-12);
        assert!((zb - (0.5 + 0.8 * (1.0 - area) + 0.5 * 0.4)).abs() < 1e-12);
        // More diverted through the branch -> higher branch loss.
        let (_, zb_hi) = cross_fitting(0.3, 0.2, 0.9).unwrap();
        assert!(zb_hi > zb);
    }

    #[test]
    fn cross_fitting_branch_costlier_than_main() {
        let (zm, zb) = cross_fitting(0.3, 0.2, 0.5).unwrap();
        assert!(zb > zm);
    }

    #[test]
    fn fire_damper_fully_open_is_small_and_closes_steeper() {
        assert!((fire_damper(100.0).unwrap() - 0.18).abs() < 1e-12);
        assert!((fire_damper(95.0).unwrap() - 0.18).abs() < 1e-12);
        let closed = fire_damper(0.0).unwrap();
        assert!(closed > 20.0);
        let half = fire_damper(50.0).unwrap();
        assert!(half > fire_damper(95.0).unwrap());
    }

    #[test]
    fn fire_damper_rejects_bad_open() {
        assert!(fire_damper(-1.0).is_err());
        assert!(fire_damper(101.0).is_err());
    }

    #[test]
    fn fire_damper_branded_uses_catalog_base() {
        // Fully open (and the 95 % threshold) returns exactly the catalogued
        // housing zeta for the brand/size — i.e. the helper is driven by the
        // ζ catalog, not a hard-coded constant.
        let base = crate::catalog::reference_catalog()
            .lookup("damper.fire.trox.200")
            .unwrap();
        assert!((fire_damper_branded("trox", 200.0, 100.0).unwrap() - base).abs() < 1e-12);
        assert!((fire_damper_branded("trox", 200.0, 95.0).unwrap() - base).abs() < 1e-12);
        // Brand matching is case-insensitive.
        let mbase = crate::catalog::reference_catalog()
            .lookup("damper.fire.mercor.315")
            .unwrap();
        assert!((fire_damper_branded("Mercor", 315.0, 100.0).unwrap() - mbase).abs() < 1e-12);
        // Different sizes carry different catalogue bases.
        let small = crate::catalog::reference_catalog()
            .lookup("damper.fire.trox.160")
            .unwrap();
        let large = crate::catalog::reference_catalog()
            .lookup("damper.fire.trox.315")
            .unwrap();
        assert!(
            small > large,
            "smaller damper should have larger housing zeta"
        );
    }

    #[test]
    fn fire_damper_branded_closing_raises_zeta() {
        // Closed-form check at 50 % open: base + 30·(1 − 0.5)² = base + 7.5.
        let base = 0.20; // damper.fire.trox.200 catalogue base
        let half = fire_damper_branded("trox", 200.0, 50.0).unwrap();
        assert!((half - (base + 30.0 * 0.5 * 0.5)).abs() < 1e-12);
        assert!(half > fire_damper_branded("trox", 200.0, 95.0).unwrap());
        assert!(fire_damper_branded("trox", 200.0, 0.0).unwrap() > 20.0);
        // Monotonic: more closed = more loss.
        let c80 = fire_damper_branded("trox", 200.0, 80.0).unwrap();
        let c50 = fire_damper_branded("trox", 200.0, 50.0).unwrap();
        assert!(c50 > c80);
    }

    #[test]
    fn fire_damper_branded_unknown_brand_or_size_errors() {
        assert!(fire_damper_branded("acme", 200.0, 100.0).is_err());
        assert!(fire_damper_branded("trox", 999.0, 100.0).is_err());
        assert!(fire_damper_branded("", 200.0, 50.0).is_err());
        // Unknown entries still error at every opening.
        assert!(fire_damper_branded("unknown", 200.0, 0.0).is_err());
    }

    #[test]
    fn fire_damper_branded_validates_inputs() {
        assert!(fire_damper_branded("trox", 0.0, 100.0).is_err());
        assert!(fire_damper_branded("trox", -5.0, 100.0).is_err());
        assert!(fire_damper_branded("trox", 200.0, -1.0).is_err());
        assert!(fire_damper_branded("trox", 200.0, 101.0).is_err());
        // A known brand with an uncatalogued size is a lookup error too.
        let e = fire_damper_branded("trox", 123.0, 100.0).unwrap_err();
        assert!(!e.to_string().is_empty());
    }

    #[test]
    fn attenuator_loses_more_as_it_closes() {
        assert!((attenuator_open(1.0).unwrap() - 0.35).abs() < 1e-12);
        assert!((attenuator_open(0.95).unwrap() - 0.35).abs() < 1e-12);
        let half = attenuator_open(0.5).unwrap();
        assert!(half > attenuator_open(1.0).unwrap());
        let shut = attenuator_open(0.0).unwrap();
        assert!(shut > half);
        // alias is identical
        assert_eq!(attenuator(0.5).unwrap(), attenuator_open(0.5).unwrap());
        assert!(attenuator_open(1.5).is_err());
        assert!(attenuator_open(-0.1).is_err());
    }
}

// ---------------------------------------------------------------------------
// Expanded library (ductwork breadth)
// ---------------------------------------------------------------------------
// Additional loss-coefficient correlations spanning rectangular transitions,
// round elbows, louvers, filters and round taps — all in the same
// `Result<f64>` + validation style, with documented sources.

/// Half-angle diffuser-performance factor (ASHRAE Fundamentals F29).
///
/// Small included angles act as efficient diffusers (low loss); the factor
/// rises toward 1.0 (the sudden-expansion Borda–Carnot limit) as the angle
/// grows. `angle_deg` is the total included cone angle, clamped to ≤ 180.
fn diffuser_factor(angle_deg: f64) -> f64 {
    if angle_deg <= 10.0 {
        0.3
    } else if angle_deg <= 20.0 {
        0.5
    } else if angle_deg <= 45.0 {
        0.7
    } else if angle_deg <= 60.0 {
        0.9
    } else {
        1.0
    }
}

/// Rectangular reducer (contraction) loss coefficient, referenced to the
/// **outlet** velocity (ASHRAE F29 / Idelchik §4).
///
/// ``zeta = (0.04 + 0.37·(1 − A_out/A_in)) · f_angle`` — smoother for smaller
/// included angles; an ideal (equal-area) transition gives `0`.
pub fn reducer_rectangular(
    w_in: f64,
    h_in: f64,
    w_out: f64,
    h_out: f64,
    angle_deg: f64,
) -> Result<f64> {
    if w_in <= 0.0 || h_in <= 0.0 || w_out <= 0.0 || h_out <= 0.0 {
        return Err("widths and heights must be positive".into());
    }
    let a_in = w_in * h_in;
    let a_out = w_out * h_out;
    if a_out > a_in {
        return Err("outlet area must be <= inlet area (this is a reducer)".into());
    }
    let r = a_out / a_in;
    let f = diffuser_factor(angle_deg.min(180.0));
    Ok((0.04 + 0.37 * (1.0 - r)) * f)
}

/// Rectangular expander / diffuser loss coefficient, referenced to the
/// **inlet** velocity (ASHRAE F29; Borda–Carnot sudden-expansion baseline).
///
/// ``zeta = (1 − A_in/A_out)² · f_angle``.
pub fn expander_rectangular(
    w_in: f64,
    h_in: f64,
    w_out: f64,
    h_out: f64,
    angle_deg: f64,
) -> Result<f64> {
    if w_in <= 0.0 || h_in <= 0.0 || w_out <= 0.0 || h_out <= 0.0 {
        return Err("widths and heights must be positive".into());
    }
    let a_in = w_in * h_in;
    let a_out = w_out * h_out;
    if a_out < a_in {
        return Err("outlet area must be >= inlet area (this is an expander)".into());
    }
    let r = a_in / a_out;
    let f = diffuser_factor(angle_deg.min(180.0));
    Ok((1.0 - r) * (1.0 - r) * f)
}

/// Round elbow loss coefficient (ASHRAE Fundamentals, smooth-radius round
/// elbow). Use `ElbowRound` (tabulated interpolation in `components::elbow`)
/// when you need the full R/D vs angle table; this is a compact algebraic form.
///
/// ``zeta = zeta_90 · (angle/90)`` with ``zeta_90 = clamp(0.21 / √(R/D)) ≤ 1.0``.
pub fn elbow_round(bend_radius: f64, diameter: f64, angle_deg: f64) -> Result<f64> {
    if diameter <= 0.0 || bend_radius <= 0.0 {
        return Err("bend_radius and diameter must be positive".into());
    }
    if !(angle_deg > 0.0 && angle_deg <= 180.0) {
        return Err("angle_deg must be in (0, 180]".into());
    }
    let rd = bend_radius / diameter;
    if rd < 0.5 {
        return Err("R/D must be >= 0.5 for a smooth round elbow".into());
    }
    let zeta_90 = (0.21 / rd.sqrt()).min(1.0);
    Ok(zeta_90 * (angle_deg / 90.0))
}

/// Louver (intake / relief / face) loss coefficient — long characteristic of
/// stationary-louver weather hoods (ASHRAE F25 / manufacturer data).
///
/// ``zeta = 0.25 + 4·(1 − open/100)³`` — low (~0.25) fully open, rising steeply
/// as the blades close.
pub fn louver_open(open_percentage: f64) -> Result<f64> {
    if !(0.0..=100.0).contains(&open_percentage) {
        return Err("open_percentage must be in [0, 100]".into());
    }
    let closed = 1.0 - open_percentage / 100.0;
    Ok(0.25 + 4.0 * closed * closed * closed)
}

/// Panel / throwaway filter-bank loss coefficient as a function of the free
/// face-area fraction left open by the media (ASHRAE / filter manufacturer
/// data). Loss rises quickly as the media clogs (free area falls).
///
/// ``zeta = 0.12 / open_fraction²``
pub fn filter_bank(open_fraction: f64) -> Result<f64> {
    if !(0.05..=1.0).contains(&open_fraction) {
        return Err("open_fraction must be in [0.05, 1]".into());
    }
    Ok(0.12 / (open_fraction * open_fraction))
}

/// Round-tap branch loss coefficient into a round main (Miller / ASHRAE tap
/// data), referenced to the branch velocity. `split_ratio` is the fraction of
/// the main flow diverted into the tap (0..1).
///
/// ``zeta = 0.6 + 0.5·(1 − (d_tap/d_main)²) + 0.8·(1 − split)``
pub fn round_tap_branch(d_main: f64, d_tap: f64, split_ratio: f64) -> Result<f64> {
    if d_main <= 0.0 {
        return Err("d_main must be positive".into());
    }
    if !(d_tap > 0.0 && d_tap <= d_main) {
        return Err("d_tap must be in (0, d_main]".into());
    }
    if !(0.0..=1.0).contains(&split_ratio) {
        return Err("split_ratio must be in [0, 1]".into());
    }
    let area = (d_tap / d_main) * (d_tap / d_main);
    Ok(0.6 + 0.5 * (1.0 - area) + 0.8 * (1.0 - split_ratio))
}

/// Named constant zeta values for common devices — the seed of a
/// vendor-catalogue lookup (FR-19). `named_zeta` resolves a device key to a
/// loss coefficient, or `None` if unknown.
pub const NAMED_FITTING_ZETAS: &[(&str, f64)] = &[
    ("duct_section", 0.0),
    ("fire_damper_open", 0.18),
    ("butterfly_open", 0.1),
    ("silencer_open", 0.35),
    ("diffuser_face", 0.4),
    ("grille_return", 0.25),
    ("louver_open", 0.25),
    ("damper_open", 0.1),
];

/// Look up a named device loss coefficient (constant), e.g.
/// `named_zeta("fire_damper_open") == Some(0.18)`.
pub fn named_zeta(name: &str) -> Option<f64> {
    NAMED_FITTING_ZETAS
        .iter()
        .find(|(k, _)| *k == name)
        .map(|(_, z)| *z)
}

#[cfg(test)]
mod tests_expanded {
    use super::*;

    #[test]
    fn reducer_rectangular_closed_form_and_ideal() {
        // equal areas -> only the connection base loss (the 0.04 housing term)
        assert!((reducer_rectangular(0.3, 0.2, 0.3, 0.2, 90.0).unwrap() - 0.04).abs() < 1e-9);
        // a real contraction is positive
        let full = reducer_rectangular(0.4, 0.3, 0.2, 0.2, 90.0).unwrap();
        // r = 0.04/0.12 = 1/3 -> (0.04 + 0.37*2/3) * f(90°)=1.0
        assert!((full - (0.04 + 0.37 * (2.0 / 3.0))).abs() < 1e-9);
        // a reducer can't expand
        assert!(reducer_rectangular(0.2, 0.2, 0.4, 0.4, 45.0).is_err());
    }

    #[test]
    fn expander_rectangular_borda_carnot() {
        let z = expander_rectangular(0.2, 0.2, 0.4, 0.4, 90.0).unwrap();
        // r = 0.25, (1-0.25)^2 * 1.0
        assert!((z - 0.75 * 0.75).abs() < 1e-9);
        assert!(expander_rectangular(0.4, 0.4, 0.2, 0.2, 45.0).is_err());
    }

    #[test]
    fn elbow_round_scales_with_angle_and_rd() {
        let tight = elbow_round(0.2, 0.4, 90.0).unwrap(); // RD=0.5
        let wide = elbow_round(0.4, 0.4, 90.0).unwrap(); // RD=1.0
        assert!(wide < tight);
        // angle scales linearly: 45° is half of 90°
        let half = elbow_round(0.4, 0.4, 45.0).unwrap();
        assert!((half - wide / 2.0).abs() < 1e-12);
        // RD < 0.5 rejected
        assert!(elbow_round(0.1, 0.4, 90.0).is_err());
    }

    #[test]
    fn louver_open_is_steep_close() {
        assert!((louver_open(100.0).unwrap() - 0.25).abs() < 1e-12);
        let closed = louver_open(0.0).unwrap();
        let half = louver_open(50.0).unwrap();
        assert!(closed > half);
        assert!((closed - (0.25 + 4.0)).abs() < 1e-9);
        assert!(louver_open(101.0).is_err());
    }

    #[test]
    fn filter_bank_monotonic_and_closed_form() {
        assert!((filter_bank(1.0).unwrap() - 0.12).abs() < 1e-12);
        let clogged = filter_bank(0.5).unwrap();
        let open = filter_bank(0.9).unwrap();
        assert!(clogged > open);
        assert!((clogged - 0.48).abs() < 1e-9); // 0.12/0.25
    }

    #[test]
    fn round_tap_branch_validation_and_monotone() {
        let z = round_tap_branch(0.3, 0.15, 0.5).unwrap();
        assert!(z > 0.0);
        // larger tap (more area) and more split -> lower loss
        assert!(round_tap_branch(0.3, 0.25, 0.5).unwrap() < z);
        assert!(round_tap_branch(0.3, 0.15, 0.9).unwrap() < z);
        assert!(round_tap_branch(0.0, 0.1, 0.5).is_err());
        assert!(round_tap_branch(0.3, 0.4, 0.5).is_err()); // tap > main
    }

    #[test]
    fn named_zeta_lookup() {
        assert_eq!(named_zeta("fire_damper_open"), Some(0.18));
        assert_eq!(named_zeta("diffuser_face"), Some(0.4));
        assert_eq!(named_zeta("no_such_device"), None);
    }

    #[test]
    fn diffuser_factor_grows_with_angle() {
        // helper: monotonic non-decreasing
        assert!(diffuser_factor(5.0) <= diffuser_factor(50.0));
        assert!(diffuser_factor(50.0) <= diffuser_factor(120.0));
    }
}
