//! Friction, losses and flex-duct corrections.

pub mod flex;
pub mod friction;
pub mod losses;

pub use flex::stretch_correction_factor;
pub use friction::{
    friction_factor, friction_factor_colebrook, relative_roughness, reynolds, LAMINAR_RE_LIMIT,
};
pub use losses::{local_pressure_drop, straight_pressure_drop};
