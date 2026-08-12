//! C-ABI exports for embedding `venti` as a WebAssembly (or native `cdylib`)
//! core from another host language: C#, Python, C++, Node, etc.
//!
//! Every function is `extern "C"` + `#[no_mangle]`, so they are the *only*
//! symbols a `cdylib` build exports. Multi-valued results are written through
//! caller-allocated `*mut f64` out-params (WASM's C ABI cannot return structs
//! with more than two f64 words by value). Functions that can reject their
//! input return an `i32` status code: `0` = ok, nonzero = error.
//!
//! Build the embeddable core:
//!
//! ```text
//! rustup target add wasm32-wasip1
//! cargo build --release --target wasm32-wasip1 --no-default-features
//! # -> target/wasm32-wasip1/release/libventi.wasm
//! ```

// These `#[no_mangle] extern "C"` exports receive raw pointers from the host.
// Their contract (valid pointers, correct lengths) is documented here; hosts
// are other programming languages, so the missing-`# Safety` lint is
// suppressed once for this whole module.
#![allow(clippy::missing_safety_doc)]

use crate::Result;
use core::slice;

use std::string::String;
use std::sync::{Mutex, OnceLock};

use crate::balancing::{balancing_zeta, damper_open_percentage, required_zeta};
use crate::components::fittings_library::{
    damper_butterfly, diffuser_ceiling, elbow_round, expander_rectangular, expander_round,
    filter_bank, grille_return, junction_tee_branch, junction_tee_combine, louver_open,
    mitered_elbow, named_zeta, rectangular_elbow, reducer_rectangular, reducer_round,
    round_tap_branch,
};
use crate::core::fluid::{self, air_at_altitude};
use crate::core::geometry::{self, Round};
use crate::network::solver::{batch_compute as kernel_batch, critical_path_sum as kernel_cp};
use crate::network::{ComponentEnum, Network};
use crate::physics::flex::stretch_correction_factor;
use crate::physics::friction::{
    friction_factor, friction_factor_colebrook, relative_roughness, reynolds,
};
use crate::physics::losses::{local_pressure_drop, straight_pressure_drop};
use crate::sizing::{
    aspect_ratio_method, equal_friction_method_round, velocity_method_batch, velocity_method_round,
};
use crate::sound::{duct_pressure_level, nc_ok, regenerated_noise_round};

// ---- core: geometry ------------------------------------------------------

/// Round duct cross-sectional area [m²].
#[no_mangle]
pub extern "C" fn venti_round_area(diameter: f64) -> f64 {
    match Round::new(diameter) {
        Ok(r) => r.area,
        Err(_) => f64::NAN,
    }
}

/// Round duct hydraulic diameter [m] (= diameter).
#[no_mangle]
pub extern "C" fn venti_round_hydraulic_diameter(diameter: f64) -> f64 {
    diameter
}

/// Rectangular duct cross-sectional area [m²] = w·h.
#[no_mangle]
pub extern "C" fn venti_rect_area(width: f64, height: f64) -> f64 {
    width * height
}

/// Rectangular duct hydraulic diameter [m] = 2·W·H/(W+H).
#[no_mangle]
pub extern "C" fn venti_rect_hydraulic_diameter(width: f64, height: f64) -> f64 {
    2.0 * width * height / (width + height)
}

/// ASHRAE equivalent round diameter. Writes `*out`. Returns status.
#[no_mangle]
pub unsafe extern "C" fn venti_equivalent_round_diameter(
    width: f64,
    height: f64,
    out: *mut f64,
) -> i32 {
    unsafe {
        match geometry::equivalent_round_diameter(width, height) {
            Ok(v) => {
                *out = v;
                0
            }
            Err(_) => 1,
        }
    }
}

// ---- core: fluid ---------------------------------------------------------

/// Standard air density [kg/m³] (20 °C, 101 325 Pa).
#[no_mangle]
pub extern "C" fn venti_standard_air_density() -> f64 {
    fluid::STANDARD_AIR.density
}

/// Standard air dynamic viscosity [Pa·s].
#[no_mangle]
pub extern "C" fn venti_standard_air_dynamic_viscosity() -> f64 {
    fluid::STANDARD_AIR.dynamic_viscosity
}

/// Air properties at altitude; writes `*out_density`, `*out_viscosity`.
#[no_mangle]
pub unsafe extern "C" fn venti_air_at_altitude(
    altitude_m: f64,
    temperature_c: f64,
    out_density: *mut f64,
    out_viscosity: *mut f64,
) -> i32 {
    unsafe {
        match air_at_altitude(altitude_m, temperature_c) {
            Ok(f) => {
                *out_density = f.density;
                *out_viscosity = f.dynamic_viscosity;
                0
            }
            Err(_) => 1,
        }
    }
}

// ---- physics: friction / losses / flex -----------------------------------

#[no_mangle]
pub extern "C" fn venti_reynolds(
    velocity: f64,
    hydraulic_diameter: f64,
    kinematic_viscosity: f64,
) -> f64 {
    reynolds(velocity, hydraulic_diameter, kinematic_viscosity)
}

#[no_mangle]
pub extern "C" fn venti_relative_roughness(
    absolute_roughness: f64,
    hydraulic_diameter: f64,
) -> f64 {
    relative_roughness(absolute_roughness, hydraulic_diameter)
}

#[no_mangle]
pub extern "C" fn venti_friction_factor(reynolds_number: f64, rel_roughness: f64) -> f64 {
    friction_factor(reynolds_number, rel_roughness)
}

#[no_mangle]
pub extern "C" fn venti_friction_factor_colebrook(
    reynolds_number: f64,
    rel_roughness: f64,
    tol: f64,
    max_iter: i32,
) -> f64 {
    friction_factor_colebrook(
        reynolds_number,
        rel_roughness,
        tol,
        max_iter.max(1) as usize,
    )
}

#[no_mangle]
pub extern "C" fn venti_straight_pressure_drop(
    friction_factor: f64,
    length: f64,
    hydraulic_diameter: f64,
    velocity: f64,
    density: f64,
) -> f64 {
    straight_pressure_drop(
        friction_factor,
        length,
        hydraulic_diameter,
        velocity,
        density,
    )
}

#[no_mangle]
pub extern "C" fn venti_local_pressure_drop(zeta: f64, velocity: f64, density: f64) -> f64 {
    local_pressure_drop(zeta, velocity, density)
}

#[no_mangle]
pub extern "C" fn venti_stretch_correction_factor(diameter: f64, stretch_percentage: f64) -> f64 {
    stretch_correction_factor(diameter, stretch_percentage)
}

// ---- sizing --------------------------------------------------------------

/// Velocity-method round sizing. Writes diameter [m] and velocity.
#[no_mangle]
pub unsafe extern "C" fn venti_velocity_method_round(
    flowrate: f64,
    target_velocity: f64,
    out_diameter_m: *mut f64,
    out_velocity: *mut f64,
) -> i32 {
    unsafe {
        match velocity_method_round(flowrate, target_velocity) {
            Ok((section, v)) => {
                *out_diameter_m = section.width();
                *out_velocity = v;
                0
            }
            Err(_) => 1,
        }
    }
}

/// Equal-friction round sizing. Writes diameter [m], velocity, ΔP/m.
#[no_mangle]
pub unsafe extern "C" fn venti_equal_friction_method_round(
    flowrate: f64,
    target_pressure_drop_per_meter: f64,
    absolute_roughness: f64,
    density: f64,
    dynamic_viscosity: f64,
    out_diameter_m: *mut f64,
    out_velocity: *mut f64,
    out_dp_per_m: *mut f64,
) -> i32 {
    unsafe {
        let fluid_ = fluid::Fluid::new(density, dynamic_viscosity);
        let fluid = match fluid_ {
            Ok(f) => f,
            Err(_) => return 1,
        };
        match equal_friction_method_round(
            flowrate,
            target_pressure_drop_per_meter,
            absolute_roughness,
            &fluid,
        ) {
            Ok((section, v, r)) => {
                *out_diameter_m = section.width();
                *out_velocity = v;
                *out_dp_per_m = r;
                0
            }
            Err(_) => 1,
        }
    }
}

/// Aspect-ratio rectangular sizing. Writes width, height [m], velocity.
#[no_mangle]
pub unsafe extern "C" fn venti_aspect_ratio_method(
    flowrate: f64,
    target_velocity: f64,
    aspect_ratio: f64,
    out_width: *mut f64,
    out_height: *mut f64,
    out_velocity: *mut f64,
) -> i32 {
    unsafe {
        match aspect_ratio_method(flowrate, target_velocity, aspect_ratio) {
            Ok((section, v)) => {
                *out_width = section.width();
                *out_height = section.height();
                *out_velocity = v;
                0
            }
            Err(_) => 1,
        }
    }
}

/// Batch round sizing; `flows`[0..n] in, writes diameters [m] + velocities.
#[no_mangle]
pub unsafe extern "C" fn venti_velocity_method_batch(
    flows: *const f64,
    n: i32,
    target_velocity: f64,
    out_diameters_m: *mut f64,
    out_velocities: *mut f64,
) -> i32 {
    let n = n.max(0) as usize;
    let flows = unsafe { slice::from_raw_parts(flows, n) };
    match velocity_method_batch(flows.iter(), target_velocity) {
        Ok((d, v)) => {
            unsafe {
                for k in 0..n {
                    *out_diameters_m.add(k) = d[k] / 1000.0; // mm -> m
                    *out_velocities.add(k) = v[k];
                }
            }
            0
        }
        Err(_) => 1,
    }
}

// ---- fittings library (pure, no error in practice) ------------------------

#[no_mangle]
pub extern "C" fn venti_rectangular_elbow(
    width: f64,
    height: f64,
    bend_radius: f64,
    angle_deg: f64,
) -> f64 {
    rectangular_elbow(width, height, bend_radius, angle_deg).unwrap_or(f64::NAN)
}

#[no_mangle]
pub extern "C" fn venti_reducer_round(d_inlet: f64, d_outlet: f64, angle_deg: f64) -> f64 {
    reducer_round(d_inlet, d_outlet, angle_deg).unwrap_or(f64::NAN)
}

#[no_mangle]
pub extern "C" fn venti_expander_round(d_inlet: f64, d_outlet: f64, angle_deg: f64) -> f64 {
    expander_round(d_inlet, d_outlet, angle_deg).unwrap_or(f64::NAN)
}

#[no_mangle]
pub extern "C" fn venti_damper_butterfly(open_percentage: f64) -> f64 {
    damper_butterfly(open_percentage).unwrap_or(f64::NAN)
}

#[no_mangle]
pub extern "C" fn venti_diffuser_ceiling(area_throw: f64) -> f64 {
    diffuser_ceiling(area_throw).unwrap_or(f64::NAN)
}

#[no_mangle]
pub extern "C" fn venti_grille_return(blockage_factor: f64) -> f64 {
    grille_return(blockage_factor).unwrap_or(f64::NAN)
}

#[no_mangle]
pub extern "C" fn venti_mitered_elbow(angle_deg: f64, vaned: i32) -> f64 {
    mitered_elbow(angle_deg, vaned != 0).unwrap_or(f64::NAN)
}

/// Splitting tee, writes `*out_main`, `*out_branch`.
#[no_mangle]
pub unsafe extern "C" fn venti_junction_tee_branch(
    d_main: f64,
    d_branch: f64,
    flowrate_main: f64,
    flowrate_branch: f64,
    out_main: *mut f64,
    out_branch: *mut f64,
) -> i32 {
    unsafe {
        match junction_tee_branch(d_main, d_branch, flowrate_main, flowrate_branch) {
            Ok((m, b)) => {
                *out_main = m;
                *out_branch = b;
                0
            }
            Err(_) => 1,
        }
    }
}

/// Combining tee, writes `*out_main`, `*out_branch`.
#[no_mangle]
pub unsafe extern "C" fn venti_junction_tee_combine(
    d_main: f64,
    d_branch: f64,
    flowrate_main: f64,
    flowrate_branch: f64,
    out_main: *mut f64,
    out_branch: *mut f64,
) -> i32 {
    unsafe {
        match junction_tee_combine(d_main, d_branch, flowrate_main, flowrate_branch) {
            Ok((m, b)) => {
                *out_main = m;
                *out_branch = b;
                0
            }
            Err(_) => 1,
        }
    }
}

// ---- network kernels ------------------------------------------------------

/// Critical-path DP over a flat projection. Returns the total (max) ΔP.
///
/// `pred_counts[k]` = number of predecessors of node k, `pred_offsets[k]` the
/// start (prefix) index into `pred_flat` for node k's predecessor list.
#[no_mangle]
pub unsafe extern "C" fn venti_critical_path_sum(
    dp: *const f64,
    n: i32,
    pred_counts: *const i32,
    pred_offsets: *const i32,
    pred_flat: *const i32,
) -> f64 {
    let n = n.max(0) as usize;
    let dp = unsafe { slice::from_raw_parts(dp, n) };
    let counts = unsafe { slice::from_raw_parts(pred_counts, n) };
    let offsets = unsafe { slice::from_raw_parts(pred_offsets, n) };
    let total_preds: usize = counts.iter().map(|&c| c.max(0) as usize).sum();
    let flat = unsafe { slice::from_raw_parts(pred_flat, total_preds) };

    // Rebuild preds as Vec<Vec<usize>> (topo order = 0..n).
    let mut preds: Vec<Vec<usize>> = vec![Vec::new(); n];
    for k in 0..n {
        let off = offsets[k].max(0) as usize;
        let cnt = counts[k].max(0) as usize;
        for j in 0..cnt {
            preds[k].push(flat[off + j] as usize);
        }
    }
    let topo: Vec<usize> = (0..n).collect();
    kernel_cp(&topo, &preds, dp)
}

/// Batch pressure-drop pass over a flat component view. Mirrors the Mojo
/// `batch_compute` layout. Writes `*out_velocities`, `*out_dps` (length `p`).
#[no_mangle]
pub unsafe extern "C" fn venti_batch_compute(
    types: *const i32,
    n: i32,
    params: *const f64,
    port_idx: *const i32,
    flows: *const f64,
    p: i32,
    density: f64,
    kinematic_viscosity: f64,
    out_velocities: *mut f64,
    out_dps: *mut f64,
) -> i32 {
    let n = n.max(0) as usize;
    let p = p.max(0) as usize;
    let types32 = unsafe { slice::from_raw_parts(types, n) };
    let params = unsafe { slice::from_raw_parts(params, n * 6) };
    let port32 = unsafe { slice::from_raw_parts(port_idx, n * 3) };
    let flows = unsafe { slice::from_raw_parts(flows, p) };
    let types: Vec<i64> = types32.iter().map(|&x| x as i64).collect();
    let port_idx: Vec<i64> = port32.iter().map(|&x| x as i64).collect();
    let (v, d) = kernel_batch(
        &types,
        params,
        &port_idx,
        flows,
        density,
        kinematic_viscosity,
    );
    unsafe {
        for k in 0..p {
            *out_velocities.add(k) = v[k];
            *out_dps.add(k) = d[k];
        }
    }
    0
}

// ---- sound / acoustics ----------------------------------------------------

/// Regenerated (airflow) sound-power level of a straight round duct [dB re
/// 1e-12 W]. `density` <= 0 selects standard air; NAN on bad input.
#[no_mangle]
pub extern "C" fn venti_regenerated_noise_round(velocity: f64, diameter: f64, density: f64) -> f64 {
    let rho = if density > 0.0 { Some(density) } else { None };
    regenerated_noise_round(velocity, diameter, rho).unwrap_or(f64::NAN)
}

/// Convert a duct sound-*power* level into the reverberant sound-*pressure*
/// level [dB re 20 µPa] in a room. NAN on bad input.
#[no_mangle]
pub extern "C" fn venti_duct_pressure_level(
    sound_power_db: f64,
    room_area: f64,
    absorption: f64,
) -> f64 {
    duct_pressure_level(sound_power_db, room_area, absorption).unwrap_or(f64::NAN)
}

/// Check `level_db` against the NC target for `space_type` (a `(ptr, len)`
/// byte string, e.g. `"office"`). Writes `*out_ok` (1 = passes). Returns 0 on
/// success, nonzero for an unknown space type.
#[no_mangle]
pub unsafe extern "C" fn venti_nc_ok(
    space_ptr: *const u8,
    space_len: usize,
    level_db: f64,
    out_ok: *mut i32,
) -> i32 {
    let space = str_from(space_ptr, space_len);
    match nc_ok(&space, level_db) {
        Ok(ok) => {
            *out_ok = if ok { 1 } else { 0 };
            0
        }
        Err(_) => 1,
    }
}

// ---- balancing -------------------------------------------------------------

/// Loss coefficient (ζ) a damper must add to produce `required_dp_pa` of
/// pressure drop at the given velocity: ζ = 2·dp/(ρ·v²).
#[no_mangle]
pub extern "C" fn venti_required_zeta(required_dp_pa: f64, velocity: f64, density: f64) -> f64 {
    required_zeta(required_dp_pa, velocity, density)
}

/// Damper ζ that balances a branch whose available pressure is below its
/// requirement (0.0 when the branch is already met/over-supplied).
#[no_mangle]
pub extern "C" fn venti_balancing_zeta(
    total_req_pa: f64,
    branch_avail_pa: f64,
    velocity: f64,
    density: f64,
) -> f64 {
    balancing_zeta(total_req_pa, branch_avail_pa, velocity, density)
}

/// Invert the butterfly-damper correlation: ζ -> open percentage [0, 100].
#[no_mangle]
pub extern "C" fn venti_damper_open_percentage(zeta: f64) -> f64 {
    damper_open_percentage(zeta)
}

// ---- Re/size-corrected fitting losses (venti::re) --------------------------

/// Re- and size-corrected smooth round-elbow loss coefficient (NAN on bad input).
#[no_mangle]
pub extern "C" fn venti_elbow_round_loss(
    bend_radius: f64,
    diameter: f64,
    angle_deg: f64,
    velocity: f64,
    density: f64,
    dynamic_viscosity: f64,
) -> f64 {
    crate::re::elbow_round_loss(
        bend_radius,
        diameter,
        angle_deg,
        velocity,
        density,
        dynamic_viscosity,
    )
    .unwrap_or(f64::NAN)
}

// ---- expanded fitting library (round-trip into the WASM core) -------------

#[no_mangle]
pub extern "C" fn venti_elbow_round(bend_radius: f64, diameter: f64, angle_deg: f64) -> f64 {
    elbow_round(bend_radius, diameter, angle_deg).unwrap_or(f64::NAN)
}

#[no_mangle]
pub extern "C" fn venti_reducer_rectangular(
    w_in: f64,
    h_in: f64,
    w_out: f64,
    h_out: f64,
    angle_deg: f64,
) -> f64 {
    reducer_rectangular(w_in, h_in, w_out, h_out, angle_deg).unwrap_or(f64::NAN)
}

#[no_mangle]
pub extern "C" fn venti_expander_rectangular(
    w_in: f64,
    h_in: f64,
    w_out: f64,
    h_out: f64,
    angle_deg: f64,
) -> f64 {
    expander_rectangular(w_in, h_in, w_out, h_out, angle_deg).unwrap_or(f64::NAN)
}

#[no_mangle]
pub extern "C" fn venti_louver_open(open_percentage: f64) -> f64 {
    louver_open(open_percentage).unwrap_or(f64::NAN)
}

#[no_mangle]
pub extern "C" fn venti_filter_bank(open_fraction: f64) -> f64 {
    filter_bank(open_fraction).unwrap_or(f64::NAN)
}

#[no_mangle]
pub extern "C" fn venti_round_tap_branch(d_main: f64, d_tap: f64, split_ratio: f64) -> f64 {
    round_tap_branch(d_main, d_tap, split_ratio).unwrap_or(f64::NAN)
}

/// Look up a named constant zeta; writes `out_ok = 1` when found.
#[no_mangle]
pub unsafe extern "C" fn venti_named_zeta(
    name: *const u8,
    name_len: usize,
    out: *mut f64,
    out_ok: *mut i32,
) {
    let name = str_from(name, name_len);
    match named_zeta(&name) {
        Some(z) => {
            *out = z;
            *out_ok = 1;
        }
        None => {
            *out = f64::NAN;
            *out_ok = 0;
        }
    }
}

// ---------------------------------------------------------------------------
// Handle-based network API
// ---------------------------------------------------------------------------
// Lets a host build a `venti::Network`, solve it, and read per-component
// results entirely through the C ABI — the piece the CAD plugin needs to push
// networks across the WASM boundary (issue #12).
//
// Network lifecycle: `venti_network_create` returns a non-negative handle;
// every other function takes that handle as its first `i32` argument. Call
// `venti_network_free` when done.
//
// Component type tags (match the Mojo `batch_compute` tags):
//   0 Source         params: [ -, -, -, -, -, - ]
//   1 Terminal       params: [ area_or_0, zeta, flowrate, -, -, - ]
//   2 RigidDuct      params: [ area, dh, length, abs_rough, -, - ]
//   3 FlexDuct       params: [ -, diameter, length, pdpm, stretch%, - ]
//   4 TwoPortFitting params: [ area, zeta, -, -, -, - ]
//   5 Tee            params: [ area, zeta_straight, zeta_branch, -, -, - ]
//
// Strings (network name, component id/name, connection endpoints, result
// fields) are passed as `(ptr, len)` bytes in the host's memory space.

const CTYPE_SOURCE: i32 = 0;
const CTYPE_TERMINAL: i32 = 1;
const CTYPE_RIGID: i32 = 2;
const CTYPE_FLEX: i32 = 3;
const CTYPE_FITTING: i32 = 4;
const CTYPE_TEE: i32 = 5;

fn registry() -> &'static Mutex<Vec<Option<Network>>> {
    static REG: OnceLock<Mutex<Vec<Option<Network>>>> = OnceLock::new();
    REG.get_or_init(|| Mutex::new(Vec::new()))
}

/// Number of handle slots currently allocated (for diagnostics).
#[no_mangle]
pub extern "C" fn venti_network_slots() -> i32 {
    registry().lock().unwrap().len() as i32
}

/// Create a network and return its handle (or -1 on failure).
#[no_mangle]
pub unsafe extern "C" fn venti_network_create(name: *const u8, name_len: usize) -> i32 {
    let name = str_from(name, name_len);
    let mut reg = registry().lock().unwrap();
    let handle = reg.len();
    reg.push(Some(Network::new(&name)));
    handle as i32
}

/// Free a network handle. Returns 0 on success, nonzero on invalid handle.
#[no_mangle]
pub unsafe extern "C" fn venti_network_free(handle: i32) -> i32 {
    let mut reg = registry().lock().unwrap();
    match reg.get_mut(handle as usize) {
        Some(slot) => {
            *slot = None;
            0
        }
        None => -1,
    }
}

/// Add a component under `id`. Returns 0 on success, nonzero on error.
#[no_mangle]
pub unsafe extern "C" fn venti_network_add(
    handle: i32,
    id: *const u8,
    id_len: usize,
    name: *const u8,
    name_len: usize,
    ctype: i32,
    params: *const f64,
) -> i32 {
    let id = str_from(id, id_len);
    let cname = str_from(name, name_len);
    let pp = unsafe { slice::from_raw_parts(params, 6) };
    let comp = match component_from_ffi(ctype, &cname, pp) {
        Ok(c) => c,
        Err(_) => return -2,
    };
    let mut reg = registry().lock().unwrap();
    match reg.get_mut(handle as usize).and_then(|s| s.as_mut()) {
        Some(net) => match net.add(&id, comp) {
            Ok(_) => 0,
            Err(_) => -3,
        },
        None => -1,
    }
}

/// Connect two component endpoints (`source` -> `target`). Returns 0 / error.
#[no_mangle]
pub unsafe extern "C" fn venti_network_connect(
    handle: i32,
    source: *const u8,
    source_len: usize,
    target: *const u8,
    target_len: usize,
) -> i32 {
    let src = str_from(source, source_len);
    let tgt = str_from(target, target_len);
    let mut reg = registry().lock().unwrap();
    match reg.get_mut(handle as usize).and_then(|s| s.as_mut()) {
        Some(net) => match net.connect(&src, &tgt) {
            Ok(_) => 0,
            Err(_) => -4,
        },
        None => -1,
    }
}

/// Number of components in the network (or -1 on invalid handle).
#[no_mangle]
pub unsafe extern "C" fn venti_network_component_count(handle: i32) -> i32 {
    let reg = registry().lock().unwrap();
    match reg.get(handle as usize).and_then(|s| s.as_ref()) {
        Some(net) => net.len() as i32,
        None => -1,
    }
}

/// Validate the network; writes the number of structural problems found.
/// Returns 0 on success (even if problems exist), nonzero on bad handle.
#[no_mangle]
pub unsafe extern "C" fn venti_network_validate(handle: i32, out_problem_count: *mut i32) -> i32 {
    let reg = registry().lock().unwrap();
    match reg.get(handle as usize).and_then(|s| s.as_ref()) {
        Some(net) => {
            let problems = net.validate();
            *out_problem_count = problems.len() as i32;
            0
        }
        None => -1,
    }
}

/// Solve the network for the given fluid and return the critical-path ΔP [Pa]
/// (f64::NAN on error).
#[no_mangle]
pub unsafe extern "C" fn venti_network_solve(
    handle: i32,
    density: f64,
    dynamic_viscosity: f64,
) -> f64 {
    let fluid = match fluid::Fluid::new(density, dynamic_viscosity) {
        Ok(f) => f,
        Err(_) => return f64::NAN,
    };
    let mut reg = registry().lock().unwrap();
    match reg.get_mut(handle as usize).and_then(|s| s.as_mut()) {
        Some(net) => net.solve(Some(&fluid)).unwrap_or(f64::NAN),
        None => f64::NAN,
    }
}

/// Number of result rows (one per component), or -1 on bad handle.
#[no_mangle]
pub unsafe extern "C" fn venti_results_count(handle: i32) -> i32 {
    let reg = registry().lock().unwrap();
    match reg.get(handle as usize).and_then(|s| s.as_ref()) {
        Some(net) => crate::results::extract_results(net).len() as i32,
        None => -1,
    }
}

/// Read one result row (index `idx`). Numeric fields are written to the out
/// params; the presence of optional fields is flagged in the `*_set` ints.
/// Returns 0 on success, -1 if idx out of range, nonzero on bad handle.
#[no_mangle]
pub unsafe extern "C" fn venti_results_row(
    handle: i32,
    idx: i32,
    out_flowrate_in: *mut f64,
    out_flow_in_set: *mut i32,
    out_velocity_in: *mut f64,
    out_vel_in_set: *mut i32,
    out_pressure_drop: *mut f64,
) -> i32 {
    let reg = registry().lock().unwrap();
    let net = match reg.get(handle as usize).and_then(|s| s.as_ref()) {
        Some(n) => n,
        None => return -1,
    };
    let rows = crate::results::extract_results(net);
    let row = match rows.get(idx as usize) {
        Some(r) => r,
        None => return -2,
    };
    write_opt(out_flowrate_in, out_flow_in_set, row.flowrate_in);
    write_opt(out_velocity_in, out_vel_in_set, row.velocity_in);
    *out_pressure_drop = row.pressure_drop;
    0
}

/// Copy a string field of a result row into `buf`. Returns 0 on success, -1 on
/// bad handle/index, -3 if `field` is unknown.
/// `field`: 0 = component_id, 1 = name, 2 = component_type.
/// Writes the full string length to `out_len` even when it is truncated to
/// `cap`, so callers can size their buffer on a first pass.
#[no_mangle]
pub unsafe extern "C" fn venti_results_field_string(
    handle: i32,
    idx: i32,
    field: i32,
    buf: *mut u8,
    cap: usize,
    out_len: *mut usize,
) -> i32 {
    let reg = registry().lock().unwrap();
    let net = match reg.get(handle as usize).and_then(|s| s.as_ref()) {
        Some(n) => n,
        None => return -1,
    };
    let rows = crate::results::extract_results(net);
    let row = match rows.get(idx as usize) {
        Some(r) => r,
        None => return -2,
    };
    let s = match field {
        0 => row.component_id.as_str(),
        1 => row.name.as_str(),
        2 => row.component_type.as_str(),
        _ => return -3,
    };
    let bytes = s.as_bytes();
    *out_len = bytes.len();
    let n = bytes.len().min(cap);
    if n > 0 {
        core::ptr::copy_nonoverlapping(bytes.as_ptr(), buf, n);
    }
    0
}

// ---- helpers --------------------------------------------------------------

unsafe fn str_from(ptr: *const u8, len: usize) -> String {
    String::from_utf8_lossy(slice::from_raw_parts(ptr, len)).into_owned()
}

fn component_from_ffi(ctype: i32, name: &str, p: &[f64]) -> Result<ComponentEnum> {
    let comp = match ctype {
        CTYPE_SOURCE => ComponentEnum::Source(crate::components::fitting::Source::new(name)),
        CTYPE_TERMINAL => {
            let area = if p[0] > 0.0 { Some(p[0]) } else { None };
            ComponentEnum::Terminal(crate::components::fitting::Terminal::new(
                name, p[2], area, p[1],
            ))
        }
        CTYPE_RIGID => ComponentEnum::RigidDuct(crate::components::duct::RigidDuct::new(
            name, p[0], p[1], p[2], p[3],
        )?),
        CTYPE_FLEX => ComponentEnum::FlexDuct(crate::components::duct::FlexDuct::new(
            name, p[1], p[2], p[3], p[4],
        )?),
        CTYPE_FITTING => ComponentEnum::TwoPortFitting(
            crate::components::fitting::TwoPortFitting::new(name, p[0], p[1]),
        ),
        CTYPE_TEE => {
            ComponentEnum::Tee(crate::components::fitting::Tee::new(name, p[0], p[1], p[2]))
        }
        _ => return Err(("unknown component type".to_string()).into()),
    };
    Ok(comp)
}

unsafe fn write_opt(out: *mut f64, out_set: *mut i32, value: Option<f64>) {
    match value {
        Some(v) => {
            *out = v;
            *out_set = 1;
        }
        None => {
            *out = 0.0;
            *out_set = 0;
        }
    }
}

/// venti library version as an `(major, minor, patch)` triple.
#[no_mangle]
pub unsafe extern "C" fn venti_version(major: *mut i32, minor: *mut i32, patch: *mut i32) {
    let v = env!("CARGO_PKG_VERSION");
    let mut it = v.split('.');
    *major = it.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    *minor = it.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    *patch = it.next().and_then(|s| s.parse().ok()).unwrap_or(0);
}

/// Allocate `len` bytes in the WASM heap and return a pointer the host can
/// write into (e.g. string / param buffers). Call `venti_free` to release.
#[no_mangle]
pub unsafe extern "C" fn venti_alloc(len: usize) -> *mut u8 {
    let size = len.max(1);
    let mut v: Vec<u8> = Vec::with_capacity(size);
    let p = v.as_mut_ptr();
    core::mem::forget(v); // keep the allocation alive; host calls venti_free
    p
}

/// Free a buffer previously returned by `venti_alloc`.
///
/// # Safety
///
/// `ptr` must come from `venti_alloc` with the same `len`.
#[no_mangle]
pub unsafe extern "C" fn venti_free(ptr: *mut u8, len: usize) {
    if !ptr.is_null() {
        drop(Vec::from_raw_parts(ptr, 0, len.max(1)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a small Source -> RigidDuct -> Fitting -> Terminal chain via the
    /// handle API; returns the handle.
    unsafe fn build_chain() -> i32 {
        let h = venti_network_create(b"Test".as_ptr(), 4);
        assert!(h >= 0, "create failed");

        let area = std::f64::consts::PI * 0.01;
        let source_params = [0f64; 6];
        assert_eq!(
            venti_network_add(
                h,
                b"ahu".as_ptr(),
                3,
                b"AHU".as_ptr(),
                3,
                0,
                source_params.as_ptr()
            ),
            0
        );
        let duct_params = [area, 0.2, 20.0, 0.0001, 0.0, 0.0];
        assert_eq!(
            venti_network_add(
                h,
                b"duct".as_ptr(),
                4,
                b"Main Duct".as_ptr(),
                9,
                2,
                duct_params.as_ptr()
            ),
            0
        );
        let fit_params = [area, 0.5, 0.0, 0.0, 0.0, 0.0];
        assert_eq!(
            venti_network_add(
                h,
                b"fit".as_ptr(),
                3,
                b"Elbow".as_ptr(),
                5,
                4,
                fit_params.as_ptr()
            ),
            0
        );
        let term_params = [area, 1.0, 0.1, 0.0, 0.0, 0.0];
        assert_eq!(
            venti_network_add(
                h,
                b"term".as_ptr(),
                4,
                b"Terminal".as_ptr(),
                8,
                1,
                term_params.as_ptr()
            ),
            0
        );

        for (a, b) in [("ahu", "duct"), ("duct", "fit"), ("fit", "term")] {
            assert_eq!(
                venti_network_connect(h, a.as_ptr(), a.len(), b.as_ptr(), b.len()),
                0,
                "connect {a}->{b}"
            );
        }
        h
    }

    #[test]
    fn handle_api_solve_and_report() {
        let h = unsafe { build_chain() };
        unsafe {
            assert_eq!(venti_network_component_count(h), 4);
            let dp = venti_network_solve(h, 1.204, 1.825e-5);
            // duct(14.135) + fitting(3.050) + terminal(6.100) = 23.285
            assert!((dp - 23.285).abs() < 0.01, "dp = {dp}");

            assert_eq!(venti_results_count(h), 4);

            // Behave correctly on an out-of-range row index.
            assert_eq!(
                venti_results_row(
                    h,
                    99,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut()
                ),
                -2
            );

            venti_network_free(h);
            assert_eq!(venti_network_component_count(h), -1); // freed
        }
    }

    #[test]
    fn handle_api_rejects_duplicate_id() {
        let h = unsafe {
            let h = venti_network_create(b"T".as_ptr(), 1);
            let p = [0f64; 6];
            assert_eq!(
                venti_network_add(h, b"a".as_ptr(), 1, b"x".as_ptr(), 1, 0, p.as_ptr()),
                0
            );
            // duplicate id -> error code
            let rc = venti_network_add(h, b"a".as_ptr(), 1, b"y".as_ptr(), 1, 0, p.as_ptr());
            assert!(rc != 0);
            venti_network_free(h);
            h
        };
        let _ = h;
    }

    #[test]
    fn sound_ffi_closed_form_and_errors() {
        // Regenerated noise with default density (density = 0 selects standard
        // air, so the ρ/ρ₀ term vanishes): Lw = 10 + 60·log10(v) − 20·log10(d).
        let lw = venti_regenerated_noise_round(2.0, 0.5, 0.0);
        let expected = 10.0 + 60.0 * 2.0f64.log10() - 20.0 * 0.5f64.log10();
        assert!((lw - expected).abs() < 1e-9, "lw = {lw}");
        // Bad input -> NAN.
        assert!(venti_regenerated_noise_round(0.0, 0.5, 1.204).is_nan());
        assert!(venti_regenerated_noise_round(4.0, -1.0, 1.204).is_nan());

        // Duct pressure level (room equation) closed form.
        let lp = venti_duct_pressure_level(60.0, 100.0, 0.2);
        let exp = 60.0 + 10.0 * (4.0 * 0.8f64 / (0.2 * 100.0)).log10();
        assert!((lp - exp).abs() < 1e-9, "lp = {lp}");
        assert!(venti_duct_pressure_level(60.0, 0.0, 0.2).is_nan());
        assert!(venti_duct_pressure_level(60.0, 100.0, 1.0).is_nan());

        // nc_ok: office target = 35 dB.
        let mut ok = -1i32;
        unsafe {
            assert_eq!(venti_nc_ok(b"office".as_ptr(), 6, 35.0, &mut ok), 0);
            assert_eq!(ok, 1);
            assert_eq!(venti_nc_ok(b"office".as_ptr(), 6, 36.0, &mut ok), 0);
            assert_eq!(ok, 0);
            // Unknown space type -> nonzero status.
            assert_ne!(venti_nc_ok(b"bogus".as_ptr(), 5, 10.0, &mut ok), 0);
        }
    }

    #[test]
    fn balancing_ffi_closed_form() {
        // ζ = 2·dp/(ρ·v²): dp=19.264, v=4, ρ=1.204 -> dynamic 9.632 -> ζ = 2.
        let z = venti_required_zeta(19.264, 4.0, 1.204);
        assert!((z - 2.0).abs() < 1e-9, "zeta = {z}");

        // Branch short of requirement by 19.264 Pa at dynamic 9.632 -> ζ = 2;
        // already-met branch stays fully open (0).
        let z2 = venti_balancing_zeta(30.0, 10.736, 4.0, 1.204);
        assert!((z2 - 2.0).abs() < 1e-9, "zeta2 = {z2}");
        assert_eq!(venti_balancing_zeta(10.0, 20.0, 4.0, 1.204), 0.0);
        assert_eq!(venti_balancing_zeta(10.0, 10.0, 4.0, 1.204), 0.0);

        // Damper open-percentage round-trips through the butterfly correlation
        // (venti_damper_open_percentage -> venti_damper_butterfly).
        for zeta in [0.1, 0.5, 1.0, 2.5, 5.0, 8.0, 10.0] {
            let open = venti_damper_open_percentage(zeta);
            let back = venti_damper_butterfly(open);
            assert!(
                (back - zeta).abs() < 1e-9,
                "zeta={zeta} open={open} back={back}"
            );
        }
        assert_eq!(venti_damper_open_percentage(0.05), 100.0); // below floor
    }

    #[test]
    fn handle_api_string_field() {
        unsafe {
            let h = build_chain();
            let n = venti_results_count(h);
            assert_eq!(n, 4);

            let mut found: Vec<String> = Vec::new();
            let mut out_len: usize = 0;
            let mut buf = [0u8; 64];
            for i in 0..n {
                assert_eq!(
                    venti_results_field_string(h, i, 0, buf.as_mut_ptr(), 64, &mut out_len),
                    0
                );
                found.push(String::from_utf8_lossy(&buf[..out_len.min(64)]).into_owned());
            }
            // Collection order is arbitrary; the set must be the 4 ids.
            let mut ids = found;
            ids.sort();
            assert_eq!(ids, vec!["ahu", "duct", "fit", "term"]);

            // type field on the component whose id is "duct".
            let mut idx = 0i32;
            for i in 0..n {
                let _ = venti_results_field_string(h, i, 0, buf.as_mut_ptr(), 64, &mut out_len);
                if String::from_utf8_lossy(&buf[..out_len.min(64)]) == "duct" {
                    idx = i;
                    break;
                }
            }
            let _ = venti_results_field_string(h, idx, 2, buf.as_mut_ptr(), 64, &mut out_len);
            assert_eq!(
                String::from_utf8_lossy(&buf[..out_len.min(64)]),
                "RigidDuct"
            );

            // unknown field code
            assert_eq!(
                venti_results_field_string(h, 0, 99, buf.as_mut_ptr(), 64, &mut out_len),
                -3
            );
            venti_network_free(h);
        }
    }
}
