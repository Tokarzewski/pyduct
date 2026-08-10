//! # venti
//!
//! Ductwork design library — sizing, pressure-drop, fitting losses, network
//! solving. This is the Rust port of the **wenta** Python reference and the
//! **wentamojo** Mojo port in the `pyduct` repository.
//!
//! The crate mirrors the reference module layout so the two can be diff-tested
//! against each other over a shared corpus of inputs:
//!
//! ```text
//! venti/
//! ├── core/       geometry (Round / Rectangular) + fluid properties
//! ├── physics/    friction, losses, flex-duct corrections
//! ├── data/       EN 1505/1506 standard sizes
//! ├── units.rs    unit converters + ACH
//! ├── sizing.rs   velocity / EF / budget / noise / aspect-ratio sizing
//! ├── components/ ducts, fittings, terminals, elbow + fittings library
//! └── network/    graph model + solver (critical-path DP, batch kernel)
//! ```
//!
//! ## Quick start
//!
//! ```rust
//! use venti::{
//!     Network, ComponentEnum, Source, RigidDuct, Terminal, Round, velocity_method_round,
//! };
//!
//! // Size a round duct for 0.1 m^3/s at a target velocity of 4 m/s.
//! let (section, v) = velocity_method_round(0.1, 4.0).unwrap();
//! assert!(v <= 4.0);
//!
//! // Or solve a small network end to end.
//! let r = Round::new(0.2).unwrap();
//! let mut net = Network::new("example");
//! net.add("ahu",  ComponentEnum::Source(Source::new("AHU"))).unwrap();
//! net.add("duct", ComponentEnum::RigidDuct(RigidDuct::new(
//!     "duct", r.area, r.hydraulic_diameter, 20.0, 0.0001,
//! ).unwrap())).unwrap();
//! net.add("term", ComponentEnum::Terminal(Terminal::new(
//!     "terminal", 0.1, Some(r.area), 1.0,
//! ))).unwrap();
//! net.connect("ahu", "duct").unwrap();
//! net.connect("duct", "term").unwrap();
//! let dp_pa = net.solve(None).unwrap();
//! assert!(dp_pa > 0.0);
//! ```

pub mod balancing;
pub mod components;
pub mod core;
pub mod data;
pub mod ffi;
pub mod network;
pub mod physics;
pub mod results;
pub mod sizing;
pub mod sound;
pub mod units;

#[cfg(feature = "cli")]
pub mod io;

// ---- top-level re-exports (mirrors `python/wenta/__init__.py`) ------------

pub use balancing::{
    balancing_zeta, balancing_zeta_batch, damper_open_percentage, required_zeta,
};
pub use components::base::{Component, Port, PortDirection};
pub use components::duct::{FlexDuct, RigidDuct};
pub use components::elbow::{ElbowRound, ANGLE_GRID, RD_GRID, ZETA_TABLE};
pub use components::fitting::{Source, Tee, Terminal, TwoPortFitting};
pub use components::fittings_library::{
    damper_butterfly, diffuser_ceiling, expander_round, grille_return, junction_tee_branch,
    junction_tee_combine, mitered_elbow, rectangular_elbow, reducer_round,
};
pub use core::fluid::{air_at_altitude, Fluid, STANDARD_AIR};
pub use core::geometry::{equivalent_round_diameter, CrossSection, Rectangular, Round};
pub use data::standard_sizes::{
    nearest_round_size, standard_rectangular_sections, standard_round_sections,
    STANDARD_RECTANGULAR_DUCT_SIZES, STANDARD_ROUND_BRANCH_SIZES, STANDARD_ROUND_DUCT_SIZES,
    STANDARD_ROUND_TRANSFORMATION_SIZES,
};
pub use network::{
    batch_compute, compute_pressure_drops, critical_path, critical_path_pressure_drop,
    critical_path_sum, port_node_id, propagate_flowrates, simple_supply_network, solve,
    ComponentEnum, Network, TAG_FITTING, TAG_FLEX, TAG_RIGID, TAG_SOURCE, TAG_TEE, TAG_TERMINAL,
};
pub use results::{extract_results, results_as_csv, results_summary, ComponentResult};

#[cfg(feature = "cli")]
pub use results::{report_json_string, results_as_json_rows};

#[cfg(feature = "cli")]
pub use io::{load_network_from_file, load_network_from_path, load_network_from_str};
pub use sizing::{
    aspect_ratio_method, equal_friction_method, equal_friction_method_rectangular,
    equal_friction_method_round, noise_limit_method, pressure_drop_budget,
    pressure_drop_budget_rectangular, pressure_drop_budget_round, velocity_method,
    velocity_method_batch, velocity_method_rectangular, velocity_method_round, Shape,
    NOISE_LIMITS_M_S,
};
pub use sound::{
    duct_pressure_level, nc_ok, nc_ok_target, regenerated_noise_round, NOISE_LIMITS_NC,
};
pub use units::{
    air_changes_per_hour, c_to_f, cfm_to_m3s, f_to_c, fpm_to_ms, ft_to_m, in_to_m, inwc_to_pa,
    m3s_to_cfm, m_to_ft, m_to_in, ms_to_fpm, pa_to_inwc,
};

/// Library version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
