//! Core value objects: fluids and cross-section geometry.
//!
//! Mirrors `python/wenta/core/` and `wentamojo/core/`.

pub mod fluid;
pub mod geometry;

pub use fluid::{air_at_altitude, Fluid, STANDARD_AIR};
pub use geometry::{equivalent_round_diameter, CrossSection, Rectangular, Round};
