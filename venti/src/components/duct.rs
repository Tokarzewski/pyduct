//! Straight duct components.

use crate::core::fluid::Fluid;
use crate::physics::flex::stretch_correction_factor;
use crate::physics::friction::{friction_factor, relative_roughness, reynolds};
use crate::physics::losses::straight_pressure_drop;

use super::base::{Component, Port, PortDirection};

/// A rigid (sheet-metal) straight duct.
///
/// The full Darcy–Weisbach pressure drop is reported on the inlet port; the
/// outlet port carries 0 so no double-counting occurs along a critical path.
#[derive(Debug, Clone)]
pub struct RigidDuct {
    pub name: String,
    pub area: f64,
    pub hydraulic_diameter: f64,
    pub length: f64,
    pub absolute_roughness: f64,
    ports: Vec<Port>,
    // cached
    eps: f64,
}

impl RigidDuct {
    pub fn new(
        name: &str,
        area: f64,
        hydraulic_diameter: f64,
        length: f64,
        absolute_roughness: f64,
    ) -> Result<Self, String> {
        if length <= 0.0 {
            return Err(format!("length must be positive, got {length}"));
        }
        let eps = relative_roughness(absolute_roughness, hydraulic_diameter);
        Ok(RigidDuct {
            name: name.to_string(),
            area,
            hydraulic_diameter,
            length,
            absolute_roughness,
            eps,
            ports: vec![
                Port::new("inlet", PortDirection::In),
                Port::new("outlet", PortDirection::Out),
            ],
        })
    }
}

impl Component for RigidDuct {
    fn name(&self) -> &str {
        &self.name
    }
    fn ports(&self) -> &[Port] {
        &self.ports
    }
    fn ports_mut(&mut self) -> &mut [Port] {
        &mut self.ports
    }

    fn compute(&mut self, fluid: &Fluid) -> Result<(), String> {
        let inlet_flow = self.ports[0]
            .flowrate
            .ok_or_else(|| format!("RigidDuct {:?}: inlet flowrate not set", self.name))?;
        let v = inlet_flow / self.area;
        let re = reynolds(v, self.hydraulic_diameter, fluid.kinematic_viscosity);
        let f = friction_factor(re, self.eps);

        self.ports[0].velocity = v;
        self.ports[1].velocity = v;
        self.ports[1].flowrate = Some(inlet_flow);
        self.ports[0].pressure_drop =
            straight_pressure_drop(f, self.length, self.hydraulic_diameter, v, fluid.density);
        self.ports[1].pressure_drop = 0.0;
        Ok(())
    }
}

/// A flexible round duct with manufacturer-supplied per-meter pressure drop.
///
/// Because flex pressure-drop curves are highly product-specific, this
/// component takes the per-meter drop as an explicit parameter rather than
/// trying to derive it from a friction factor.
#[derive(Debug, Clone)]
pub struct FlexDuct {
    pub name: String,
    pub diameter: f64,                // [m]
    pub length: f64,                  // [m]
    pub pressure_drop_per_meter: f64, // [Pa/m]
    pub stretch_percentage: f64,
    ports: Vec<Port>,
    area: f64,
}

impl FlexDuct {
    pub fn new(
        name: &str,
        diameter: f64,
        length: f64,
        pressure_drop_per_meter: f64,
        stretch_percentage: f64,
    ) -> Result<Self, String> {
        if diameter <= 0.0 || length <= 0.0 {
            return Err("diameter and length must be positive".to_string());
        }
        if !(0.0 < stretch_percentage && stretch_percentage <= 100.0) {
            return Err(format!(
                "stretch_percentage must be in (0, 100], got {stretch_percentage}"
            ));
        }
        let r = diameter * 0.5;
        let area = std::f64::consts::PI * r * r;
        Ok(FlexDuct {
            name: name.to_string(),
            diameter,
            length,
            pressure_drop_per_meter,
            stretch_percentage,
            area,
            ports: vec![
                Port::new("inlet", PortDirection::In),
                Port::new("outlet", PortDirection::Out),
            ],
        })
    }
}

impl Component for FlexDuct {
    fn name(&self) -> &str {
        &self.name
    }
    fn ports(&self) -> &[Port] {
        &self.ports
    }
    fn ports_mut(&mut self) -> &mut [Port] {
        &mut self.ports
    }

    fn compute(&mut self, fluid: &Fluid) -> Result<(), String> {
        let inlet_flow = self.ports[0]
            .flowrate
            .ok_or_else(|| format!("FlexDuct {:?}: inlet flowrate not set", self.name))?;
        let v = inlet_flow / self.area;
        let beta = stretch_correction_factor(self.diameter, self.stretch_percentage);
        self.ports[0].velocity = v;
        self.ports[1].velocity = v;
        self.ports[1].flowrate = Some(inlet_flow);
        self.ports[0].pressure_drop = self.pressure_drop_per_meter * self.length * beta;
        self.ports[1].pressure_drop = 0.0;
        let _ = fluid;
        Ok(())
    }
}
