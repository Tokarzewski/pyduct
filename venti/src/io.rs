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

use crate::Result;
use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::components::duct::{FlexDuct, RigidDuct};
use crate::components::fitting::{Source, Tee, Terminal, TwoPortFitting};
use crate::core::geometry::{CrossSection, Rectangular, Round};
use crate::network::{ComponentEnum, Network};

#[derive(Serialize, Deserialize)]
struct NetworkFile {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    components: HashMap<String, ComponentFile>,
    connections: Vec<ConnectionFile>,
}

#[derive(Serialize, Deserialize)]
struct ConnectionFile {
    source: String,
    target: String,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "PascalCase")]
enum ComponentFile {
    Source {
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
    },
    RigidDuct {
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        cross_section: CrossSectionFile,
        length: f64,
        #[serde(default = "default_roughness")]
        absolute_roughness: f64,
    },
    FlexDuct {
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        diameter: f64,
        length: f64,
        pressure_drop_per_meter: f64,
        #[serde(default = "default_stretch")]
        stretch_percentage: f64,
    },
    TwoPortFitting {
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        cross_section: CrossSectionFile,
        zeta: f64,
    },
    Tee {
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        cross_section: CrossSectionFile,
        #[serde(default)]
        zeta_straight: f64,
        #[serde(default = "default_zeta_branch")]
        zeta_branch: f64,
    },
    Terminal {
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        flowrate: f64,
        #[serde(default)]
        zeta: f64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        cross_section: Option<CrossSectionFile>,
    },
}

#[derive(Serialize, Deserialize, Clone)]
struct CrossSectionFile {
    #[serde(rename = "shape")]
    shape: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    diameter: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    width: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
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

fn build_cross_section(cs: &CrossSectionFile) -> Result<CrossSection> {
    match cs.shape.as_str() {
        "round" => Ok(CrossSection::Round(Round::new(
            cs.diameter.ok_or("round cross_section needs diameter")?,
        )?)),
        "rectangular" => Ok(CrossSection::Rectangular(Rectangular::new(
            cs.width.ok_or("rectangular cross_section needs width")?,
            cs.height.ok_or("rectangular cross_section needs height")?,
        )?)),
        other => Err((format!("unknown cross-section shape {other:?}")).into()),
    }
}

fn build_component(c: ComponentFile) -> Result<ComponentEnum> {
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
pub fn load_network_from_str(text: &str) -> Result<Network> {
    let nf: NetworkFile = serde_yaml::from_str(text).map_err(|e| format!("parse: {e}"))?;
    build_network(nf)
}

fn build_network(nf: NetworkFile) -> Result<Network> {
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
pub fn load_network_from_path(path: &Path) -> Result<Network> {
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
pub fn load_network_from_file(path: &str) -> Result<Network> {
    load_network_from_path(Path::new(path))
}

// ---------------------------------------------------------------------------
// Save / serialize (SAVE support — round-trips the schema `load` reads)
// ---------------------------------------------------------------------------

/// Build a `round` cross-section whose area reconstructs to `area`.
///
/// Used for components that only carry an `area` (Terminal, TwoPortFitting,
/// Tee) — their loader consumes only `cross_section.area()`, so a round section
/// derived from the stored area round-trips exactly.
fn cross_section_from_area(area: f64) -> CrossSectionFile {
    let d = (4.0 * area / std::f64::consts::PI).sqrt();
    CrossSectionFile {
        shape: "round".into(),
        diameter: Some(d),
        width: None,
        height: None,
    }
}

/// Reconstruct a serializable cross-section from stored `area` and
/// `hydraulic_diameter` so the loader reproduces both exactly.
///
/// A section whose area is consistent with `π (dh/2)²` is round (diameter =
/// hydraulic diameter); otherwise it is rectangular, and `width`/`height` are
/// recovered as the two roots of `x² - (2·area/dh)·x + area = 0` (derived from
/// `w·h = area` and `D_h = 2wh/(w+h)`). Either branch rebuilds the same area
/// and hydraulic diameter, preserving the RigidDuct round-trip.
fn reconstruct_cross_section(area: f64, dh: f64) -> CrossSectionFile {
    let round_area = std::f64::consts::PI * (dh / 2.0).powi(2);
    let round = (area - round_area).abs() <= 1e-9 * area.max(round_area);
    if round {
        return CrossSectionFile {
            shape: "round".into(),
            diameter: Some(dh),
            width: None,
            height: None,
        };
    }
    // Rectangular: solve w,h from A=wh and Dh=2wh/(w+h).
    let sum = 2.0 * area / dh; // w + h
    let disc = sum * sum - 4.0 * area;
    if disc > 0.0 {
        let s = disc.sqrt();
        CrossSectionFile {
            shape: "rectangular".into(),
            diameter: None,
            width: Some((sum + s) * 0.5),
            height: Some((sum - s) * 0.5),
        }
    } else {
        // Degenerate/defensive fallback: keep it round from area alone.
        cross_section_from_area(area)
    }
}

fn component_to_file(comp: &ComponentEnum) -> ComponentFile {
    match comp {
        ComponentEnum::Source(s) => ComponentFile::Source {
            name: Some(s.name.clone()),
        },
        ComponentEnum::RigidDuct(d) => ComponentFile::RigidDuct {
            name: Some(d.name.clone()),
            cross_section: reconstruct_cross_section(d.area, d.hydraulic_diameter),
            length: d.length,
            absolute_roughness: d.absolute_roughness,
        },
        ComponentEnum::FlexDuct(fd) => ComponentFile::FlexDuct {
            name: Some(fd.name.clone()),
            diameter: fd.diameter,
            length: fd.length,
            pressure_drop_per_meter: fd.pressure_drop_per_meter,
            stretch_percentage: fd.stretch_percentage,
        },
        ComponentEnum::TwoPortFitting(f) => ComponentFile::TwoPortFitting {
            name: Some(f.name.clone()),
            cross_section: cross_section_from_area(f.area),
            zeta: f.zeta,
        },
        ComponentEnum::Tee(t) => ComponentFile::Tee {
            name: Some(t.name.clone()),
            cross_section: cross_section_from_area(t.area),
            zeta_straight: t.zeta_straight,
            zeta_branch: t.zeta_branch,
        },
        ComponentEnum::Terminal(t) => ComponentFile::Terminal {
            name: Some(t.name.clone()),
            flowrate: t.flowrate_demand,
            zeta: t.zeta,
            cross_section: if t.cross_section_area > 0.0 {
                Some(cross_section_from_area(t.cross_section_area))
            } else {
                None
            },
        },
    }
}

fn network_to_file(net: &Network) -> NetworkFile {
    let mut components = HashMap::new();
    for (cid, comp) in &net.components {
        components.insert(cid.clone(), component_to_file(comp));
    }
    let connections = net
        .connections()
        .into_iter()
        .map(|(source, target)| ConnectionFile { source, target })
        .collect();
    NetworkFile {
        name: if net.name.is_empty() {
            None
        } else {
            Some(net.name.clone())
        },
        components,
        connections,
    }
}

/// Serialize a network to a wenta YAML string.
pub fn save_network_to_string(net: &Network) -> Result<String> {
    Ok(serde_yaml::to_string(&network_to_file(net)).map_err(|e| format!("serialize YAML: {e}"))?)
}

/// Serialize a network to a wenta JSON string.
pub fn save_network_to_json_string(net: &Network) -> Result<String> {
    Ok(serde_json::to_string_pretty(&network_to_file(net))
        .map_err(|e| format!("serialize JSON: {e}"))?)
}

/// Write a network to a `.yaml` or `.json` file (chosen by extension).
pub fn save_network_to_path(net: &Network, path: &Path) -> Result<()> {
    let is_json = path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase() == "json")
        .unwrap_or(false);
    let text = if is_json {
        save_network_to_json_string(net)?
    } else {
        save_network_to_string(net)?
    };
    Ok(std::fs::write(path, text).map_err(|e| format!("write {path:?}: {e}"))?)
}

/// Convenience: save to a path-like `&str`.
pub fn save_network_to_file(net: &Network, path: &str) -> Result<()> {
    save_network_to_path(net, Path::new(path))
}
