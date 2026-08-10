//! Ductwork network: a directed graph of components and ports.
//!
//! This mirrors `python/wenta/network/network.py` (which builds a NetworkX
//! `DiGraph`) but with a self-contained adjacency representation, so `venti`
//! has no graph-library dependency. The graph convention is identical to the
//! reference:
//!
//! * every component is a node identified by its `component_id`;
//! * every port is a node identified by `"{component_id}:{port_name}"`;
//! * internal edges follow physical airflow (in ports → component → out ports);
//! * connection edges go from one component's `out` port to another
//!   component's `in` port.

use std::collections::{HashMap, VecDeque};

use super::solver;
use crate::components::base::{Component, Port};
use crate::components::duct::{FlexDuct, RigidDuct};
use crate::components::fitting::{Source, Tee, Terminal, TwoPortFitting};
use crate::core::fluid::{Fluid, STANDARD_AIR};

/// A component stored in a network, boxed behind an enum so the graph can hold
/// heterogeneous component types in one map.
#[derive(Debug, Clone)]
pub enum ComponentEnum {
    Source(Source),
    Terminal(Terminal),
    RigidDuct(RigidDuct),
    FlexDuct(FlexDuct),
    TwoPortFitting(TwoPortFitting),
    Tee(Tee),
}

impl ComponentEnum {
    pub fn as_component(&self) -> &dyn Component {
        match self {
            ComponentEnum::Source(c) => c,
            ComponentEnum::Terminal(c) => c,
            ComponentEnum::RigidDuct(c) => c,
            ComponentEnum::FlexDuct(c) => c,
            ComponentEnum::TwoPortFitting(c) => c,
            ComponentEnum::Tee(c) => c,
        }
    }

    pub fn as_component_mut(&mut self) -> &mut dyn Component {
        match self {
            ComponentEnum::Source(c) => c,
            ComponentEnum::Terminal(c) => c,
            ComponentEnum::RigidDuct(c) => c,
            ComponentEnum::FlexDuct(c) => c,
            ComponentEnum::TwoPortFitting(c) => c,
            ComponentEnum::Tee(c) => c,
        }
    }

    pub fn is_source(&self) -> bool {
        matches!(self, ComponentEnum::Source(_))
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, ComponentEnum::Terminal(_))
    }

    /// Rust type name, matching the Python class name of the same component.
    pub fn kind(&self) -> &'static str {
        match self {
            ComponentEnum::Source(_) => "Source",
            ComponentEnum::Terminal(_) => "Terminal",
            ComponentEnum::RigidDuct(_) => "RigidDuct",
            ComponentEnum::FlexDuct(_) => "FlexDuct",
            ComponentEnum::TwoPortFitting(_) => "TwoPortFitting",
            ComponentEnum::Tee(_) => "Tee",
        }
    }
}

/// Stable graph-node id for a port.
pub fn port_node_id(component_id: &str, port_name: &str) -> String {
    format!("{component_id}:{port_name}")
}

/// A directed graph of ductwork components.
#[derive(Debug, Clone, Default)]
pub struct Network {
    pub name: String,
    pub(crate) components: HashMap<String, ComponentEnum>,
    edges: Vec<(String, String)>,
}

impl Network {
    pub fn new(name: &str) -> Self {
        Network {
            name: name.to_string(),
            components: HashMap::new(),
            edges: Vec::new(),
        }
    }

    // ---- building the network --------------------------------------------

    /// Register a component in the network under `component_id`.
    pub fn add(&mut self, component_id: &str, component: ComponentEnum) -> Result<(), String> {
        if self.components.contains_key(component_id) {
            return Err(format!("duplicate component id: {component_id:?}"));
        }
        let ports = component.as_component().ports().to_vec();
        self.components.insert(component_id.to_string(), component);

        // Register each port's stable node id and internal edges.
        for p in ports {
            let pid = port_node_id(component_id, &p.name);
            match p.direction {
                crate::components::base::PortDirection::In => {
                    // air enters: port -> component
                    self.edges.push((pid, component_id.to_string()));
                }
                crate::components::base::PortDirection::Out => {
                    // air leaves: component -> port
                    self.edges.push((component_id.to_string(), pid));
                }
            }
        }
        // Set node ids on the stored ports (after moving into the map).
        self.assign_port_node_ids(component_id);
        Ok(())
    }

    fn assign_port_node_ids(&mut self, component_id: &str) {
        let comp = self
            .components
            .get_mut(component_id)
            .expect("component present after add");
        for pp in comp.as_component_mut().ports_mut().iter_mut() {
            pp.node_id = port_node_id(component_id, &pp.name);
        }
    }

    /// Add a physical-airflow connection from `source` to `target`.
    ///
    /// Each endpoint is either `"<component_id>"` (the default port is used) or
    /// `"<component_id>.<port_name>"`. The source must resolve to an `out` port
    /// and the target to an `in` port.
    pub fn connect(&mut self, source: &str, target: &str) -> Result<(), String> {
        let (src_cid, src_port) = self.resolve(source, PortDirectionSimple::Out)?;
        let (dst_cid, dst_port) = self.resolve(target, PortDirectionSimple::In)?;
        self.edges.push((
            port_node_id(&src_cid, &src_port),
            port_node_id(&dst_cid, &dst_port),
        ));
        Ok(())
    }

    // ---- analysis ---------------------------------------------------------

    /// All graph node ids (component nodes + port nodes).
    pub fn node_ids(&self) -> Vec<String> {
        let mut ids: Vec<String> = Vec::new();
        for cid in self.components.keys() {
            ids.push(cid.clone());
            let comp = self.components[cid].as_component();
            for p in comp.ports() {
                ids.push(port_node_id(cid, &p.name));
            }
        }
        ids
    }

    /// Successor adjacency `node_id -> [successor_ids]`.
    pub fn successors(&self) -> HashMap<String, Vec<String>> {
        let mut succ: HashMap<String, Vec<String>> = HashMap::new();
        for (from, to) in &self.edges {
            succ.entry(from.clone()).or_default().push(to.clone());
        }
        succ
    }

    /// Predecessor adjacency `node_id -> [predecessor_ids]`.
    pub fn predecessors(&self) -> HashMap<String, Vec<String>> {
        let mut pred: HashMap<String, Vec<String>> = HashMap::new();
        for (from, to) in &self.edges {
            pred.entry(to.clone()).or_default().push(from.clone());
        }
        pred
    }

    /// Kahn's-algorithm topological order of graph nodes (a DAG).
    pub fn topo_order(&self) -> Result<Vec<String>, String> {
        let nodes = self.node_ids();
        let succ = self.successors();
        let mut indegree: HashMap<String, usize> = nodes.iter().map(|n| (n.clone(), 0)).collect();
        for (_, to) in &self.edges {
            *indegree.entry(to.clone()).or_default() += 1;
        }

        let mut queue: VecDeque<String> = nodes
            .iter()
            .filter(|n| indegree[*n] == 0)
            .cloned()
            .collect();
        let mut order = Vec::with_capacity(nodes.len());
        while let Some(n) = queue.pop_front() {
            order.push(n.clone());
            if let Some(nexts) = succ.get(&n) {
                for m in nexts {
                    let d = indegree.get_mut(m).expect("node in graph");
                    *d -= 1;
                    if *d == 0 {
                        queue.push_back(m.clone());
                    }
                }
            }
        }
        if order.len() != nodes.len() {
            return Err("graph contains a cycle; expected a DAG".into());
        }
        Ok(order)
    }

    /// The terminal components of the network.
    pub fn terminals(&self) -> Vec<&Terminal> {
        self.components
            .values()
            .filter_map(|c| match c {
                ComponentEnum::Terminal(t) => Some(t),
                _ => None,
            })
            .collect()
    }

    /// The source components of the network.
    pub fn sources(&self) -> Vec<&Source> {
        self.components
            .values()
            .filter_map(|c| match c {
                ComponentEnum::Source(s) => Some(s),
                _ => None,
            })
            .collect()
    }

    /// Iterate `(component_id, component)` pairs.
    pub fn iter_components(&self) -> impl Iterator<Item = (&String, &ComponentEnum)> {
        self.components.iter()
    }

    pub fn len(&self) -> usize {
        self.components.len()
    }

    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
    }

    pub fn get(&self, component_id: &str) -> Option<&ComponentEnum> {
        self.components.get(component_id)
    }

    /// Number of connection edges (graph connections, not internal edges).
    pub fn connection_count(&self) -> usize {
        self.edges.len()
    }

    /// Structural validation. Empty list means the network is healthy.
    pub fn validate(&self) -> Vec<String> {
        let mut problems = Vec::new();
        if self.sources().is_empty() {
            problems.push("no Source component".into());
        }
        if self.terminals().is_empty() {
            problems.push("no Terminal component".into());
        }
        // Connectedness: every component must be wired to at least one other.
        let pred = self.predecessors();
        let succ = self.successors();
        for cid in self.components.keys() {
            let comp = self.components[cid].as_component();
            let mut connected = false;
            for p in comp.ports() {
                let pid = port_node_id(cid, &p.name);
                let neighbours = match p.direction {
                    crate::components::base::PortDirection::Out => succ.get(&pid).cloned(),
                    crate::components::base::PortDirection::In => pred.get(&pid).cloned(),
                };
                if let Some(ns) = neighbours {
                    for n in ns {
                        // A connection edge targets/comes-from another component's port.
                        if self
                            .components
                            .keys()
                            .any(|c| n.starts_with(c) && n != cid.as_str())
                        {
                            connected = true;
                            break;
                        }
                    }
                }
                if connected {
                    break;
                }
            }
            if !connected {
                problems.push(format!("component {cid:?} is not connected"));
            }
        }
        problems
    }

    /// Run the full solver and return critical-path pressure drop [Pa].
    pub fn solve(&mut self, fluid: Option<&Fluid>) -> Result<f64, String> {
        solver::solve(self, fluid.unwrap_or(&STANDARD_AIR))
    }

    fn resolve(
        &self,
        ref_: &str,
        expected_direction: PortDirectionSimple,
    ) -> Result<(String, String), String> {
        let (cid, pname) = match ref_.split_once('.') {
            Some((c, p)) => (c.to_string(), Some(p.to_string())),
            None => (ref_.to_string(), None),
        };
        let comp = self
            .components
            .get(&cid)
            .ok_or_else(|| format!("unknown component id: {cid:?}"))?
            .as_component();

        let expected = match expected_direction {
            PortDirectionSimple::In => crate::components::base::PortDirection::In,
            PortDirectionSimple::Out => crate::components::base::PortDirection::Out,
        };

        let port_name: String = if let Some(pn) = pname {
            let port = comp.port(&pn).map_err(|e| e.to_string())?;
            if port.direction != expected {
                return Err(format!(
                    "port {cid}.{pn} is {:?}, expected {expected:?}",
                    port.direction
                ));
            }
            pn
        } else {
            let matching: Vec<&Port> = comp
                .ports()
                .iter()
                .filter(|p| p.direction == expected)
                .collect();
            match matching.len() {
                0 => {
                    return Err(format!(
                        "component {cid:?} has no {expected:?} ports"
                    ))
                }
                1 => matching[0].name.clone(),
                _ => {
                    return Err(format!(
                        "component {cid:?} has multiple {expected:?} ports; specify one with {cid:?} + '.<port_name>'"
                    ))
                }
            }
        };
        Ok((cid, port_name))
    }
}

#[derive(Debug, Clone, Copy)]
enum PortDirectionSimple {
    In,
    Out,
}

/// Build a simple `Source -> RigidDuct -> Terminal` network for tests/examples.
pub fn simple_supply_network(
    flowrate: f64,
    duct_length: f64,
    duct_diameter: f64,
) -> Result<Network, String> {
    use crate::core::geometry::Round;
    let mut net = Network::new("supply");
    net.add("ahu", ComponentEnum::Source(Source::new("AHU")))?;
    let r = Round::new(duct_diameter)?;
    net.add(
        "duct",
        ComponentEnum::RigidDuct(RigidDuct::new(
            "duct",
            r.area,
            r.hydraulic_diameter,
            duct_length,
            0.0001,
        )?),
    )?;
    net.add(
        "term",
        ComponentEnum::Terminal(Terminal::new("terminal", flowrate, Some(r.area), 1.0)),
    )?;
    net.connect("ahu", "duct")?;
    net.connect("duct", "term")?;
    Ok(net)
}
