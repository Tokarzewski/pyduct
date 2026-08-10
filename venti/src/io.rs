//! Serialize / deserialize ductwork networks (YAML/JSON).
//!
//! Mirrors `python/wenta/io.py` + `schemas.py` (SPEC FR-16 / roadmap Phase-0
//! network-I/O item). Gated behind the `cli` feature so the core library
//! stays dependency-free — the plugin can enable this feature to load/save
//! designs in the `wenta` YAML/JSON format.
//!
//! ```text
//! name: My network
//! components:
//!   ahu:  { type: Source, name: AHU }
//!   duct: { type: RigidDuct, cross_section: { shape: round, diameter: 0.2 }, length: 10 }
//!   term: { type: Terminal, flowrate: 0.1 }
//! connections:
//!   - { source: ahu, target: duct }
//!   - { source: duct, target: term }
//! ```

use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

use crate::components::duct::{FlexDuct, RigidDuct};
use crate::components::fitting::{Source, Tee, Terminal, TwoPortFitting};
use crate::core::geometry::{CrossSection, Rectangular, Round};
use crate::network::{ComponentEnum, Network};

#[derive(Deserialize)]
struct NetworkFile {
    name: Option<String>,
    components: HashMap<String, ComponentFile>,
    connections: Vec<ConnectionFile>,
}

#[derive(Deserialize)]
struct ConnectionFile {
    source: String,
    target: String,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "PascalCase")]
enum ComponentFile {
    Source {
        name: Option<String>,
    },
    RigidDuct {
        name: Option<String>,
        cross_section: CrossSectionFile,
        length: f64,
        #[serde(default = "default_roughness")]
        absolute_roughness: f64,
    },
    FlexDuct {
        name: Option<String>,
        diameter: f64,
        length: f64,
        pressure_drop_per_meter: f64,
        #[serde(default = "default_stretch")]
        stretch_percentage: f64,
    },
    TwoPortFitting {
        name: Option<String>,
        cross_section: CrossSectionFile,
        zeta: f64,
    },
    Tee {
        name: Option<String>,
        cross_section: CrossSectionFile,
        #[serde(default)]
        zeta_straight: f64,
        #[serde(default = "default_zeta_branch")]
        zeta_branch: f64,
    },
    Terminal {
        name: Option<String>,
        flowrate: f64,
        #[serde(default)]
        zeta: f64,
        #[serde(default)]
        cross_section: Option<CrossSectionFile>,
    },
}

#[derive(Deserialize, Clone)]
struct CrossSectionFile {
    #[serde(rename = "shape")]
    shape: String,
    #[serde(default)]
    diameter: Option<f64>,
    #[serde(default)]
    width: Option<f64>,
    #[serde(default)]
    height: Option<f64>,
}

fn default_roughness() -> f64 {
    0.0001
}
fn default_stretch() -> f64 {
    100.0
}
fn default_zeta_branch() -> f64 {
    0.5
}

fn build_cross_section(cs: &CrossSectionFile) -> Result<CrossSection, String> {
    match cs.shape.as_str() {
        "round" => Ok(CrossSection::Round(Round::new(
            cs.diameter.ok_or("round cross_section needs diameter")?,
        )?)),
        "rectangular" => Ok(CrossSection::Rectangular(Rectangular::new(
            cs.width.ok_or("rectangular cross_section needs width")?,
            cs.height.ok_or("rectangular cross_section needs height")?,
        )?)),
        other => Err(format!("unknown cross-section shape {other:?}")),
    }
}

fn build_component(c: ComponentFile) -> Result<ComponentEnum, String> {
    let comp = match c {
        ComponentFile::Source { name } => {
            ComponentEnum::Source(Source::new(name.as_deref().unwrap_or("Source")))
        }
        ComponentFile::RigidDuct {
            name,
            cross_section,
            length,
            absolute_roughness,
        } => {
            let cs = build_cross_section(&cross_section)?;
            ComponentEnum::RigidDuct(RigidDuct::new(
                name.as_deref().unwrap_or("RigidDuct"),
                cs.area(),
                cs.hydraulic_diameter(),
                length,
                absolute_roughness,
            )?)
        }
        ComponentFile::FlexDuct {
            name,
            diameter,
            length,
            pressure_drop_per_meter,
            stretch_percentage,
        } => ComponentEnum::FlexDuct(FlexDuct::new(
            name.as_deref().unwrap_or("FlexDuct"),
            diameter,
            length,
            pressure_drop_per_meter,
            stretch_percentage,
        )?),
        ComponentFile::TwoPortFitting {
            name,
            cross_section,
            zeta,
        } => {
            let cs = build_cross_section(&cross_section)?;
            ComponentEnum::TwoPortFitting(TwoPortFitting::new(
                name.as_deref().unwrap_or("Fitting"),
                cs.area(),
                zeta,
            ))
        }
        ComponentFile::Tee {
            name,
            cross_section,
            zeta_straight,
            zeta_branch,
        } => {
            let cs = build_cross_section(&cross_section)?;
            ComponentEnum::Tee(Tee::new(
                name.as_deref().unwrap_or("Tee"),
                cs.area(),
                zeta_straight,
                zeta_branch,
            ))
        }
        ComponentFile::Terminal {
            name,
            flowrate,
            zeta,
            cross_section,
        } => {
            let area = cross_section
                .as_ref()
                .map(|cs| build_cross_section(cs).unwrap().area());
            ComponentEnum::Terminal(Terminal::new(
                name.as_deref().unwrap_or("Terminal"),
                flowrate,
                area,
                zeta,
            ))
        }
    };
    Ok(comp)
}

/// Parse a network from a YAML or JSON string (format guessed by content).
pub fn load_network_from_str(text: &str) -> Result<Network, String> {
    let nf: NetworkFile = serde_yaml::from_str(text).map_err(|e| format!("parse: {e}"))?;
    build_network(nf)
}

fn build_network(nf: NetworkFile) -> Result<Network, String> {
    let mut net = Network::new(nf.name.as_deref().unwrap_or(""));
    for (cid, comp) in nf.components {
        net.add(&cid, build_component(comp)?)?;
    }
    for conn in nf.connections {
        net.connect(&conn.source, &conn.target)?;
    }
    Ok(net)
}

/// Load a network from a `.yaml` or `.json` file (chosen by extension).
pub fn load_network_from_path(path: &Path) -> Result<Network, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("read {path:?}: {e}"))?;
    let is_json = path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase() == "json")
        .unwrap_or(false);
    if is_json {
        let nf: NetworkFile =
            serde_json::from_str(&text).map_err(|e| format!("JSON parse: {e}"))?;
        build_network(nf)
    } else {
        load_network_from_str(&text)
    }
}

/// Convenience: load from a path-like `&str`.
pub fn load_network_from_file(path: &str) -> Result<Network, String> {
    load_network_from_path(Path::new(path))
}
