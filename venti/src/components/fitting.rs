//! Fitting and terminal components.

use crate::core::fluid::Fluid;
use crate::physics::losses::local_pressure_drop;
use crate::Result;

use super::base::{Component, Port, PortDirection};

/// A flow source — typically the AHU/fan supplying the network.
///
/// `Source` has a single `out` port. Its flowrate is determined by the solver
/// as the sum of all downstream terminal demands.
#[derive(Debug, Clone)]
pub struct Source {
    pub name: String,
    ports: Vec<Port>,
}

impl Source {
    pub fn new(name: &str) -> Self {
        Source {
            name: name.to_string(),
            ports: vec![Port::new("outlet", PortDirection::Out)],
        }
    }
}

impl Component for Source {
    fn name(&self) -> &str {
        &self.name
    }
    fn ports(&self) -> &[Port] {
        &self.ports
    }
    fn ports_mut(&mut self) -> &mut [Port] {
        &mut self.ports
    }

    fn compute(&mut self, _fluid: &Fluid) -> Result<()> {
        // A pure source contributes no pressure drop of its own.
        self.ports[0].velocity = 0.0;
        self.ports[0].pressure_drop = 0.0;
        Ok(())
    }
}

/// A one-port terminal: diffuser, grille, register, or cap.
///
/// `flowrate` is the *demanded* volumetric flow [m³/s] at this terminal —
/// use 0 for a cap. If `cross_section` and `zeta` are supplied, the local
/// pressure drop of the terminal device is also computed.
#[derive(Debug, Clone)]
pub struct Terminal {
    pub name: String,
    pub flowrate_demand: f64,
    pub cross_section_area: f64, // 0 if none
    pub zeta: f64,
    ports: Vec<Port>,
}

impl Terminal {
    pub fn new(name: &str, flowrate: f64, cross_section_area: Option<f64>, zeta: f64) -> Self {
        let area = cross_section_area.unwrap_or(0.0);
        let mut port = Port::new("inlet", PortDirection::In);
        port.flowrate = Some(flowrate);
        Terminal {
            name: name.to_string(),
            flowrate_demand: flowrate,
            cross_section_area: area,
            zeta,
            ports: vec![port],
        }
    }
}

impl Component for Terminal {
    fn name(&self) -> &str {
        &self.name
    }
    fn ports(&self) -> &[Port] {
        &self.ports
    }
    fn ports_mut(&mut self) -> &mut [Port] {
        &mut self.ports
    }

    fn compute(&mut self, fluid: &Fluid) -> Result<()> {
        let port = &mut self.ports[0];
        let flow = port.flowrate.unwrap_or(0.0);
        if flow == 0.0 || self.cross_section_area <= 0.0 {
            port.velocity = 0.0;
            port.pressure_drop = 0.0;
            return Ok(());
        }
        let v = flow / self.cross_section_area;
        port.velocity = v;
        port.pressure_drop = local_pressure_drop(self.zeta, v, fluid.density);
        Ok(())
    }
}

/// A generic in-line fitting (elbow, reducer, transition, damper, ...).
///
/// The local pressure drop is computed from `zeta` referenced to the velocity
/// at `cross_section`. The drop is reported on the outlet port so it
/// accumulates correctly along the critical path.
#[derive(Debug, Clone)]
pub struct TwoPortFitting {
    pub name: String,
    pub area: f64,
    pub zeta: f64,
    ports: Vec<Port>,
    inv_area: f64,
}

impl TwoPortFitting {
    pub fn new(name: &str, area: f64, zeta: f64) -> Self {
        TwoPortFitting {
            name: name.to_string(),
            area,
            zeta,
            inv_area: 1.0 / area,
            ports: vec![
                Port::new("inlet", PortDirection::In),
                Port::new("outlet", PortDirection::Out),
            ],
        }
    }
}

impl Component for TwoPortFitting {
    fn name(&self) -> &str {
        &self.name
    }
    fn ports(&self) -> &[Port] {
        &self.ports
    }
    fn ports_mut(&mut self) -> &mut [Port] {
        &mut self.ports
    }

    fn compute(&mut self, fluid: &Fluid) -> Result<()> {
        let inlet_flow = self.ports[0]
            .flowrate
            .ok_or_else(|| format!("TwoPortFitting {:?}: inlet flowrate not set", self.name))?;
        let v = inlet_flow * self.inv_area;
        self.ports[0].velocity = v;
        self.ports[1].velocity = v;
        self.ports[1].flowrate = Some(inlet_flow);
        self.ports[0].pressure_drop = 0.0;
        self.ports[1].pressure_drop = local_pressure_drop(self.zeta, v, fluid.density);
        Ok(())
    }
}

/// A three-port branch fitting.
///
/// Ports: `combined` (in), `straight` (out), `branch` (out). Each leg has its
/// own loss coefficient. The drop is reported on the corresponding leg port.
#[derive(Debug, Clone)]
pub struct Tee {
    pub name: String,
    pub area: f64,
    pub zeta_straight: f64,
    pub zeta_branch: f64,
    ports: Vec<Port>,
    inv_area: f64,
}

impl Tee {
    pub fn new(name: &str, area: f64, zeta_straight: f64, zeta_branch: f64) -> Self {
        Tee {
            name: name.to_string(),
            area,
            zeta_straight,
            zeta_branch,
            inv_area: 1.0 / area,
            ports: vec![
                Port::new("combined", PortDirection::In),
                Port::new("straight", PortDirection::Out),
                Port::new("branch", PortDirection::Out),
            ],
        }
    }
}

impl Component for Tee {
    fn name(&self) -> &str {
        &self.name
    }
    fn ports(&self) -> &[Port] {
        &self.ports
    }
    fn ports_mut(&mut self) -> &mut [Port] {
        &mut self.ports
    }

    fn compute(&mut self, fluid: &Fluid) -> Result<()> {
        let straight_flow = self.ports[1]
            .flowrate
            .ok_or_else(|| format!("Tee {:?}: leg flowrates not set", self.name))?;
        let branch_flow = self.ports[2]
            .flowrate
            .ok_or_else(|| format!("Tee {:?}: leg flowrates not set", self.name))?;
        let inv_a = self.inv_area;
        let rho = fluid.density;
        let v_s = straight_flow * inv_a;
        let v_b = branch_flow * inv_a;
        let v_c = (straight_flow + branch_flow) * inv_a;

        self.ports[0].flowrate = Some(straight_flow + branch_flow);
        self.ports[0].velocity = v_c;
        self.ports[0].pressure_drop = 0.0;
        self.ports[1].velocity = v_s;
        self.ports[2].velocity = v_b;
        self.ports[1].pressure_drop = local_pressure_drop(self.zeta_straight, v_s, rho);
        self.ports[2].pressure_drop = local_pressure_drop(self.zeta_branch, v_b, rho);
        Ok(())
    }
}
