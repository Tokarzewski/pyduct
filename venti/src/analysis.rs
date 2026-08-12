//! Per-branch analysis report: combine the pressure-drop network solution
//! with the sound (regenerated noise) and balancing (damper ζ) modules into a
//! single dockable-panel report per branch.
//!
//! This is the "core" part of pyduct issue #35 — *Surfacing sound + balancing
//! in the dockable panel*. For every duct branch in a solved network we
//! report its flow, velocity, pressure drop, regenerated noise level and the
//! damper loss coefficient (ζ) needed to balance it against the critical
//! path.
//!
//! The module is **dependency-free** — pure `f64` math over the existing
//! `sound` / `balancing` / `results` infrastructure.

use crate::Result;

/// Per-branch analysis row shown in the dockable panel.
///
/// One entry per duct component (`RigidDuct` or `FlexDuct`) in the network.
#[derive(Debug, Clone, PartialEq)]
pub struct BranchInfo {
    /// The component id of the duct in the network.
    pub component_id: String,
    /// Component kind: `"RigidDuct"` or `"FlexDuct"`.
    pub kind: String,
    /// Volumetric flow carried by the branch [m³/s].
    pub flow_m3s: f64,
    /// Mean duct air velocity [m/s].
    pub velocity_ms: f64,
    /// Total pressure drop across the branch [Pa].
    pub pressure_drop_pa: f64,
    /// Airflow-regenerated noise level [dB re 1e-12 W], `None` when it cannot
    /// be evaluated (e.g. zero velocity).
    pub regenerated_noise_db: Option<f64>,
    /// Damper loss coefficient (ζ) required to balance the branch against the
    /// critical path, `None` when not meaningful (e.g. zero velocity).
    pub balancing_zeta: Option<f64>,
}

/// The analysis report: the critical-path pressure drop plus one branch per
/// duct.
#[derive(Debug, Clone, PartialEq)]
pub struct AnalysisSummary {
    /// Total pressure drop along the critical path [Pa].
    pub critical_dp_pa: f64,
    /// One [`BranchInfo`] per duct component.
    pub branches: Vec<BranchInfo>,
    /// The number of duct branches (equal to `branches.len()`).
    pub n_branches: usize,
}

/// Analyze a network and produce a per-branch sound + balancing report.
///
/// The network is solved (cloned internally so the caller's graph is not
/// mutated) to get the critical-path pressure drop; then every
/// `RigidDuct`/`FlexDuct` branch is resolved into a [`BranchInfo`]:
///
/// * `flow_m3s` — the inlet-port flowrate after the solve.
/// * `velocity_ms` — `flow / cross-section area`.
/// * `pressure_drop_pa` — the total drop across the component's ports.
/// * `regenerated_noise_db` — [`crate::sound::regenerated_noise_round`] using
///   the duct's hydraulic diameter (`RigidDuct` / rectangular) or diameter
///   (`FlexDuct` / round), wrapped in `Option` so unphysical inputs become
///   `None` rather than an error.
/// * `balancing_zeta` — [`crate::balancing::balancing_zeta`] against the
///   critical-path drop, using the component's *own* pressure drop as the
///   available-pressure proxy (see below).
///
/// # Balancing "available" pressure proxy
///
/// A branch's *available* pressure is not directly solvable from the scalar
/// critical-path DP alone; for this dockable-panel report the component's own
/// pressure drop is used as a proxy for the pressure available at that
/// branch. **This is a deliberate approximation.** Branches whose own drop is
/// close to the critical drop see `ζ ≈ 0` (damper fully open), while branches
/// that drop much less than the critical path are treated as over-supplied
/// and get a positive ζ to eat the surplus. See
/// [`crate::balancing::balancing_zeta`].
///
/// # Errors
///
/// Returns the error from solving the network (e.g. a cyclic graph) or from
/// computing the critical path.
///
/// # Examples
///
/// ```
/// use venti::{
///     analyze, Network, ComponentEnum, Source, RigidDuct, Terminal, Round,
///     STANDARD_AIR,
/// };
///
/// let r = Round::new(0.2).unwrap();
/// let mut net = Network::new("panel");
/// net.add("ahu",  ComponentEnum::Source(Source::new("AHU"))).unwrap();
/// net.add("duct", ComponentEnum::RigidDuct(RigidDuct::new(
///     "duct", r.area, r.hydraulic_diameter, 10.0, 0.0001,
/// ).unwrap())).unwrap();
/// net.add("term", ComponentEnum::Terminal(Terminal::new(
///     "term", 0.1, Some(r.area), 1.0,
/// ))).unwrap();
/// net.connect("ahu", "duct").unwrap();
/// net.connect("duct", "term").unwrap();
///
/// let summary = analyze(&net, &STANDARD_AIR).unwrap();
/// assert!(summary.critical_dp_pa > 0.0);
/// assert_eq!(summary.n_branches, 1);
/// let branch = &summary.branches[0];
/// assert_eq!(branch.kind, "RigidDuct");
/// assert!(branch.velocity_ms > 0.0);
/// assert!(branch.regenerated_noise_db.is_some());
/// assert!(branch.balancing_zeta.is_some());
/// ```
pub fn analyze(
    network: &crate::network::Network,
    fluid: &crate::core::fluid::Fluid,
) -> Result<AnalysisSummary> {
    // Solve a clone so the caller's network is left untouched.
    let mut solved = network.clone();
    let critical_dp_pa = crate::network::solver::solve(&mut solved, fluid)?;

    let mut branches = Vec::new();

    for (cid, comp) in solved.iter_components() {
        let c = comp.as_component();
        let kind = comp.kind();

        // Only ducts are reported as branches.
        let (area, diameter) = match comp {
            crate::network::ComponentEnum::RigidDuct(d) => (d.area, d.hydraulic_diameter),
            crate::network::ComponentEnum::FlexDuct(d) => {
                let r = d.diameter * 0.5;
                (std::f64::consts::PI * r * r, d.diameter)
            }
            _ => continue,
        };

        // Inlet flow from the component's inlet port.
        let flow_m3s = c.inlets().first().and_then(|p| p.flowrate).unwrap_or(0.0);

        let velocity_ms = flow_m3s / area;

        // Total pressure drop across all ports.
        let pressure_drop_pa = c.ports().iter().map(|p| p.pressure_drop).sum();

        // Regenerated noise: velocity>0 and diameter>0 required, else None.
        let regenerated_noise_db =
            crate::sound::regenerated_noise_round(velocity_ms, diameter, Some(fluid.density)).ok();

        // Balancing ζ: use the component's own drop as the available-pressure
        // proxy (documented above). None when the velocity is not meaningful.
        let balancing_zeta = if velocity_ms > 0.0 {
            Some(crate::balancing::balancing_zeta(
                critical_dp_pa,
                pressure_drop_pa,
                velocity_ms,
                fluid.density,
            ))
        } else {
            None
        };

        branches.push(BranchInfo {
            component_id: cid.clone(),
            kind: kind.to_string(),
            flow_m3s,
            velocity_ms,
            pressure_drop_pa,
            regenerated_noise_db,
            balancing_zeta,
        });
    }

    let n_branches = branches.len();
    Ok(AnalysisSummary {
        critical_dp_pa,
        branches,
        n_branches,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::duct::{FlexDuct, RigidDuct};
    use crate::components::fitting::{Source, Terminal};
    use crate::core::fluid::STANDARD_AIR;
    use crate::core::geometry::Round;
    use crate::network::network::ComponentEnum;
    use crate::network::Network;

    /// Source -> RigidDuct -> Terminal chain.
    fn build_chain(flowrate: f64, length: f64) -> Network {
        let r = Round::new(0.2).unwrap();
        let mut net = Network::new("chain");
        net.add("ahu", ComponentEnum::Source(Source::new("AHU")))
            .unwrap();
        net.add(
            "duct",
            ComponentEnum::RigidDuct(
                RigidDuct::new("duct", r.area, r.hydraulic_diameter, length, 0.0001).unwrap(),
            ),
        )
        .unwrap();
        net.add(
            "term",
            ComponentEnum::Terminal(Terminal::new("term", flowrate, Some(r.area), 1.0)),
        )
        .unwrap();
        net.connect("ahu", "duct").unwrap();
        net.connect("duct", "term").unwrap();
        net
    }

    /// Source -> FlexDuct -> Terminal chain.
    fn build_flex_chain(flowrate: f64, length: f64) -> Network {
        let mut net = Network::new("flex-chain");
        net.add("ahu", ComponentEnum::Source(Source::new("AHU")))
            .unwrap();
        net.add(
            "duct",
            ComponentEnum::FlexDuct(FlexDuct::new("fduct", 0.2, length, 2.0, 100.0).unwrap()),
        )
        .unwrap();
        net.add(
            "term",
            ComponentEnum::Terminal(Terminal::new("term", flowrate, None, 1.0)),
        )
        .unwrap();
        net.connect("ahu", "duct").unwrap();
        net.connect("duct", "term").unwrap();
        net
    }

    #[test]
    fn chain_analysis_has_duct_branch_with_velocity() {
        let net = build_chain(0.1, 10.0);
        let s = analyze(&net, &STANDARD_AIR).unwrap();
        assert!(s.n_branches >= 1);
        let branch = s.branches.iter().find(|b| b.kind == "RigidDuct").unwrap();
        assert!(
            branch.velocity_ms > 0.0,
            "velocity = {}",
            branch.velocity_ms
        );
        assert!(branch.flow_m3s > 0.0);
        assert!(branch.pressure_drop_pa > 0.0);
    }

    #[test]
    fn regenerated_noise_in_plausible_range() {
        let net = build_chain(0.1, 10.0);
        let s = analyze(&net, &STANDARD_AIR).unwrap();
        let db = s.branches[0].regenerated_noise_db.expect("noise present");
        assert!(db > 0.0 && db < 90.0, "regenerated noise = {db} dB");
    }

    #[test]
    fn balancing_zeta_non_negative() {
        let net = build_chain(0.1, 10.0);
        let s = analyze(&net, &STANDARD_AIR).unwrap();
        for b in &s.branches {
            if let Some(z) = b.balancing_zeta {
                assert!(z >= 0.0, "zeta = {z}");
            }
        }
    }

    #[test]
    fn n_branches_counts_ducts_only() {
        // Chain has 1 duct (plus source + terminal, which are not branches).
        let net = build_chain(0.1, 10.0);
        let s = analyze(&net, &STANDARD_AIR).unwrap();
        assert_eq!(s.n_branches, 1);
        assert_eq!(s.branches.len(), s.n_branches);
        assert_eq!(s.branches[0].kind, "RigidDuct");
    }

    #[test]
    fn critical_dp_is_positive() {
        let net = build_chain(0.1, 10.0);
        let s = analyze(&net, &STANDARD_AIR).unwrap();
        assert!(s.critical_dp_pa > 0.0, "critical dp = {}", s.critical_dp_pa);
    }

    #[test]
    fn flex_duct_branch_reports_round_diameter() {
        let net = build_flex_chain(0.1, 10.0);
        let s = analyze(&net, &STANDARD_AIR).unwrap();
        assert_eq!(s.n_branches, 1);
        let b = &s.branches[0];
        assert_eq!(b.kind, "FlexDuct");
        assert!(b.velocity_ms > 0.0);
        assert!(b.regenerated_noise_db.is_some());
        assert!(b.balancing_zeta.is_some());
    }

    #[test]
    fn solve_error_propagates_on_cyclic_network() {
        // A cyclic graph must fail to analyze rather than panic or succeed.
        let r = Round::new(0.2).unwrap();
        let mut net = Network::new("cyclic");
        net.add("s", ComponentEnum::Source(Source::new("AHU")))
            .unwrap();
        net.add(
            "d0",
            ComponentEnum::RigidDuct(
                RigidDuct::new("d0", r.area, r.hydraulic_diameter, 5.0, 0.0001).unwrap(),
            ),
        )
        .unwrap();
        net.add(
            "d1",
            ComponentEnum::RigidDuct(
                RigidDuct::new("d1", r.area, r.hydraulic_diameter, 5.0, 0.0001).unwrap(),
            ),
        )
        .unwrap();
        net.add(
            "t",
            ComponentEnum::Terminal(Terminal::new("t", 0.1, Some(r.area), 1.0)),
        )
        .unwrap();
        net.connect("s", "d0").unwrap();
        net.connect("d0", "d1").unwrap();
        net.connect("d1", "t").unwrap();
        net.connect("d1", "d0").unwrap(); // cycle
        let err = analyze(&net, &STANDARD_AIR).unwrap_err();
        assert!(err.to_string().contains("cycle"));
    }
}
