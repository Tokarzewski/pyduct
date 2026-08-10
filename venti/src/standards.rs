//! Configurable STANDARDS (size tables) for duct sizing.
//!
//! Different projects and regions size ductwork to different preferred
//! standard dimension sets. This module exposes the three most common ones so
//! the sizing code can choose between **EN 1505/1506**, **ASHRAE/SMACNA**, and
//! **DIN** without changing the sizing machinery itself.
//!
//! All dimension tables are in **millimetres**.
//!
//! * [`Standard::En1505_1506`] — the European norms, reusing the existing
//!   `venti::data::standard_sizes` tables (EN 1505:2001 rectangular, EN
//!   1506:2007 round).
//! * [`Standard::AsHrae`]      — ASHRAE / SMACNA nominal sizes derived from the
//!   customary **inch-based** series, converted to mm and rounded.
//! * [`Standard::Din`]         — the German DIN 24155 series based on Renard
//!   R10/R20 preferred numbers.
//!
//! The module is fully self-contained (no external dependencies): the EN
//! tables are read from `crate::data::standard_sizes` (which itself has no
//! dependencies), and the ASHRAE/DIN tables are declared inline here.

use crate::data::standard_sizes::{STANDARD_RECTANGULAR_DUCT_SIZES, STANDARD_ROUND_DUCT_SIZES};

/// A selectable sizing standard (dimension set).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Standard {
    /// EN 1505:2001 (rectangular) + EN 1506:2007 (round).
    ///
    /// Uses the existing `venti::data::standard_sizes` tables verbatim, so the
    /// canonical EN behaviour is preserved.
    En1505_1506,
    /// ASHRAE / SMACNA nominal sizes (inch-based series, converted to mm).
    AsHrae,
    /// DIN 24155 series (Renard R10/R20 preferred numbers).
    Din,
}

/// ASHRAE round ducts — customary nominal diameters in inches converted to
/// millimetres and rounded to whole mm: 4..40 in one-inch steps.
pub const ASHRAE_ROUND_DUCT_SIZES: &[u16] = &[
    102, 127, 152, 178, 203, 229, 254, 305, 356, 406, 457, 508, 559, 610, 660, 711, 762, 813, 864,
    914, 965, 1016,
];

/// ASHRAE / SMACNA rectangular ducts — customary nominal width×height in
/// inches converted to mm and rounded (`w × h`, `w ≥ h`).
pub const ASHRAE_RECTANGULAR_DUCT_SIZES: &[(u16, u16)] = &[
    (152, 102),
    (203, 102),
    (203, 152),
    (254, 102),
    (254, 152),
    (305, 102),
    (305, 152),
    (305, 203),
    (356, 152),
    (356, 203),
    (356, 254),
    (406, 152),
    (406, 203),
    (406, 254),
    (406, 305),
    (457, 203),
    (457, 254),
    (457, 305),
    (508, 203),
    (508, 254),
    (508, 305),
    (508, 356),
    (559, 254),
    (559, 305),
    (610, 203),
    (610, 254),
    (610, 305),
    (610, 356),
    (610, 406),
    (660, 305),
    (711, 305),
    (762, 305),
    (762, 356),
    (914, 356),
    (914, 406),
    (1067, 406),
    (1219, 406),
    (1219, 457),
    (1372, 457),
    (1524, 508),
    (1676, 559),
    (1829, 610),
];

/// DIN 24155 round ducts — nominal diameter in mm from the Renard R10 series.
pub const DIN_ROUND_DUCT_SIZES: &[u16] = &[
    63, 80, 100, 125, 160, 200, 250, 315, 400, 500, 630, 800, 1000, 1250,
];

/// DIN 24155 rectangular ducts — `w × h` in mm where both sides come from the
/// Renard R10/R20 preferred-number series and `w ≥ h`.
pub const DIN_RECTANGULAR_DUCT_SIZES: &[(u16, u16)] = &[
    (100, 100),
    (125, 100),
    (125, 125),
    (160, 100),
    (160, 125),
    (160, 160),
    (200, 100),
    (200, 125),
    (200, 160),
    (200, 200),
    (250, 100),
    (250, 125),
    (250, 160),
    (250, 200),
    (250, 250),
    (315, 100),
    (315, 125),
    (315, 160),
    (315, 200),
    (315, 250),
    (315, 315),
    (400, 100),
    (400, 125),
    (400, 160),
    (400, 200),
    (400, 250),
    (400, 315),
    (400, 400),
    (500, 100),
    (500, 125),
    (500, 160),
    (500, 200),
    (500, 250),
    (500, 315),
    (500, 400),
    (500, 500),
    (630, 100),
    (630, 125),
    (630, 160),
    (630, 200),
    (630, 250),
    (630, 315),
    (630, 400),
    (630, 500),
    (630, 630),
    (800, 100),
    (800, 125),
    (800, 160),
    (800, 200),
    (800, 250),
    (800, 315),
    (800, 400),
    (800, 500),
    (800, 630),
    (800, 800),
    (1000, 100),
    (1000, 125),
    (1000, 160),
    (1000, 200),
    (1000, 250),
    (1000, 315),
    (1000, 400),
    (1000, 500),
    (1000, 630),
    (1000, 800),
    (1000, 1000),
    (1250, 100),
    (1250, 125),
    (1250, 160),
    (1250, 200),
    (1250, 250),
    (1250, 315),
    (1250, 400),
    (1250, 500),
    (1250, 630),
    (1250, 800),
    (1250, 1000),
    (1250, 1250),
    (1600, 100),
    (1600, 125),
    (1600, 160),
    (1600, 200),
    (1600, 250),
    (1600, 315),
    (1600, 400),
    (1600, 500),
    (1600, 630),
    (1600, 800),
    (1600, 1000),
    (1600, 1250),
    (1600, 1600),
    (2000, 100),
    (2000, 125),
    (2000, 160),
    (2000, 200),
    (2000, 250),
    (2000, 315),
    (2000, 400),
    (2000, 500),
    (2000, 630),
    (2000, 800),
    (2000, 1000),
    (2000, 1250),
    (2000, 1600),
    (2000, 2000),
];

/// The standard round duct sizes for a given [`Standard`], in mm.
pub fn standard_round_sizes(standard: Standard) -> &'static [u16] {
    match standard {
        Standard::En1505_1506 => &STANDARD_ROUND_DUCT_SIZES,
        Standard::AsHrae => ASHRAE_ROUND_DUCT_SIZES,
        Standard::Din => DIN_ROUND_DUCT_SIZES,
    }
}

/// The standard rectangular duct sizes for a given [`Standard`], as
/// `(width, height)` in mm.
pub fn standard_rectangular_sizes(standard: Standard) -> &'static [(u16, u16)] {
    match standard {
        Standard::En1505_1506 => &STANDARD_RECTANGULAR_DUCT_SIZES,
        Standard::AsHrae => ASHRAE_RECTANGULAR_DUCT_SIZES,
        Standard::Din => DIN_RECTANGULAR_DUCT_SIZES,
    }
}

/// Convenience: the standard round sizes for a [`Standard`] as an owned `Vec`.
pub fn round_sizes_mm(standard: Standard) -> Vec<u16> {
    standard_round_sizes(standard).to_vec()
}

/// Convenience: the standard rectangular sizes for a [`Standard`] as an owned
/// `Vec` of `(width, height)` in mm.
pub fn rect_sizes_mm(standard: Standard) -> Vec<(u16, u16)> {
    standard_rectangular_sizes(standard).to_vec()
}

/// Return the nearest standard round diameter [mm] for the given
/// [`Standard`].
///
/// With `round_up = true` (default), picks the smallest standard size that is
/// `>= diameter_mm`; otherwise picks the closest standard size in either
/// direction. Values below the smallest / above the largest standard size are
/// clamped to the respective end of the table.
pub fn nearest_round_size_for(standard: Standard, diameter_mm: f64, round_up: bool) -> u16 {
    let sizes = standard_round_sizes(standard);
    let first = sizes[0];
    let last = sizes[sizes.len() - 1];
    let dd = diameter_mm;

    if dd <= f64::from(first) {
        return first;
    }
    if dd >= f64::from(last) {
        return last;
    }

    // Linear scan: tables are small and already sorted.
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

    fn assert_sorted(st: Standard) {
        let round = standard_round_sizes(st);
        assert!(!round.is_empty(), "round table must be non-empty");
        for w in round.windows(2) {
            assert!(w[0] < w[1], "round table not strictly increasing");
        }

        let rect = standard_rectangular_sizes(st);
        assert!(!rect.is_empty(), "rect table must be non-empty");
        // Every dimension is a positive, non-degenerate size.
        for &(w, h) in rect {
            assert!(w > 0 && h > 0, "rect pair has zero dimension");
        }
    }

    #[test]
    fn every_table_is_nonempty_and_sorted() {
        assert_sorted(Standard::En1505_1506);
        assert_sorted(Standard::AsHrae);
        assert_sorted(Standard::Din);
    }

    #[test]
    fn en_table_shares_canonical_constants() {
        assert_eq!(
            standard_round_sizes(Standard::En1505_1506),
            &STANDARD_ROUND_DUCT_SIZES[..]
        );
        assert_eq!(
            standard_rectangular_sizes(Standard::En1505_1506),
            &STANDARD_RECTANGULAR_DUCT_SIZES[..]
        );
    }

    #[test]
    fn nearest_round_up_and_closest() {
        // EN: 300 is an exact standard size.
        assert_eq!(
            nearest_round_size_for(Standard::En1505_1506, 300.0, true),
            300
        );
        assert_eq!(
            nearest_round_size_for(Standard::En1505_1506, 300.0, false),
            300
        );
        // 120 rounds up to 125 but closest is between 100 and 125...
        assert_eq!(
            nearest_round_size_for(Standard::En1505_1506, 120.0, true),
            125
        );
        assert_eq!(
            nearest_round_size_for(Standard::En1505_1506, 110.0, false),
            100
        );
        // Clamp at the ends.
        assert_eq!(
            nearest_round_size_for(Standard::En1505_1506, 10.0, true),
            63
        );
        assert_eq!(
            nearest_round_size_for(Standard::En1505_1506, 9999.0, true),
            1250
        );
    }

    #[test]
    fn ashrae_vs_en_expected_diameters() {
        // ASHRAE is inch-based: 12" = 305 mm, EN 1506 has an exact 300.
        assert_eq!(nearest_round_size_for(Standard::AsHrae, 300.0, true), 305);
        assert_eq!(
            nearest_round_size_for(Standard::En1505_1506, 300.0, true),
            300
        );
        // 9" = 229 mm, EN picks 250.
        assert_eq!(nearest_round_size_for(Standard::AsHrae, 228.0, true), 229);
        assert_eq!(
            nearest_round_size_for(Standard::En1505_1506, 228.0, true),
            250
        );
        // 6" = 152 mm, EN 1506 has no 150-sized gap to 160; 150 -> 150 exactly.
        assert_eq!(nearest_round_size_for(Standard::AsHrae, 150.0, true), 152);
        assert_eq!(
            nearest_round_size_for(Standard::En1505_1506, 150.0, true),
            150
        );
    }

    #[test]
    fn din_round_renard_series() {
        // DIN round uses Renard R10 figures; 250 is exact, 300 -> 315.
        assert_eq!(nearest_round_size_for(Standard::Din, 250.0, true), 250);
        assert_eq!(nearest_round_size_for(Standard::Din, 300.0, true), 315);
        assert_eq!(nearest_round_size_for(Standard::Din, 300.0, false), 315);
    }
}
