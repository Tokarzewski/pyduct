//! Cross-section geometry primitives.
//!
//! A cross-section is an immutable value that knows its area and hydraulic
//! diameter. Both are computed once at construction time and cached, so
//! hot-path sizing loops avoid repeated `f64` math.

use crate::Result;
use std::f64::consts::PI;

/// A duct cross-section: either a `Round` or a `Rectangular` shape.
///
/// This enum mirrors the duck-typed `CrossSection` base class in the Python
/// reference so the sizing API can work uniformly over round and rectangular
/// ducts without needing a trait object.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CrossSection {
    Round(Round),
    Rectangular(Rectangular),
}

impl CrossSection {
    /// Cross-sectional area [m²].
    #[inline]
    pub fn area(&self) -> f64 {
        match self {
            CrossSection::Round(r) => r.area,
            CrossSection::Rectangular(r) => r.area,
        }
    }

    /// Hydraulic diameter D_h [m].
    #[inline]
    pub fn hydraulic_diameter(&self) -> f64 {
        match self {
            CrossSection::Round(r) => r.hydraulic_diameter,
            CrossSection::Rectangular(r) => r.hydraulic_diameter,
        }
    }

    /// The larger side/width dimension [m] (a round duct returns diameter).
    #[inline]
    pub fn width(&self) -> f64 {
        match self {
            CrossSection::Round(r) => r.diameter,
            CrossSection::Rectangular(r) => r.width,
        }
    }

    /// The smaller side/height dimension [m] (a round duct returns diameter).
    #[inline]
    pub fn height(&self) -> f64 {
        match self {
            CrossSection::Round(r) => r.diameter,
            CrossSection::Rectangular(r) => r.height,
        }
    }
}

/// Circular duct cross-section.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Round {
    pub diameter: f64,           // [m]
    pub area: f64,               // [m²]
    pub hydraulic_diameter: f64, // [m]
}

impl Round {
    /// Construct a round cross-section from a diameter [m].
    ///
    /// # Examples
    /// ```
    /// use venti::Round;
    /// let r = Round::new(0.2).unwrap();
    /// assert!((r.area - std::f64::consts::PI * 0.01).abs() < 1e-12);
    /// assert_eq!(r.hydraulic_diameter, 0.2);
    /// ```
    pub fn new(diameter: f64) -> Result<Self> {
        if diameter <= 0.0 {
            return Err("diameter must be positive".into());
        }
        let r = diameter * 0.5;
        Ok(Round {
            diameter,
            area: PI * r * r,
            // For a round duct the hydraulic diameter equals the diameter.
            hydraulic_diameter: diameter,
        })
    }
}

/// Rectangular duct cross-section.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rectangular {
    pub width: f64,              // [m]
    pub height: f64,             // [m]
    pub area: f64,               // [m²]
    pub hydraulic_diameter: f64, // [m]
}

impl Rectangular {
    pub fn new(width: f64, height: f64) -> Result<Self> {
        if width <= 0.0 || height <= 0.0 {
            return Err("width and height must be positive".into());
        }
        let area = width * height;
        // D_h = 4 A / P = 2 W H / (W + H)
        let hydraulic_diameter = 2.0 * width * height / (width + height);
        Ok(Rectangular {
            width,
            height,
            area,
            hydraulic_diameter,
        })
    }
}

/// ASHRAE equivalent round diameter for a rectangular duct.
///
/// ```text
/// D_eq = 1.30 * (a*b)**0.625 / (a + b)**0.25     [m]
/// ```
pub fn equivalent_round_diameter(width: f64, height: f64) -> Result<f64> {
    if width <= 0.0 || height <= 0.0 {
        return Err("width and height must be positive".into());
    }
    Ok(1.30 * (width * height).powf(0.625) / (width + height).powf(0.25))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_area_and_dh() {
        // D = 0.2 -> area = pi*0.01, Dh = 0.2
        let r = Round::new(0.2).unwrap();
        assert!((r.area - std::f64::consts::PI * 0.01).abs() < 1e-15);
        assert_eq!(r.hydraulic_diameter, 0.2);
        assert_eq!(r.diameter, 0.2);
    }

    #[test]
    fn rect_area_and_dh() {
        // W=0.3, H=0.2 -> area=0.06, Dh=2*0.06/0.5=0.24
        let r = Rectangular::new(0.3, 0.2).unwrap();
        assert!((r.area - 0.06).abs() < 1e-15);
        assert!((r.hydraulic_diameter - 0.24).abs() < 1e-15);
    }

    #[test]
    fn round_rejects_nonpositive() {
        assert!(Round::new(0.0).is_err());
        assert!(Round::new(-1.0).is_err());
    }

    #[test]
    fn equivalent_round_diameter_matches_reference() {
        // Reference (Python/Mojo) value for 0.3 x 0.2 m.
        let d = equivalent_round_diameter(0.3, 0.2).unwrap();
        let expected = 1.30 * (0.3f64 * 0.2).powf(0.625) / (0.5f64).powf(0.25);
        assert!((d - expected).abs() < 1e-15);
    }
}
