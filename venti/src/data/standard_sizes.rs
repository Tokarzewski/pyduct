//! Standard EN duct sizes (dimensions in millimetres).
//!
//! * `STANDARD_RECTANGULAR_DUCT_SIZES`     — EN 1505:2001
//! * `STANDARD_ROUND_DUCT_SIZES`           — EN 1506:2007
//! * `STANDARD_ROUND_BRANCH_SIZES`         — round branch combinations
//! * `STANDARD_ROUND_TRANSFORMATION_SIZES` — round reducer combinations
//! * `STANDARD_ROUND_SECTIONS`             — pre-built `Round` instances
//! * `STANDARD_RECTANGULAR_SECTIONS`       — pre-built `Rectangular` instances

use crate::core::geometry::{Rectangular, Round};

/// EN 1505:2001 — Rectangular ducts (width × height in mm).
pub const STANDARD_RECTANGULAR_DUCT_SIZES: [(u16, u16); 74] = [
    (100, 200),
    (150, 200),
    (200, 200),
    (100, 250),
    (150, 250),
    (200, 250),
    (250, 250),
    (100, 300),
    (150, 300),
    (200, 300),
    (250, 300),
    (300, 300),
    (100, 400),
    (150, 400),
    (200, 400),
    (250, 400),
    (300, 400),
    (400, 400),
    (150, 500),
    (200, 500),
    (250, 500),
    (300, 500),
    (400, 500),
    (500, 500),
    (150, 600),
    (200, 600),
    (250, 600),
    (300, 600),
    (400, 600),
    (500, 600),
    (600, 600),
    (200, 800),
    (250, 800),
    (300, 800),
    (400, 800),
    (500, 800),
    (600, 800),
    (800, 800),
    (250, 1000),
    (300, 1000),
    (400, 1000),
    (500, 1000),
    (600, 1000),
    (800, 1000),
    (1000, 1000),
    (300, 1200),
    (400, 1200),
    (500, 1200),
    (600, 1200),
    (800, 1200),
    (1000, 1200),
    (1200, 1200),
    (400, 1400),
    (500, 1400),
    (600, 1400),
    (800, 1400),
    (1000, 1400),
    (1200, 1400),
    (400, 1600),
    (500, 1600),
    (600, 1600),
    (800, 1600),
    (1000, 1600),
    (1200, 1600),
    (500, 1800),
    (600, 1800),
    (800, 1800),
    (1000, 1800),
    (1200, 1800),
    (500, 2000),
    (600, 2000),
    (800, 2000),
    (1000, 2000),
    (1200, 2000),
];

/// EN 1506:2007 — Round ducts (nominal diameter in mm).
pub const STANDARD_ROUND_DUCT_SIZES: [u16; 22] = [
    63, 80, 100, 125, 150, 160, 200, 250, 300, 315, 355, 400, 450, 500, 560, 630, 710, 800, 900,
    1000, 1120, 1250,
];

/// Round branch sizes `[d3, d1]`.
pub const STANDARD_ROUND_BRANCH_SIZES: [(u16, u16); 141] = [
    (63, 80),
    (80, 80),
    (63, 100),
    (80, 100),
    (100, 100),
    (80, 125),
    (100, 125),
    (125, 125),
    (80, 150),
    (100, 150),
    (125, 150),
    (150, 150),
    (80, 160),
    (100, 160),
    (125, 160),
    (150, 160),
    (160, 160),
    (80, 200),
    (100, 200),
    (125, 200),
    (150, 200),
    (160, 200),
    (200, 200),
    (80, 250),
    (100, 250),
    (125, 250),
    (150, 250),
    (160, 250),
    (200, 250),
    (250, 250),
    (100, 300),
    (125, 300),
    (150, 300),
    (160, 300),
    (200, 300),
    (250, 300),
    (300, 300),
    (100, 315),
    (125, 315),
    (150, 315),
    (160, 315),
    (200, 315),
    (250, 315),
    (300, 315),
    (315, 315),
    (160, 355),
    (200, 355),
    (250, 355),
    (300, 355),
    (315, 355),
    (355, 355),
    (160, 400),
    (200, 400),
    (250, 400),
    (300, 400),
    (315, 400),
    (355, 400),
    (400, 400),
    (200, 450),
    (250, 450),
    (300, 450),
    (315, 450),
    (355, 450),
    (400, 450),
    (450, 450),
    (200, 500),
    (250, 500),
    (300, 500),
    (315, 500),
    (355, 500),
    (400, 500),
    (450, 500),
    (500, 500),
    (250, 560),
    (300, 560),
    (315, 560),
    (355, 560),
    (400, 560),
    (450, 560),
    (500, 560),
    (560, 560),
    (250, 630),
    (300, 630),
    (315, 630),
    (355, 630),
    (400, 630),
    (450, 630),
    (500, 630),
    (560, 630),
    (630, 630),
    (315, 710),
    (355, 710),
    (400, 710),
    (450, 710),
    (500, 710),
    (560, 710),
    (630, 710),
    (710, 710),
    (315, 800),
    (355, 800),
    (400, 800),
    (450, 800),
    (500, 800),
    (560, 800),
    (630, 800),
    (710, 800),
    (800, 800),
    (400, 900),
    (450, 900),
    (500, 900),
    (560, 900),
    (630, 900),
    (710, 900),
    (800, 900),
    (900, 900),
    (400, 1000),
    (450, 1000),
    (500, 1000),
    (560, 1000),
    (630, 1000),
    (710, 1000),
    (800, 1000),
    (900, 1000),
    (1000, 1000),
    (500, 1120),
    (560, 1120),
    (630, 1120),
    (710, 1120),
    (800, 1120),
    (900, 1120),
    (1000, 1120),
    (1120, 1120),
    (500, 1250),
    (560, 1250),
    (630, 1250),
    (710, 1250),
    (800, 1250),
    (900, 1250),
    (1000, 1250),
    (1120, 1250),
    (1250, 1250),
];

/// Round reducer sizes `[d3, d1]`.
pub const STANDARD_ROUND_TRANSFORMATION_SIZES: [(u16, u16); 68] = [
    (63, 80),
    (80, 80),
    (63, 100),
    (80, 100),
    (100, 100),
    (63, 125),
    (80, 125),
    (100, 125),
    (80, 150),
    (100, 150),
    (125, 150),
    (80, 160),
    (100, 160),
    (125, 160),
    (150, 160),
    (100, 200),
    (125, 200),
    (150, 200),
    (160, 200),
    (125, 250),
    (150, 250),
    (160, 250),
    (200, 250),
    (150, 300),
    (160, 300),
    (200, 300),
    (250, 300),
    (160, 315),
    (200, 315),
    (250, 315),
    (200, 355),
    (250, 355),
    (300, 355),
    (315, 355),
    (250, 400),
    (300, 400),
    (315, 400),
    (355, 400),
    (300, 450),
    (315, 450),
    (355, 450),
    (400, 450),
    (355, 500),
    (400, 500),
    (450, 500),
    (400, 560),
    (450, 560),
    (500, 560),
    (450, 630),
    (500, 630),
    (560, 630),
    (500, 710),
    (560, 710),
    (630, 710),
    (560, 800),
    (630, 800),
    (710, 800),
    (630, 900),
    (710, 900),
    (800, 900),
    (710, 1000),
    (800, 1000),
    (900, 1000),
    (800, 1120),
    (900, 1120),
    (1000, 1120),
    (900, 1250),
    (1000, 1250),
];

/// Lazy pre-built round sections (area + hydraulic diameter cached).
pub fn standard_round_sections() -> Vec<Round> {
    STANDARD_ROUND_DUCT_SIZES
        .iter()
        .map(|d| Round::new(f64::from(*d) / 1000.0).expect("standard sizes are positive"))
        .collect()
}

/// Lazy pre-built rectangular sections (area + hydraulic diameter cached).
pub fn standard_rectangular_sections() -> Vec<Rectangular> {
    STANDARD_RECTANGULAR_DUCT_SIZES
        .iter()
        .map(|(w, h)| {
            Rectangular::new(f64::from(*w) / 1000.0, f64::from(*h) / 1000.0)
                .expect("standard sizes are positive")
        })
        .collect()
}

/// Return the nearest EN 1506 nominal diameter [mm].
///
/// With `round_up = true` (default), picks the smallest standard size that is
/// `>= diameter_mm`; otherwise picks the closest standard size in either
/// direction.
pub fn nearest_round_size(diameter_mm: f64, round_up: bool) -> u16 {
    let sizes = &STANDARD_ROUND_DUCT_SIZES;
    let first = sizes[0];
    let last = sizes[sizes.len() - 1];
    let dd = diameter_mm;

    if dd <= f64::from(first) {
        return first;
    }
    if dd >= f64::from(last) {
        return last;
    }

    // 22 elements — a linear scan is cheaper than a bisect here.
    let mut idx = 0usize;
    for (i, s) in sizes.iter().enumerate() {
        if f64::from(*s) >= dd {
            idx = i;
            break;
        }
    }

    let hi = sizes[idx];
    if round_up || f64::from(hi) == dd {
        return hi;
    }
    let lo = sizes[idx - 1];
    let d_hi = f64::from(hi) - dd;
    let d_lo = dd - f64::from(lo);
    if d_hi < d_lo {
        hi
    } else {
        lo
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_round_sizes_have_expected_head() {
        assert_eq!(STANDARD_ROUND_DUCT_SIZES[0], 63);
        assert_eq!(STANDARD_ROUND_DUCT_SIZES.len(), 22);
    }

    #[test]
    fn nearest_round_up() {
        assert_eq!(nearest_round_size(120.0, true), 125);
        assert_eq!(nearest_round_size(63.0, true), 63);
        assert_eq!(nearest_round_size(10.0, true), 63); // below min -> min
        assert_eq!(nearest_round_size(20000.0, true), 1250); // above max -> max
    }

    #[test]
    fn nearest_round_closest() {
        // 110 is closer to 100 than 125 on either side.
        assert_eq!(nearest_round_size(110.0, false), 100);
        // 120 is closer to 125.
        assert_eq!(nearest_round_size(120.0, false), 125);
    }

    #[test]
    fn sections_have_positive_area() {
        let rounds = standard_round_sections();
        assert_eq!(rounds.len(), STANDARD_ROUND_DUCT_SIZES.len());
        for r in &rounds {
            assert!(r.area > 0.0);
        }
    }
}
