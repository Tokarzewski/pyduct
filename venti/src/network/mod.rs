//! Network model and solver.

#[allow(clippy::module_inception)]
pub mod network;
pub mod solver;

pub use network::{port_node_id, simple_supply_network, ComponentEnum, Network};
pub use solver::{
    batch_compute, compute_pressure_drops, critical_path, critical_path_pressure_drop,
    critical_path_sum, propagate_flowrates, solve, TAG_FITTING, TAG_FLEX, TAG_RIGID, TAG_SOURCE,
    TAG_TEE, TAG_TERMINAL,
};
