//! Ductwork components.

pub mod base;
pub mod duct;
pub mod elbow;
pub mod fitting;
pub mod fittings_library;

pub use base::{Component, Port, PortDirection};
pub use duct::{FlexDuct, RigidDuct};
pub use elbow::{ElbowRound, ANGLE_GRID, RD_GRID, ZETA_TABLE};
pub use fitting::{Source, Tee, Terminal, TwoPortFitting};
pub use fittings_library::{
    damper_butterfly, diffuser_ceiling, expander_round, grille_return, junction_tee_branch,
    junction_tee_combine, mitered_elbow, rectangular_elbow, reducer_round,
};
