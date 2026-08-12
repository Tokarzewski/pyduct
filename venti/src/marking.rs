//! Duct marking: branch numbering and per-duct ID marks.
//!
//! [`assign_branch_marks`] walks a duct network downstream from its source(s)
//! using a breadth-first search and assigns a deterministic branch number to
//! every [`RigidDuct`]. Branch numbers start at 1 and increment for each duct
//! encountered, so a simple chain yields `1, 2, 3, …` and the two legs split
//! at a tee receive distinct branch numbers. Each mark carries the component
//! id, kind, the duct's rounded hydraulic diameter in mm and, when the network
//! has been solved, the flow in m³/s.

use crate::Result;
use std::collections::{HashSet, VecDeque};

use crate::network::{port_node_id, ComponentEnum, Network};

/// A single marking row for a ducted component.
#[derive(Debug, Clone, PartialEq)]
pub struct Mark {
    /// Branch number assigned by downstream BFS from the source (`1, 2, …`).
    pub branch_no: u32,
    /// Component id within the network.
    pub component_id: String,
    /// Component kind, e.g. `"RigidDuct"` (`ComponentEnum::kind()`).
    pub kind: String,
    /// Hydraulic diameter in millimetres, rounded to the nearest mm (`Some`
    /// for ducts).
    pub size_mm: Option<f64>,
    /// Volumetric flow in m³/s from the solved results, if available.
    pub flow_m3s: Option<f64>,
}

/// Assign a deterministic branch number to every [`RigidDuct`] in `network`.
///
/// The traversal is a breadth-first search seeded from every `Source` and
/// follows connection edges downstream. Each rigid duct is assigned the next
/// branch number in discovery order. Neighbours discovered at the same step
/// are sorted and de-duplicated so the numbering is stable regardless of
/// hash-map iteration order. Returns an empty vector for an empty network.
///
/// `size_mm` is the duct's hydraulic diameter expressed in millimetres and
/// rounded to the nearest integer millimetre. We derive the size directly from
/// [`RigidDuct::hydraulic_diameter`]: for a circular duct the hydraulic
/// diameter equals the physical diameter (`d = sqrt(4A/π)`), so using it
/// directly reproduces the round-duct diameter while also remaining sensible
/// for any cross-section via the equivalent-diameter definition.
/// `flow_m3s` is read from the duct's inlet port, so it is `Some` only after
/// the network has been solved / flowrates propagated.
///
/// # Examples
///
/// ```
/// use venti::{assign_branch_marks, ComponentEnum, Network, RigidDuct, Round, Source, Terminal};
///
/// let r = Round::new(0.2).unwrap();
/// let mut net = Network::new("marking");
/// net.add("ahu", ComponentEnum::Source(Source::new("AHU"))).unwrap();
/// net.add("d1", ComponentEnum::RigidDuct(
///     RigidDuct::new("d1", r.area, r.hydraulic_diameter, 10.0, 0.0001).unwrap(),
/// )).unwrap();
/// net.add("d2", ComponentEnum::RigidDuct(
///     RigidDuct::new("d2", r.area, r.hydraulic_diameter, 10.0, 0.0001).unwrap(),
/// )).unwrap();
/// net.add("term", ComponentEnum::Terminal(
///     Terminal::new("term", 0.1, Some(r.area), 1.0),
/// )).unwrap();
/// net.connect("ahu", "d1").unwrap();
/// net.connect("d1", "d2").unwrap();
/// net.connect("d2", "term").unwrap();
///
/// let marks = assign_branch_marks(&net).unwrap();
/// assert_eq!(marks.len(), 2);
/// assert_eq!(marks[0].branch_no, 1);
/// assert_eq!(marks[1].branch_no, 2);
/// assert_eq!(marks[0].size_mm, Some(200.0));
/// ```
pub fn assign_branch_marks(network: &Network) -> Result<Vec<Mark>> {
    if network.is_empty() {
        return Ok(Vec::new());
    }

    let succ = network.successors();

    // Deterministic BFS seed: source component ids, sorted.
    let mut seeds: Vec<String> = network
        .iter_components()
        .filter(|(_, c)| c.is_source())
        .map(|(id, _)| id.clone())
        .collect();
    seeds.sort();

    let mut visited: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<String> = seeds.into_iter().collect();
    let mut marks: Vec<Mark> = Vec::new();
    let mut next_branch: u32 = 1;

    while let Some(cid) = queue.pop_front() {
        if !visited.insert(cid.clone()) {
            continue;
        }
        let component = match network.get(&cid) {
            Some(c) => c,
            None => continue,
        };

        if let ComponentEnum::RigidDuct(duct) = component {
            marks.push(Mark {
                branch_no: next_branch,
                component_id: cid.clone(),
                kind: component.kind().to_string(),
                size_mm: Some((duct.hydraulic_diameter * 1000.0).round()),
                flow_m3s: inlet_flow(component),
            });
            next_branch += 1;
        }

        // Discover downstream components through this component's outlet ports.
        let mut downstream: Vec<String> = Vec::new();
        for port in component.as_component().outlets() {
            let pid = port_node_id(&cid, &port.name);
            if let Some(neighbours) = succ.get(&pid) {
                for node in neighbours {
                    if let Some((ncid, _)) = node.split_once(':') {
                        if ncid != cid.as_str() && !visited.contains(ncid) {
                            downstream.push(ncid.to_string());
                        }
                    }
                }
            }
        }
        downstream.sort();
        downstream.dedup();
        queue.extend(downstream);
    }

    Ok(marks)
}

/// Flowrate on the component's first inlet port, if the network was solved.
fn inlet_flow(component: &ComponentEnum) -> Option<f64> {
    component
        .as_component()
        .inlets()
        .first()
        .and_then(|p| p.flowrate)
}

/// Format an optional float (empty string for `None`).
fn fmt_opt(v: Option<f64>) -> String {
    match v {
        Some(x) => format!("{}", x),
        None => String::new(),
    }
}

/// Render marks as a comma-separated values string with a header row:
/// `branch_no,component_id,kind,size_mm,flow_m3s`.
///
/// # Examples
///
/// ```
/// use venti::{marks_as_csv, Mark};
///
/// let marks = vec![Mark {
///     branch_no: 1,
///     component_id: "d1".into(),
///     kind: "RigidDuct".into(),
///     size_mm: Some(200.0),
///     flow_m3s: Some(0.1),
/// }];
/// let csv = marks_as_csv(&marks);
/// assert!(csv.starts_with("branch_no,component_id,kind,size_mm,flow_m3s"));
/// assert!(csv.contains("1,d1,RigidDuct,200,0.1"));
/// ```
pub fn marks_as_csv(marks: &[Mark]) -> String {
    let mut lines = vec!["branch_no,component_id,kind,size_mm,flow_m3s".to_string()];
    for m in marks {
        lines.push(format!(
            "{},{},{},{},{}",
            m.branch_no,
            m.component_id,
            m.kind,
            fmt_opt(m.size_mm),
            fmt_opt(m.flow_m3s),
        ));
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::duct::RigidDuct;
    use crate::components::fitting::{Source, Tee, Terminal};
    use crate::core::geometry::Round;
    use crate::network::ComponentEnum;
    use crate::network::Network;

    fn rigid(name: &str, area: f64, hd: f64) -> RigidDuct {
        RigidDuct::new(name, area, hd, 10.0, 0.0001).unwrap()
    }

    fn round_net() -> Round {
        Round::new(0.2).unwrap()
    }

    #[test]
    fn simple_chain_gives_consecutive_branch_numbers() {
        let r = round_net();
        let mut net = Network::new("chain");
        net.add("ahu", ComponentEnum::Source(Source::new("AHU")))
            .unwrap();
        net.add(
            "d1",
            ComponentEnum::RigidDuct(rigid("d1", r.area, r.hydraulic_diameter)),
        )
        .unwrap();
        net.add(
            "d2",
            ComponentEnum::RigidDuct(rigid("d2", r.area, r.hydraulic_diameter)),
        )
        .unwrap();
        net.add(
            "term",
            ComponentEnum::Terminal(Terminal::new("term", 0.1, Some(r.area), 1.0)),
        )
        .unwrap();
        net.connect("ahu", "d1").unwrap();
        net.connect("d1", "d2").unwrap();
        net.connect("d2", "term").unwrap();

        let marks = assign_branch_marks(&net).unwrap();
        let nums: Vec<u32> = marks.iter().map(|m| m.branch_no).collect();
        assert_eq!(nums, vec![1, 2]);
        assert_eq!(marks[0].component_id, "d1");
        assert_eq!(marks[1].component_id, "d2");
    }

    #[test]
    fn tee_split_gives_distinct_downstream_branch_numbers() {
        let r = round_net();
        let mut net = Network::new("tee");
        net.add("ahu", ComponentEnum::Source(Source::new("AHU")))
            .unwrap();
        net.add(
            "d1",
            ComponentEnum::RigidDuct(rigid("d1", r.area, r.hydraulic_diameter)),
        )
        .unwrap();
        net.add("tee", ComponentEnum::Tee(Tee::new("tee", r.area, 0.3, 0.5)))
            .unwrap();
        net.add(
            "d2",
            ComponentEnum::RigidDuct(rigid("d2", r.area, r.hydraulic_diameter)),
        )
        .unwrap();
        net.add(
            "d3",
            ComponentEnum::RigidDuct(rigid("d3", r.area, r.hydraulic_diameter)),
        )
        .unwrap();
        net.add(
            "t2",
            ComponentEnum::Terminal(Terminal::new("t2", 0.05, Some(r.area), 1.0)),
        )
        .unwrap();
        net.add(
            "t3",
            ComponentEnum::Terminal(Terminal::new("t3", 0.05, Some(r.area), 1.0)),
        )
        .unwrap();
        net.connect("ahu", "d1").unwrap();
        net.connect("d1", "tee").unwrap();
        net.connect("tee.straight", "d2").unwrap();
        net.connect("tee.branch", "d3").unwrap();
        net.connect("d2", "t2").unwrap();
        net.connect("d3", "t3").unwrap();

        let marks = assign_branch_marks(&net).unwrap();
        assert_eq!(marks.len(), 3);
        let nums: Vec<u32> = marks.iter().map(|m| m.branch_no).collect();
        assert_eq!(nums, vec![1, 2, 3]);
        let d2 = marks.iter().find(|m| m.component_id == "d2").unwrap();
        let d3 = marks.iter().find(|m| m.component_id == "d3").unwrap();
        assert_ne!(d2.branch_no, d3.branch_no);
    }

    #[test]
    fn size_mm_is_some_for_ducts() {
        let r = round_net();
        let mut net = Network::new("size");
        net.add("ahu", ComponentEnum::Source(Source::new("AHU")))
            .unwrap();
        net.add(
            "d1",
            ComponentEnum::RigidDuct(rigid("d1", r.area, r.hydraulic_diameter)),
        )
        .unwrap();
        net.add(
            "term",
            ComponentEnum::Terminal(Terminal::new("term", 0.1, Some(r.area), 1.0)),
        )
        .unwrap();
        net.connect("ahu", "d1").unwrap();
        net.connect("d1", "term").unwrap();

        let marks = assign_branch_marks(&net).unwrap();
        assert_eq!(marks.len(), 1);
        assert!(marks[0].size_mm.is_some());
        assert_eq!(marks[0].size_mm.unwrap(), 200.0);
        assert_eq!(marks[0].kind, "RigidDuct");
    }

    #[test]
    fn flow_is_some_after_solve_else_none() {
        let r = round_net();
        let mut net = Network::new("flow");
        net.add("ahu", ComponentEnum::Source(Source::new("AHU")))
            .unwrap();
        net.add(
            "d1",
            ComponentEnum::RigidDuct(rigid("d1", r.area, r.hydraulic_diameter)),
        )
        .unwrap();
        net.add(
            "term",
            ComponentEnum::Terminal(Terminal::new("term", 0.1, Some(r.area), 1.0)),
        )
        .unwrap();
        net.connect("ahu", "d1").unwrap();
        net.connect("d1", "term").unwrap();

        // Unsolved network: flow unavailable.
        let before = assign_branch_marks(&net).unwrap();
        assert_eq!(before[0].flow_m3s, None);

        net.solve(None).unwrap();
        let after = assign_branch_marks(&net).unwrap();
        assert_eq!(after[0].flow_m3s, Some(0.1));
    }

    #[test]
    fn marks_as_csv_has_header_and_content() {
        let marks = vec![
            Mark {
                branch_no: 1,
                component_id: "d1".into(),
                kind: "RigidDuct".into(),
                size_mm: Some(200.0),
                flow_m3s: Some(0.1),
            },
            Mark {
                branch_no: 2,
                component_id: "d2".into(),
                kind: "RigidDuct".into(),
                size_mm: Some(160.0),
                flow_m3s: None,
            },
        ];
        let csv = marks_as_csv(&marks);
        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "branch_no,component_id,kind,size_mm,flow_m3s");
        assert_eq!(lines[1], "1,d1,RigidDuct,200,0.1");
        assert_eq!(lines[2], "2,d2,RigidDuct,160,"); // blank flow -> trailing empty
    }

    #[test]
    fn empty_network_returns_empty_marks() {
        let net = Network::new("empty");
        let marks = assign_branch_marks(&net).unwrap();
        assert!(marks.is_empty());
        // CSV of no marks still emits the header row.
        assert_eq!(
            marks_as_csv(&marks),
            "branch_no,component_id,kind,size_mm,flow_m3s"
        );
    }
}
