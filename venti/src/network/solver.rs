//! Pure-function solver for a `Network`.
//!
//! Mirrors `python/wenta/network/solver.py` and the Mojo kernels in
//! `wentamojo/network/`. The three steps are:
//!
//! 1. `propagate_flowrates` — sum terminal demands upstream.
//! 2. `compute_pressure_drops` — per-component velocity + ΔP.
//! 3. `critical_path` / `critical_path_pressure_drop` — longest weighted path.
//!
//! The flat-array kernels `critical_path_sum` and `batch_compute` reproduce
//! the Mojo kernels exactly so the port can be diff-tested against the Python
//! reference projections.

use std::collections::HashMap;

use super::network::Network;
use crate::components::base::Component;
use crate::core::fluid::Fluid;
use crate::physics::friction::{friction_factor, relative_roughness, reynolds};

/// Walk the graph and assign a flowrate to every port.
///
/// Terminal demands are propagated upstream so each duct/fitting/source sees
/// the total volumetric flow it must carry.
pub fn propagate_flowrates(network: &mut Network) -> Result<(), String> {
    let topo = network.topo_order()?;
    let preds = network.predecessors();

    // node_id -> flowrate, reset all to zero.
    let mut flows: HashMap<String, f64> =
        network.node_ids().into_iter().map(|n| (n, 0.0)).collect();

    // Seed terminal demands onto their in-port nodes.
    for term in network.terminals().to_vec() {
        let inlet_id = match term.ports().first() {
            Some(p) => p.node_id.clone(),
            None => continue,
        };
        flows.insert(inlet_id, term.flowrate_demand);
    }

    // Walk downstream-first; each node forwards its accumulated flow to every
    // predecessor (the correct upstream node by construction).
    for node in topo.iter().rev() {
        let flow = flows[node];
        if flow != 0.0 {
            if let Some(pred_list) = preds.get(node) {
                for pred in pred_list {
                    if let Some(v) = flows.get_mut(pred) {
                        *v += flow;
                    }
                }
            }
        }
    }

    // Copy graph flowrates back onto the Port objects.
    let cids: Vec<String> = network.components.keys().cloned().collect();
    for cid in &cids {
        let comp = network
            .components
            .get_mut(cid)
            .expect("component id present");
        for pp in comp.as_component_mut().ports_mut().iter_mut() {
            if let Some(v) = flows.get(&pp.node_id) {
                pp.flowrate = Some(*v);
            }
        }
    }
    Ok(())
}

/// Compute every port's pressure drop for the network.
pub fn compute_pressure_drops(network: &mut Network, fluid: &Fluid) -> Result<(), String> {
    let cids: Vec<String> = network.components.keys().cloned().collect();
    for cid in &cids {
        let comp = network
            .components
            .get_mut(cid)
            .expect("component id present");
        comp.as_component_mut().compute(fluid)?;
    }
    Ok(())
}

/// Per-graph-node pressure-drop weights `node_id -> Pa` (ports carry their
/// drop, component nodes carry 0).
fn node_pressure_drops(network: &Network) -> HashMap<String, f64> {
    let mut out: HashMap<String, f64> = HashMap::new();
    for cid in network.components.keys() {
        let comp = &network.components[cid];
        out.insert(cid.clone(), 0.0);
        for p in comp.as_component().ports() {
            out.insert(p.node_id.clone(), p.pressure_drop);
        }
    }
    out
}

/// Return the graph node ids on the critical path.
///
/// The critical path is the longest path (by total node `pressure_drop`) from
/// any `Source` to any `Terminal`. A single-pass DP over the topological
/// order — O(V + E).
pub fn critical_path(network: &Network) -> Result<Vec<String>, String> {
    let topo = network.topo_order()?;
    let preds = network.predecessors();
    let weights = node_pressure_drops(network);

    let mut dist: HashMap<String, f64> = HashMap::new();
    let mut prev: HashMap<String, Option<String>> = HashMap::new();

    for n in &topo {
        let (best_p, best_d) = match preds.get(n) {
            None => (None, 0.0),
            Some(list) if list.is_empty() => (None, 0.0),
            Some(list) if list.len() == 1 => {
                let p = &list[0];
                (Some(p.clone()), dist[p])
            }
            Some(list) => {
                let mut best_p: Option<String> = None;
                let mut best_d = f64::NEG_INFINITY;
                for p in list {
                    let d = dist[p];
                    if d > best_d {
                        best_d = d;
                        best_p = Some(p.clone());
                    }
                }
                (best_p, best_d)
            }
        };
        prev.insert(n.clone(), best_p);
        dist.insert(n.clone(), best_d + weights[n]);
    }

    if dist.is_empty() {
        return Ok(Vec::new());
    }
    let end = dist
        .iter()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        .map(|(k, _)| k.clone())
        .unwrap();
    let mut path = Vec::new();
    let mut cur: Option<String> = Some(end);
    while let Some(c) = cur {
        path.push(c.clone());
        cur = prev.get(&c).cloned().flatten();
    }
    path.reverse();
    Ok(path)
}

/// Total pressure drop along the critical path [Pa].
pub fn critical_path_pressure_drop(network: &Network) -> Result<f64, String> {
    let topo = network.topo_order()?;
    let preds = network.predecessors();
    let weights = node_pressure_drops(network);

    // Flat projection for the kernel-parity path.
    let int_topo: Vec<usize> = (0..topo.len()).collect();
    let int_preds: Vec<Vec<usize>> = topo
        .iter()
        .map(|n| {
            preds
                .get(n)
                .map(|list| {
                    list.iter()
                        .filter_map(|p| topo.iter().position(|t| t == p))
                        .collect()
                })
                .unwrap_or_default()
        })
        .collect();
    let dp: Vec<f64> = topo.iter().map(|n| weights[n]).collect();
    Ok(critical_path_sum(&int_topo, &int_preds, &dp))
}

/// Run the full solver pipeline and return the critical-path pressure drop.
pub fn solve(network: &mut Network, fluid: &Fluid) -> Result<f64, String> {
    propagate_flowrates(network)?;
    compute_pressure_drops(network, fluid)?;
    critical_path_pressure_drop(network)
}

// ---------------------------------------------------------------------------
// Mojo parity kernels (flat int-indexed arrays)
// ---------------------------------------------------------------------------

/// Return the longest weighted path's total (critical-path pressure drop).
///
/// Mirrors `wentamojo.network.solver.critical_path_sum`. Walks `topo` once,
/// accumulating the maximum weighted path ending at each node. O(V + E).
pub fn critical_path_sum(topo: &[usize], preds: &[Vec<usize>], dp: &[f64]) -> f64 {
    let n = dp.len();
    let mut dist = vec![0.0f64; n];
    let mut max_dist = 0.0f64;
    for &node in topo {
        let mut best = 0.0f64;
        for &pd in &preds[node] {
            if dist[pd] > best {
                best = dist[pd];
            }
        }
        let d = best + dp[node];
        dist[node] = d;
        if d > max_dist {
            max_dist = d;
        }
    }
    max_dist
}

// Component type tags (mirror the Mojo constants).
pub const TAG_SOURCE: i64 = 0;
pub const TAG_TERMINAL: i64 = 1;
pub const TAG_RIGID: i64 = 2;
pub const TAG_FLEX: i64 = 3;
pub const TAG_FITTING: i64 = 4;
pub const TAG_TEE: i64 = 5;

/// Full pressure-drop pass over a flat component view.
///
/// Mirrors `wentamojo.network.compute_batch.batch_compute`. Returns
/// `(velocities, dps)` lists of length `p` (per-port).
///
/// * `types` — `[i64; n]` component type tags.
/// * `params` — `[f64; 6n]` row-major per-component params (see the Mojo file).
/// * `port_idx` — `[i64; 3n]` row-major per-component port indices, `-1` unused.
/// * `flows` — `[f64; p]` per-port incoming flow.
pub fn batch_compute(
    types: &[i64],
    params: &[f64],
    port_idx: &[i64],
    flows: &[f64],
    density: f64,
    kinematic_viscosity: f64,
) -> (Vec<f64>, Vec<f64>) {
    let p = flows.len();
    let mut velocities = vec![0.0f64; p];
    let mut dps = vec![0.0f64; p];
    let n = types.len();

    for i in 0..n {
        let tag = types[i];
        let p0 = params[i * 6];
        let p1 = params[i * 6 + 1];
        let p2 = params[i * 6 + 2];
        let p3 = params[i * 6 + 3];
        let p4 = params[i * 6 + 4];
        let ix0 = port_idx[i * 3] as usize;
        let ix1 = port_idx[i * 3 + 1] as usize;
        let ix2 = port_idx[i * 3 + 2] as usize;

        match tag {
            TAG_SOURCE => {} // no drop; out-port v = 0
            TAG_TERMINAL => {
                if p0 > 0.0 {
                    let v = flows[ix0] / p0;
                    velocities[ix0] = v;
                    dps[ix0] = p1 * density * v * v * 0.5;
                }
            }
            TAG_RIGID => {
                let v = flows[ix0] / p0;
                let re = reynolds(v, p1, kinematic_viscosity);
                let eps = relative_roughness(p3, p1);
                let f = friction_factor(re, eps);
                velocities[ix0] = v;
                velocities[ix1] = v;
                dps[ix0] = f * (p2 / p1) * density * v * v * 0.5;
            }
            TAG_FLEX => {
                let v = flows[ix0] / p0;
                let beta = 0.557 * (100.0 - p4) * (-4.93 * p1).exp() + 1.0;
                velocities[ix0] = v;
                velocities[ix1] = v;
                dps[ix0] = p3 * p2 * beta;
            }
            TAG_FITTING => {
                let v = flows[ix0] / p0;
                velocities[ix0] = v;
                velocities[ix1] = v;
                dps[ix1] = p1 * density * v * v * 0.5;
            }
            TAG_TEE => {
                let inv_a = 1.0 / p0;
                let v_s = flows[ix1] * inv_a;
                let v_b = flows[ix2] * inv_a;
                let v_c = flows[ix0] * inv_a;
                velocities[ix0] = v_c;
                velocities[ix1] = v_s;
                velocities[ix2] = v_b;
                dps[ix1] = p1 * density * v_s * v_s * 0.5;
                dps[ix2] = p2 * density * v_b * v_b * 0.5;
            }
            _ => {}
        }
    }
    (velocities, dps)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::duct::RigidDuct;
    use crate::components::fitting::Terminal;
    use crate::components::fitting::{Source, TwoPortFitting};
    use crate::core::geometry::Round;
    use crate::network::network::ComponentEnum as CE;

    fn build_chain(flowrate: f64) -> Network {
        let r = Round::new(0.2).unwrap();
        let mut net = Network::new("chain");
        net.add("ahu", CE::Source(Source::new("AHU"))).unwrap();
        net.add(
            "duct",
            CE::RigidDuct(
                RigidDuct::new("duct", r.area, r.hydraulic_diameter, 10.0, 0.0001).unwrap(),
            ),
        )
        .unwrap();
        net.add(
            "fit",
            CE::TwoPortFitting(TwoPortFitting::new("elbow", r.area, 0.5)),
        )
        .unwrap();
        net.add(
            "term",
            CE::Terminal(Terminal::new("term", flowrate, Some(r.area), 1.0)),
        )
        .unwrap();
        net.connect("ahu", "duct").unwrap();
        net.connect("duct", "fit").unwrap();
        net.connect("fit", "term").unwrap();
        net
    }

    #[test]
    fn critical_path_sum_simple_chain() {
        // topo [a, b, c], dp [0, 5, 3], edges a->b, b->c
        let topo = vec![0usize, 1, 2];
        let preds = vec![vec![], vec![0usize], vec![1usize]];
        let dp = vec![0.0, 5.0, 3.0];
        assert_eq!(critical_path_sum(&topo, &preds, &dp), 8.0);
    }

    #[test]
    fn critical_path_sum_picks_max_branch() {
        // node 2 has two preds with dist 4 and 7 -> take 7.
        let topo = vec![0usize, 1, 2, 3];
        let preds = vec![vec![], vec![0usize], vec![0usize], vec![1usize, 2usize]];
        let dp = vec![0.0, 4.0, 7.0, 3.0];
        assert_eq!(critical_path_sum(&topo, &preds, &dp), 10.0);
    }

    #[test]
    fn batch_compute_rigid_and_fitting() {
        // Two components: a rigid duct (tag 2) and a fitting (tag 4).
        // 4 ports total.
        let types = vec![2i64, 4i64];
        // rigid: area=0.0314, dh=0.2, len=10, eps=0.0001
        let area = std::f64::consts::PI * 0.01;
        let params = vec![
            area, 0.2, 10.0, 0.0001, 0.0, 0.0, area, 0.5, 0.0, 0.0, 0.0, 0.0,
        ];
        // ports: rigid in=0,out=1; fitting in=2,out=3
        let port_idx = vec![0i64, 1, -1, 2, 3, -1];
        let flows = vec![0.1, 0.1, 0.1, 0.1];
        let density = 1.204;
        let kin = 1.825e-5 / 1.204;
        let (v, dp) = batch_compute(&types, &params, &port_idx, &flows, density, kin);
        // all ports at v = 0.1/area
        let v_expected = 0.1 / area;
        assert!((v[0] - v_expected).abs() < 1e-12);
        assert!((v[3] - v_expected).abs() < 1e-12);
        // fitting dp on port 3 = 0.5 * rho * v^2 / 2
        let dp_fit = 0.5 * density * v_expected * v_expected * 0.5;
        assert!((dp[3] - dp_fit).abs() < 1e-9);
    }

    #[test]
    fn end_to_end_chain_solve() {
        let mut net = build_chain(0.1);
        let dp = net.solve(None).unwrap();
        // Critical path = duct dp + fitting dp + terminal dp.
        assert!(dp > 0.0);
        // A physically plausible value for 10 m 200 mm duct + zeta=0.5 fit.
        assert!(dp > 5.0 && dp < 150.0, "dp = {dp}");
    }

    #[test]
    fn propagate_seeds_terminal_demand() {
        let mut net = build_chain(0.1);
        propagate_flowrates(&mut net).unwrap();
        let term = net.terminals()[0];
        assert!((term.ports()[0].flowrate.unwrap() - 0.1).abs() < 1e-12);
    }
}
