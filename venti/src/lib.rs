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
pub mod catalog;
pub mod clash;
pub mod components;
pub mod core;
pub mod data;
pub mod development;
pub mod electrical;
pub mod error;
pub mod fabrication;
pub mod fan;
pub mod ffi;
pub mod insulation;
pub mod network;
pub mod physics;
pub mod re;
pub mod results;
pub mod room;
pub mod settings;
pub mod sizing;
pub mod sound;
pub mod standards;
pub mod topology;
pub mod units;

#[cfg(feature = "cli")]
pub mod io;

#[cfg(feature = "export")]
pub mod export;

// ---- top-level re-exports (mirrors `python/wenta/__init__.py`) ------------

pub use balancing::{balancing_zeta, balancing_zeta_batch, damper_open_percentage, required_zeta};
pub use catalog::{reference_catalog, FittingCategory, VelocityRef, ZetaCatalog, ZetaEntry};
#[cfg(feature = "cli")]
pub use catalog::{vendor_catalog_from_file, vendor_catalog_from_json, VendorCatalog};
pub use clash::{clash_count, clashes_as_csv, find_clashes, Clash};
pub use components::base::{Component, Port, PortDirection};
pub use components::duct::{FlexDuct, RigidDuct};
pub use components::elbow::{ElbowRound, ANGLE_GRID, RD_GRID, ZETA_TABLE};
pub use components::fitting::{Source, Tee, Terminal, TwoPortFitting};
pub use components::fittings_library::{
    attenuator, attenuator_open, cross_fitting, damper_butterfly, diffuser_ceiling, elbow_round,
    expander_rectangular, expander_round, filter_bank, fire_damper, grille_return,
    junction_tee_branch, junction_tee_combine, louver_open, mitered_elbow, named_zeta,
    rectangular_elbow, reducer_rectangular, reducer_round, round_tap_branch, taper_transition,
    NAMED_FITTING_ZETAS,
};
pub use core::fluid::{air_at_altitude, Fluid, STANDARD_AIR};
pub use core::geometry::{equivalent_round_diameter, CrossSection, Rectangular, Round};
pub use data::standard_sizes::{
    nearest_round_size, standard_rectangular_sections, standard_round_sections,
    STANDARD_RECTANGULAR_DUCT_SIZES, STANDARD_ROUND_BRANCH_SIZES, STANDARD_ROUND_DUCT_SIZES,
    STANDARD_ROUND_TRANSFORMATION_SIZES,
};
pub use development::{
    reducer_cone_development, round_duct_development, round_elbow_development, FlatPiece,
};
pub use electrical::{electrical_as_csv, ElectricalData, ElectricalSchedule};

pub use error::{Error, Result};
#[cfg(feature = "export")]
pub use export::{
    electrical_schedule_to_pdf, electrical_schedule_to_xlsx, schedule_to_pdf_bytes,
    schedule_to_xlsx_bytes,
};
pub use fabrication::{
    cutting_schedule, duct_surface_area_m2, duct_weight_kg, FabricationBreakout,
};
pub use fan::margin_pa as margin;
pub use fan::{fan_power, margin_pa, pick_fan, FanCurve, FanPoint};
pub use network::{
    batch_compute, compute_pressure_drops, critical_path, critical_path_pressure_drop,
    critical_path_sum, port_node_id, propagate_flowrates, simple_supply_network, solve,
    ComponentEnum, Network, TAG_FITTING, TAG_FLEX, TAG_RIGID, TAG_SOURCE, TAG_TEE, TAG_TERMINAL,
};
pub use re::{corrected_zeta, elbow_round_loss, re_correction, size_correction, D_REF_M, RE_REF};
pub use results::{extract_results, results_as_csv, results_summary, ComponentResult};
pub use room::{room_ach, RoomBalance, RoomBalanceSet};
#[cfg(feature = "cli")]
pub use settings::{settings_from_json, settings_to_json};
pub use settings::{ProjectSettings, Units};
pub use standards::{nearest_round_size_for, rect_sizes_mm, round_sizes_mm, Standard};
pub use topology::{trace, Polyline, Segment, TraceOptions, TracedSystem};

#[cfg(feature = "cli")]
pub use results::{report_json_string, results_as_json_rows};

#[cfg(feature = "cli")]
pub use io::{
    load_network_from_file, load_network_from_path, load_network_from_str, save_network_to_file,
    save_network_to_json_string, save_network_to_path, save_network_to_string,
};
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
