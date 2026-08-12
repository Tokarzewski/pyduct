//! Duct fabrication breakout — lengths, surface area, weight, cutting schedule.
//!
//! This module ("CADvent feature", issue #30) computes the fabrication
//! quantities a duct shop needs to cut and order sheet metal for a run of
//! ductwork:
//!
//! * **[`duct_surface_area_m2`]** — the wetted surface area of a duct run
//!   (perimeter × length), the basis for estimating sheet-metal area.
//! * **[`duct_weight_kg`]** — the weight of that sheet-metal area given a steel
//!   gauge (thickness) and density (steel ≈ 7850 kg/m³).
//! * **[`FabricationBreakout`]** — a straight-vs-fittings length summary for a
//!   fabricated run.
//! * **[`cutting_schedule`]** — accumulate per-component lengths across many
//!   runs and return a sorted, de-duplicated list (component, total length).
//!
//! All quantities are SI: length `m`, area `m²`, weight `kg`, density `kg/m³`.

use crate::core::geometry::CrossSection;
use crate::Result;
use std::collections::BTreeMap;
use std::f64::consts::PI;

/// Default density of duct steel [kg/m³] (mild / galvanized sheet steel).
pub const STEEL_DENSITY_KG_M3: f64 = 7850.0;

/// Wetted surface area of a duct run with the given cross-section and length
/// [m²] — the metal area to be fabricated.
///
/// This is the perimeter of the cross-section multiplied by the run length:
///
/// ```text
/// round:        A = π · D · L
/// rectangular:  A = 2 · (W + H) · L
/// ```
///
/// # Examples
///
/// ```
/// use venti::{duct_surface_area_m2, CrossSection, Round, Rectangular};
///
/// // Round duct, D = 0.2 m, 10 m long.
/// let round = CrossSection::Round(Round::new(0.2).unwrap());
/// let a = duct_surface_area_m2(&round, 10.0).unwrap();
/// assert!((a - std::f64::consts::PI * 0.2 * 10.0).abs() < 1e-12);
///
/// // Rectangular duct, 0.3 m × 0.2 m, 5 m long.
/// let rect = CrossSection::Rectangular(Rectangular::new(0.3, 0.2).unwrap());
/// let a = duct_surface_area_m2(&rect, 5.0).unwrap();
/// assert!((a - 2.0 * (0.3 + 0.2) * 5.0).abs() < 1e-12);
/// ```
pub fn duct_surface_area_m2(cross_section: &CrossSection, length_m: f64) -> Result<f64> {
    if length_m < 0.0 {
        return Err("length must be non-negative".into());
    }
    let perimeter = match cross_section {
        CrossSection::Round(r) => PI * r.diameter,
        CrossSection::Rectangular(r) => 2.0 * (r.width + r.height),
    };
    Ok(perimeter * length_m)
}

/// Weight [kg] of fabricated sheet metal for a given surface area, steel gauge
/// (thickness) and density.
///
/// ```text
/// weight = surface_area_m2 · gauge_m · density_kg_m3
/// ```
///
/// `gauge_mm` is required (a thickness of zero contributes no weight); when
/// `density_kg_m3` is `None` it defaults to [`STEEL_DENSITY_KG_M3`] (7850 kg/m³).
///
/// # Examples
///
/// ```
/// use venti::duct_weight_kg;
///
/// // 1 m² of 1 mm galvanized steel plate.
/// let w = duct_weight_kg(1.0, Some(1.0), None).unwrap();
/// assert!((w - 7850.0 * 0.001).abs() < 1e-9);
/// assert!((w - 7.85).abs() < 1e-9);
///
/// // A gauge is mandatory.
/// assert!(duct_weight_kg(1.0, None, None).is_err());
/// ```
pub fn duct_weight_kg(
    surface_area_m2: f64,
    gauge_mm: Option<f64>,
    density_kg_m3: Option<f64>,
) -> Result<f64> {
    if surface_area_m2 < 0.0 {
        return Err("surface area must be non-negative".into());
    }
    let gauge_m = match gauge_mm {
        Some(g) if g > 0.0 => g / 1000.0,
        Some(_) => return Err("gauge must be positive".into()),
        None => return Err("gauge (steel thickness) is required to compute weight".into()),
    };
    let density = density_kg_m3.unwrap_or(STEEL_DENSITY_KG_M3);
    if density <= 0.0 {
        return Err("density must be positive".into());
    }
    Ok(surface_area_m2 * gauge_m * density)
}

/// A fabrication length breakout for a run: straight duct vs. fittings.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FabricationBreakout {
    /// Total straight (non-fitting) duct length [m].
    pub straight_m: f64,
    /// Total equivalent fitting length [m].
    pub fittings_m: f64,
}

impl FabricationBreakout {
    /// Build a breakout from straight and fitting lengths, validating that both
    /// are non-negative.
    pub fn new(straight: f64, fittings: f64) -> Result<Self> {
        if straight < 0.0 || fittings < 0.0 {
            return Err("straight and fittings lengths must be non-negative".into());
        }
        Ok(FabricationBreakout {
            straight_m: straight,
            fittings_m: fittings,
        })
    }

    /// Total fabricated duct length [m] = straight + fittings.
    ///
    /// # Examples
    ///
    /// ```
    /// use venti::FabricationBreakout;
    /// let b = FabricationBreakout::new(20.0, 3.5).unwrap();
    /// assert!((b.total_m() - 23.5).abs() < 1e-12);
    /// ```
    pub fn total_m(&self) -> f64 {
        self.straight_m + self.fittings_m
    }
}

/// Accumulate per-component duct lengths across many runs.
///
/// Takes a list of `(component, length_m)` pairs and returns a sorted,
/// de-duplicated `Vec<(component, total_length_m)>`, summing the lengths of
/// each component wherever it repeats.
///
/// # Examples
///
/// ```
/// use venti::cutting_schedule;
///
/// let cuts = vec![
///     ("M1".to_string(), 10.0),
///     ("M2".to_string(), 5.0),
///     ("M1".to_string(), 6.0),
/// ];
/// let sched = cutting_schedule(&cuts);
/// // Sorted by component name: M1 then M2.
/// assert_eq!(sched, vec![("M1".to_string(), 16.0), ("M2".to_string(), 5.0)]);
/// ```
pub fn cutting_schedule(ducts: &[(String, f64)]) -> Vec<(String, f64)> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for (component, length_m) in ducts {
        *totals.entry(component.clone()).or_insert(0.0) += length_m;
    }
    totals.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::geometry::{Rectangular, Round};

    #[test]
    fn round_surface_area_is_pi_d_times_len() {
        let r = Round::new(0.2).unwrap();
        let a = duct_surface_area_m2(&CrossSection::Round(r), 10.0).unwrap();
        assert!((a - PI * 0.2 * 10.0).abs() < 1e-12);
    }

    #[test]
    fn rectangular_surface_area_is_twice_w_plus_h_times_len() {
        let rect = Rectangular::new(0.3, 0.2).unwrap();
        let a = duct_surface_area_m2(&CrossSection::Rectangular(rect), 5.0).unwrap();
        assert!((a - 2.0 * (0.3 + 0.2) * 5.0).abs() < 1e-12);
    }

    #[test]
    fn zero_length_surface_area_is_zero() {
        let r = Round::new(0.2).unwrap();
        let a = duct_surface_area_m2(&CrossSection::Round(r), 0.0).unwrap();
        assert_eq!(a, 0.0);
    }

    #[test]
    fn surface_area_rejects_negative_length() {
        let r = Round::new(0.2).unwrap();
        assert!(duct_surface_area_m2(&CrossSection::Round(r), -1.0).is_err());
    }

    #[test]
    fn weight_formula_and_default_density() {
        // 1 m² of 1 mm steel at default 7850 kg/m³.
        let w = duct_weight_kg(1.0, Some(1.0), None).unwrap();
        assert!((w - 7.85).abs() < 1e-9);
        // Custom density.
        let w = duct_weight_kg(2.0, Some(0.5), Some(8000.0)).unwrap();
        assert!((w - 2.0 * 0.0005 * 8000.0).abs() < 1e-9);
        assert!((w - 8.0).abs() < 1e-9);
    }

    #[test]
    fn weight_requires_gauge() {
        assert!(duct_weight_kg(1.0, None, None).is_err());
        assert!(duct_weight_kg(1.0, Some(0.0), None).is_err());
        assert!(duct_weight_kg(-1.0, Some(1.0), None).is_err());
        assert!(duct_weight_kg(1.0, Some(1.0), Some(0.0)).is_err());
    }

    #[test]
    fn breakout_totals() {
        let b = FabricationBreakout::new(20.0, 3.5).unwrap();
        assert_eq!(b.straight_m, 20.0);
        assert_eq!(b.fittings_m, 3.5);
        assert!((b.total_m() - 23.5).abs() < 1e-12);
    }

    #[test]
    fn breakout_rejects_negative() {
        assert!(FabricationBreakout::new(-1.0, 2.0).is_err());
        assert!(FabricationBreakout::new(1.0, -2.0).is_err());
        assert!(FabricationBreakout::new(-1.0, -2.0).is_err());
        assert!(FabricationBreakout::new(0.0, 0.0).is_ok());
    }

    #[test]
    fn cutting_schedule_sums_and_sorts() {
        let cuts = vec![
            ("B".to_string(), 3.0),
            ("A".to_string(), 10.0),
            ("A".to_string(), 2.0),
            ("B".to_string(), 1.0),
        ];
        let sched = cutting_schedule(&cuts);
        assert_eq!(
            sched,
            vec![("A".to_string(), 12.0), ("B".to_string(), 4.0),]
        );
    }

    #[test]
    fn cutting_schedule_empty_input() {
        assert_eq!(cutting_schedule(&[]), Vec::<(String, f64)>::new());
    }
}
