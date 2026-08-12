//! Sheet-metal developments (rozwinięcia) — unfold duct and fitting
//! geometries to flat patterns.
//!
//! These routines estimate the **flat-pattern size** (developed width,
//! length and surface area) needed to fabricate a duct, elbow or reducer out
//! of flat sheet metal. All values are SI (metres, radians / degrees).
//!
//! > **Heuristic note.** Everything in this module is a *design
//! > approximation* for estimating flat-pattern sizes (area / length), not
//! > an exact sheet-metal unfolding. Real fabrication must account for seam
//! > and edge allowances (pleating, standing seams, lock-forming, welding
//! > margins), material thickness, and the exact segmented construction of
//! > an elbow. Use the returned dimensions as starting estimates for
//! > quoting, weight, and logistics — not as cut-sheet-ready geometry.

use crate::Result;

/// A rectangular flat-pattern cut for a duct or fitting development.
///
/// `width_m` and `length_m` describe the bounding rectangle of the unfolded
/// sheet; `area_m2` is the developed surface area of the flat pattern.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FlatPiece {
    /// Developed width of the flat pattern (metres).
    pub width_m: f64,
    /// Developed length of the flat pattern (metres).
    pub length_m: f64,
    /// Developed surface area of the flat pattern (square metres).
    pub area_m2: f64,
}

impl FlatPiece {
    /// Build a piece from its width and length, computing the area.
    pub fn new(width_m: f64, length_m: f64) -> FlatPiece {
        FlatPiece {
            width_m,
            length_m,
            area_m2: width_m * length_m,
        }
    }
}

/// Validate that a dimensional argument is finite and positive.
fn require_positive(value: f64, name: &str) -> Result<()> {
    if !value.is_finite() {
        return Err(format!("{name} must be finite, got {value}").into());
    }
    if value <= 0.0 {
        return Err(format!("{name} must be positive, got {value}").into());
    }
    Ok(())
}

/// Unfolds a straight round duct into a flat rectangle.
///
/// The flat pattern is a rectangle whose *width* is the duct circumference
/// (`π·d`) and whose *length* is the duct length, so the developed area is
/// `π·d·l` — the lateral surface of the cylinder.
///
/// # Examples
///
/// ```
/// use venti::round_duct_development;
///
/// // A 0.2 m round duct, 3 m long.
/// let piece = round_duct_development(0.2, 3.0).unwrap();
/// assert!((piece.width_m - std::f64::consts::PI * 0.2).abs() < 1e-12);
/// assert_eq!(piece.length_m, 3.0);
/// assert!((piece.area_m2 - piece.width_m * piece.length_m).abs() < 1e-12);
/// ```
pub fn round_duct_development(duct_diameter_m: f64, length_m: f64) -> Result<FlatPiece> {
    require_positive(duct_diameter_m, "duct_diameter_m")?;
    require_positive(length_m, "length_m")?;
    let width = std::f64::consts::PI * duct_diameter_m;
    Ok(FlatPiece::new(width, length_m))
}

/// Unfolds a segmented round elbow into a flat strip.
///
/// The elbow centreline sweeps through `angle_deg` around a bend radius
/// `radius_m`, so the developed *length* along the centreline is the arc
/// length `angle_rad·radius`. The developed *width* is the duct
/// circumference `π·d`. The area is `width · arc_length`, independent of the
/// number of segments used (segment count only affects how the curved strip
/// is subdivided, not its total size).
///
/// # Examples
///
/// ```
/// use venti::round_elbow_development;
///
/// // A 90° elbow with 0.3 m bend radius around a 0.2 m duct.
/// let piece = round_elbow_development(0.3, 0.2, 90.0, 5).unwrap();
/// let arc = std::f64::consts::FRAC_PI_2 * 0.3;
/// assert!((piece.length_m - arc).abs() < 1e-12);
/// assert!((piece.width_m - std::f64::consts::PI * 0.2).abs() < 1e-12);
/// ```
pub fn round_elbow_development(
    radius_m: f64,
    diameter_m: f64,
    angle_deg: f64,
    segments: u32,
) -> Result<FlatPiece> {
    require_positive(radius_m, "radius_m")?;
    require_positive(diameter_m, "diameter_m")?;
    if !angle_deg.is_finite() {
        return Err(format!("angle_deg must be finite, got {angle_deg}").into());
    }
    if angle_deg <= 0.0 {
        return Err(format!("angle_deg must be positive, got {angle_deg}").into());
    }
    if segments == 0 {
        return Err("segments must be at least 1".into());
    }
    let angle_rad = angle_deg.to_radians();
    let width = std::f64::consts::PI * diameter_m;
    let arc_length = angle_rad * radius_m;
    Ok(FlatPiece::new(width, arc_length))
}

/// Unfolds a conical reducer (frustum) into a flat pattern using a trapezoid
/// approximation.
///
/// The flat pattern is approximated as a trapezoid (or, for a straight duct,
/// a rectangle) whose *width* is the average circumference `π·(d1+d2)/2` and
/// whose *length* is the slant height `√(length² + ((d2−d1)/2)²)`. This is a
/// close estimate of the developed surface of a conical frustum for shallow
/// tapers, but not an exact annular-sector unfolding.
///
/// See the module-level note: this is an estimate for sizing, not a
/// cut-sheet layout.
pub fn reducer_cone_development(
    d_small_m: f64,
    d_large_m: f64,
    length_m: f64,
) -> Result<FlatPiece> {
    require_positive(d_small_m, "d_small_m")?;
    require_positive(d_large_m, "d_large_m")?;
    require_positive(length_m, "length_m")?;
    let half_delta = (d_large_m - d_small_m).abs() / 2.0;
    let slant = (length_m * length_m + half_delta * half_delta).sqrt();
    let width = std::f64::consts::PI * (d_small_m + d_large_m) / 2.0;
    Ok(FlatPiece::new(width, slant))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duct_circumference_area_exact() {
        let d = 0.25;
        let l = 4.0;
        let p = round_duct_development(d, l).unwrap();
        let circ = std::f64::consts::PI * d;
        assert!((p.width_m - circ).abs() < 1e-12);
        assert_eq!(p.length_m, l);
        assert!((p.area_m2 - circ * l).abs() < 1e-12);
    }

    #[test]
    fn elbow_area_is_width_times_arclength() {
        let r = 0.3;
        let d = 0.2;
        let deg = 45.0;
        let p = round_elbow_development(r, d, deg, 5).unwrap();
        let arc = deg.to_radians() * r;
        let width = std::f64::consts::PI * d;
        assert!((p.length_m - arc).abs() < 1e-12);
        assert!((p.area_m2 - width * arc).abs() < 1e-12);
    }

    #[test]
    fn reducer_trrapezoid_area() {
        let d_small = 0.1;
        let d_large = 0.3;
        let l = 0.5;
        let p = reducer_cone_development(d_small, d_large, l).unwrap();
        let slant = (l * l + ((d_large - d_small) / 2.0).powi(2)).sqrt();
        let width = std::f64::consts::PI * (d_small + d_large) / 2.0;
        assert!((p.length_m - slant).abs() < 1e-12);
        assert!((p.width_m - width).abs() < 1e-12);
        assert!((p.area_m2 - width * slant).abs() < 1e-12);
    }

    #[test]
    fn angle_180_is_full_turn_half_circumference() {
        let r = 0.3;
        let d = 0.2;
        let p = round_elbow_development(r, d, 180.0, 7).unwrap();
        // 180° = π radians, so the centreline length is π·r (half a full turn
        // would be 2π·r).
        assert!((p.length_m - std::f64::consts::PI * r).abs() < 1e-12);
        assert!((p.area_m2 - std::f64::consts::PI * d * std::f64::consts::PI * r).abs() < 1e-12);
    }

    #[test]
    fn segment_count_does_not_change_area() {
        let r = 0.3;
        let d = 0.2;
        let deg = 90.0;
        let a = round_elbow_development(r, d, deg, 3).unwrap();
        let b = round_elbow_development(r, d, deg, 12).unwrap();
        assert_eq!(a.area_m2, b.area_m2);
    }

    #[test]
    fn straight_reducer_is_rectangle() {
        // d_small == d_large => slant == length and width == circumference.
        let d = 0.2;
        let l = 1.0;
        let p = reducer_cone_development(d, d, l).unwrap();
        assert!((p.length_m - l).abs() < 1e-12);
        assert!((p.width_m - std::f64::consts::PI * d).abs() < 1e-12);
    }

    #[test]
    fn rejects_non_positive_inputs() {
        assert!(round_duct_development(0.0, 1.0).is_err());
        assert!(round_duct_development(1.0, -1.0).is_err());
        assert!(round_duct_development(f64::NAN, 1.0).is_err());

        assert!(round_elbow_development(0.0, 0.2, 90.0, 5).is_err());
        assert!(round_elbow_development(0.3, 0.0, 90.0, 5).is_err());
        assert!(round_elbow_development(0.3, 0.2, 0.0, 5).is_err());
        assert!(round_elbow_development(0.3, 0.2, 90.0, 0).is_err());
        assert!(round_elbow_development(f64::INFINITY, 0.2, 90.0, 5).is_err());

        assert!(reducer_cone_development(0.0, 0.3, 0.5).is_err());
        assert!(reducer_cone_development(0.1, 0.3, -1.0).is_err());
        assert!(reducer_cone_development(0.1, f64::NAN, 0.5).is_err());
    }
}
