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
    attenuator, attenuator_open, cross_fitting, damper_butterfly, diffuser_ceiling, elbow_round,
    expander_rectangular, expander_round, filter_bank, fire_damper, grille_return,
    junction_tee_branch, junction_tee_combine, louver_open, mitered_elbow, named_zeta,
    rectangular_elbow, reducer_rectangular, reducer_round, round_tap_branch, taper_transition,
    NAMED_FITTING_ZETAS,
};
