//! Extract results from a solved network into structured formats.
//!
//! Mirrors `python/wenta/results.py` (SPEC FR-16): one row per component,
//! aggregating its ports' flow, velocity and pressure drop. These are the
//! building blocks for schedules, CSVs and reports.

use super::network::{ComponentEnum, Network};

/// Results for a single component.
#[derive(Debug, Clone, PartialEq)]
pub struct ComponentResult {
    pub component_id: String,
    pub name: String,
    pub component_type: String,
    /// Volumetric flow entering the component [m³/s].
    pub flowrate_in: Option<f64>,
    /// Volumetric flow leaving the component [m³/s].
    pub flowrate_out: Option<f64>,
    /// Velocity at the inlet port [m/s].
    pub velocity_in: Option<f64>,
    /// Velocity at the outlet port [m/s].
    pub velocity_out: Option<f64>,
    /// Total pressure drop across all ports [Pa].
    pub pressure_drop: f64,
}

impl ComponentResult {
    /// Field names, in order, for a CSV header (matches `results_as_dicts`).
    pub const FIELDS: [&'static str; 8] = [
        "component_id",
        "name",
        "component_type",
        "flowrate_in",
        "flowrate_out",
        "velocity_in",
        "velocity_out",
        "pressure_drop",
    ];

    pub fn to_dict(&self) -> Vec<(String, String)> {
        vec![
            ("component_id".to_string(), self.component_id.clone()),
            ("name".to_string(), self.name.clone()),
            ("component_type".to_string(), self.component_type.clone()),
            ("flowrate_in".to_string(), opt_fmt(self.flowrate_in)),
            ("flowrate_out".to_string(), opt_fmt(self.flowrate_out)),
            ("velocity_in".to_string(), opt_fmt(self.velocity_in)),
            ("velocity_out".to_string(), opt_fmt(self.velocity_out)),
            (
                "pressure_drop".to_string(),
                format!("{}", self.pressure_drop),
            ),
        ]
    }
}

fn opt_fmt(v: Option<f64>) -> String {
    match v {
        Some(x) => format!("{x}"),
        None => String::new(),
    }
}

/// Isolate the first matching value from a list of ports.
fn first_opt(
    ports: &[&crate::components::base::Port],
    f: impl Fn(&crate::components::base::Port) -> Option<f64>,
) -> Option<f64> {
    ports.iter().find_map(|p| f(p))
}

/// Extract results from a solved network into a list, one row per component.
///
/// The `network` should have been solved (`Network::solve` /
/// `solver::solve`) so flowrates and velocities are populated.
pub fn extract_results(network: &Network) -> Vec<ComponentResult> {
    let mut results = Vec::new();
    for (cid, comp) in network.iter_components() {
        let c = comp.as_component();
        let in_ports = c.inlets();
        let out_ports = c.outlets();

        // A port's velocity is considered "set" once it has a flowrate (i.e.
        // it went through solve); matches the Python reference semantics.
        let flowrate_in = first_opt(&in_ports, |p| p.flowrate);
        let flowrate_out = first_opt(&out_ports, |p| p.flowrate);
        let velocity_in = first_opt(&in_ports, |p| p.flowrate.map(|_| p.velocity));
        let velocity_out = first_opt(&out_ports, |p| p.flowrate.map(|_| p.velocity));

        let pressure_drop = c.ports().iter().map(|p| p.pressure_drop).sum();

        results.push(ComponentResult {
            component_id: cid.clone(),
            name: c.name().to_string(),
            component_type: kind_of(comp).to_string(),
            flowrate_in,
            flowrate_out,
            velocity_in,
            velocity_out,
            pressure_drop,
        });
    }
    results
}

fn kind_of(comp: &ComponentEnum) -> &'static str {
    comp.kind()
}

/// Export results as a CSV string with a header row.
pub fn results_as_csv(network: &Network, delimiter: char) -> String {
    let results = extract_results(network);
    if results.is_empty() {
        return String::new();
    }
    let mut lines = vec![ComponentResult::FIELDS.join(&delimiter.to_string())];
    for r in &results {
        let values: Vec<String> = r.to_dict().into_iter().map(|(_, v)| v).collect();
        lines.push(values.join(&delimiter.to_string()));
    }
    lines.join("\n")
}

/// JSON rows for a resolved report (gated on the `cli` feature, which brings
/// in `serde_json`).
#[cfg(feature = "cli")]
pub fn results_as_json_rows(network: &Network) -> Vec<serde_json::Value> {
    extract_results(network)
        .into_iter()
        .map(|r| {
            serde_json::json! ({
                "component_id": r.component_id,
                "name": r.name,
                "component_type": r.component_type,
                "flowrate_in": r.flowrate_in,
                "flowrate_out": r.flowrate_out,
                "velocity_in": r.velocity_in,
                "velocity_out": r.velocity_out,
                "pressure_drop": r.pressure_drop,
            })
        })
        .collect()
}

/// Compact JSON string of the report rows (used by the CLI and parity tests).
#[cfg(feature = "cli")]
pub fn report_json_string(network: &Network) -> String {
    serde_json::to_string(&results_as_json_rows(network)).unwrap_or_else(|e| {
        // serde_json::to_string on these values cannot fail in practice.
        format!("{{\"error\": {e:?}}}")
    })
}

/// Format results as a human-readable table.
pub fn results_summary(network: &Network) -> String {
    let results = extract_results(network);
    if results.is_empty() {
        return "(no components)".to_string();
    }
    // Column widths mirroring the Python reference.
    let widths = [
        ("ID", 12usize),
        ("Name", 20usize),
        ("Type", 18usize),
        ("Q_in [m³/s]", 13usize),
        ("V_in [m/s]", 12usize),
        ("ΔP [Pa]", 11usize),
    ];
    let mut lines = Vec::new();
    let header = widths
        .iter()
        .map(|(k, w)| format!("{k:<w$}"))
        .collect::<Vec<_>>()
        .join(" | ");
    let sep_len = widths.iter().map(|(_, w)| w).sum::<usize>() + widths.len() * 3 - 3;
    let sep = "-".repeat(sep_len);

    lines.push(sep.clone());
    lines.push(header);
    lines.push(sep.clone());
    for r in &results {
        let q_in = match r.flowrate_in {
            Some(x) => format!("{x:.3}"),
            None => "—".to_string(),
        };
        let v_in = match r.velocity_in {
            Some(x) => format!("{x:.2}"),
            None => "—".to_string(),
        };
        let dp = format!("{:.2}", r.pressure_drop);
        lines.push(format!(
            "{:<12} | {:<20} | {:<18} | {:>12} | {:>11} | {:>10}",
            r.component_id, r.name, r.component_type, q_in, v_in, dp
        ));
    }
    lines.push(sep);
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::duct::RigidDuct;
    use crate::components::fitting::{Source, Terminal, TwoPortFitting};
    use crate::core::geometry::Round;
    use crate::network::network::ComponentEnum;
    use crate::network::Network;

    fn solved_chain() -> Network {
        let r = Round::new(0.2).unwrap();
        let mut net = Network::new("chain");
        net.add("ahu", ComponentEnum::Source(Source::new("AHU")))
            .unwrap();
        net.add(
            "duct",
            ComponentEnum::RigidDuct(
                RigidDuct::new("duct", r.area, r.hydraulic_diameter, 10.0, 0.0001).unwrap(),
            ),
        )
        .unwrap();
        net.add(
            "fit",
            ComponentEnum::TwoPortFitting(TwoPortFitting::new("elbow", r.area, 0.5)),
        )
        .unwrap();
        net.add(
            "term",
            ComponentEnum::Terminal(Terminal::new("term", 0.1, Some(r.area), 1.0)),
        )
        .unwrap();
        net.connect("ahu", "duct").unwrap();
        net.connect("duct", "fit").unwrap();
        net.connect("fit", "term").unwrap();
        net.solve(None).unwrap();
        net
    }

    #[test]
    fn extract_one_row_per_component_with_types() {
        let net = solved_chain();
        let res = extract_results(&net);
        assert_eq!(res.len(), 4);
        let by_id: std::collections::HashMap<&str, &ComponentResult> =
            res.iter().map(|r| (r.component_id.as_str(), r)).collect();
        assert_eq!(by_id["duct"].component_type, "RigidDuct");
        assert_eq!(by_id["fit"].component_type, "TwoPortFitting");
        assert_eq!(by_id["term"].component_type, "Terminal");
        assert_eq!(by_id["ahu"].component_type, "Source");
    }

    #[test]
    fn terminal_flow_propagates_to_duct() {
        let net = solved_chain();
        let res = extract_results(&net);
        let duct = res.iter().find(|r| r.component_id == "duct").unwrap();
        assert!((duct.flowrate_in.unwrap() - 0.1).abs() < 1e-12);
        assert!(duct.velocity_in.unwrap() > 0.0);
        assert!(duct.pressure_drop > 0.0);
    }

    #[test]
    fn csv_has_header_and_rows() {
        let net = solved_chain();
        let csv = results_as_csv(&net, ',');
        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(lines.len(), 5); // header + 4 components
        assert!(lines[0].starts_with("component_id,"));
    }

    #[test]
    fn summary_is_non_empty_table() {
        let net = solved_chain();
        let s = results_summary(&net);
        assert!(s.contains("ID"));
        assert!(s.contains("RigidDuct"));
    }

    #[test]
    fn unsolved_network_yields_no_flow() {
        let r = Round::new(0.2).unwrap();
        let mut net = Network::new("chain");
        net.add(
            "term",
            ComponentEnum::Terminal(Terminal::new("term", 0.1, Some(r.area), 1.0)),
        )
        .unwrap();
        let res = extract_results(&net);
        // Terminal is pre-seeded with flow; no duct, so only one row.
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].flowrate_in, Some(0.1));
    }
}
