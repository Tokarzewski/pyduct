//! Loss-coefficient correlations for common HVAC fittings.
//!
//! Mirrors `wentamojo/components/fittings_library.mojo` and the Python
//! `wenta.components.fittings_library`. Coefficients from ASHRAE Fundamentals
//! and ductwork design guides (Hendiger, Idelchik).

/// Smooth-radius rectangular elbow (Idelchik §6) with aspect correction.
pub fn rectangular_elbow(
    width: f64,
    height: f64,
    bend_radius: f64,
    angle_deg: f64,
) -> Result<f64, String> {
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
pub fn reducer_round(d_inlet: f64, d_outlet: f64, angle_deg: f64) -> Result<f64, String> {
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
pub fn expander_round(d_inlet: f64, d_outlet: f64, angle_deg: f64) -> Result<f64, String> {
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
) -> Result<(f64, f64), String> {
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
) -> Result<(f64, f64), String> {
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
pub fn damper_butterfly(open_percentage: f64) -> Result<f64, String> {
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
pub fn diffuser_ceiling(area_throw: f64) -> Result<f64, String> {
    if area_throw <= 0.0 {
        return Err("area_throw must be positive".into());
    }
    Ok(0.4 / area_throw)
}

/// Return-grille face-velocity loss coefficient.
pub fn grille_return(blockage_factor: f64) -> Result<f64, String> {
    if !(0.0..=1.0).contains(&blockage_factor) {
        return Err("blockage_factor must be in [0, 1]".into());
    }
    Ok(0.25 * (1.0 + blockage_factor))
}

/// Sharp-corner mitered elbow loss coefficient; `vaned=True` cuts it to ~40 %.
pub fn mitered_elbow(angle_deg: f64, vaned: bool) -> Result<f64, String> {
    if angle_deg <= 0.0 || angle_deg > 180.0 {
        return Err("angle_deg must be in (0, 180]".into());
    }
    let a = angle_deg / 90.0;
    let zeta_unvaned = 0.55 * a + 0.65 * a * a;
    Ok(zeta_unvaned * if vaned { 0.4 } else { 1.0 })
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
}
