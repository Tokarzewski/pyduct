//! Base types shared by all ductwork components.

use crate::core::fluid::Fluid;

/// Direction of airflow through a port (physical convention).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortDirection {
    /// Air enters the component through this port.
    In,
    /// Air leaves the component through this port.
    Out,
}

/// A connection point on a `Component`.
///
/// A `Port` carries the local flow state (flowrate, velocity) and the local
/// pressure drop attributed to it. Ports are owned by exactly one component
/// and identified by their `name` within that component.
#[derive(Debug, Clone)]
pub struct Port {
    pub name: String,
    pub direction: PortDirection,
    /// Volumetric flow [m³/s], set once flowrates are propagated.
    pub flowrate: Option<f64>,
    /// Flow velocity [m/s], populated by `compute`.
    pub velocity: f64,
    /// Local pressure drop [Pa].
    pub pressure_drop: f64,
    /// Graph node id (`"{component_id}:{port_name}"`), set by `Network::add`.
    pub node_id: String,
}

impl Port {
    pub fn new(name: &str, direction: PortDirection) -> Self {
        Port {
            name: name.to_string(),
            direction,
            flowrate: None,
            velocity: 0.0,
            pressure_drop: 0.0,
            node_id: String::new(),
        }
    }

    #[inline]
    pub fn velocity(&self) -> f64 {
        self.velocity
    }
}

/// A piece of ductwork (duct, fitting, terminal) with one or more ports.
///
/// Subtypes populate `ports` and implement `compute` to fill in the velocity
/// and pressure drop on each port given the upstream flowrates and a `Fluid`.
pub trait Component {
    /// The component name (matches the `name` field on concrete types).
    fn name(&self) -> &str;

    /// This component's ports (must be non-empty).
    fn ports(&self) -> &[Port];

    /// Mutable access to ports, used when the solver propagates flowrates.
    fn ports_mut(&mut self) -> &mut [Port];

    /// Populate `velocity` and `pressure_drop` on each port.
    ///
    /// Called by the solver after flowrates have been propagated. The
    /// flowrate on each port must already be set when this is called.
    fn compute(&mut self, fluid: &Fluid) -> Result<(), String>;

    /// Look up a port by name.
    fn port(&self, name: &str) -> Result<&Port, String> {
        self.ports().iter().find(|p| p.name == name).ok_or_else(|| {
            format!(
                "{} has no port named {:?}",
                std::any::type_name::<Self>(),
                name
            )
        })
    }

    /// Inlet ports (air enters here).
    fn inlets(&self) -> Vec<&Port> {
        self.ports()
            .iter()
            .filter(|p| p.direction == PortDirection::In)
            .collect()
    }

    /// Outlet ports (air leaves here).
    fn outlets(&self) -> Vec<&Port> {
        self.ports()
            .iter()
            .filter(|p| p.direction == PortDirection::Out)
            .collect()
    }
}
