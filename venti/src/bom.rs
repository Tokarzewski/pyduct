//! Bill of materials (BOM / *zestawienie*) for a ductwork network.
//!
//! Builds a fabrication-oriented parts list on top of [`crate::results`] and
//! [`crate::fabrication`]: for every component in a [`crate::network::Network`]
//! it produces one [`BomItem`] carrying the component identity, the fabricated
//! straight-duct **length** and the wetted **surface area** to be cut/ordered.
//!
//! Non-duct components (sources, terminals, fittings, tees) contribute zero
//! length and zero area — the sheet-metal quantities live on straight duct runs
//! only.
//!
//! All quantities are SI: length `m`, area `m²`.

use crate::core::geometry::{CrossSection, Round};
use crate::fabrication::duct_surface_area_m2;
use crate::network::{ComponentEnum, Network};
use crate::Result;

#[cfg(test)]
use crate::components::duct::RigidDuct;
#[cfg(test)]
use crate::components::fitting::{Source, Terminal, TwoPortFitting};

/// One row of the bill of materials: a component and its fabrication
/// quantities.
#[derive(Debug, Clone, PartialEq)]
pub struct BomItem {
    /// Network component id (node key in the [`Network`]).
    pub component_id: String,
    /// Rust component kind, e.g. `"RigidDuct"`, `"FlexDuct"`, `"Source"`.
    pub kind: String,
    /// Straight duct length of this component [m] (0 for non-duct items).
    pub length_m: f64,
    /// Fabricated surface (sheet-metal) area of this component [m²]
    /// (0 for non-duct items).
    pub area_m2: f64,
}

/// Build a bill of materials for a network: one [`BomItem`] per component.
///
/// Straight-duct components (`RigidDuct`, `FlexDuct`) report their length and
/// the wetted surface area of the run. Every other component (source, terminal,
/// two-port fitting, tee) reports zero length and zero area.
///
/// # Errors
///
/// Returns an error if a duct's hydraulic geometry cannot be turned into a
/// valid cross-section (e.g. a non-positive diameter).
///
/// # Examples
///
/// ```
/// use venti::{
///     build_bom, bom_as_csv, total_length, ComponentEnum, Network, RigidDuct,
///     Round, Source, TwoPortFitting, Terminal,
/// };
///
/// // A chain: source → 10 m round rigid duct (D = 0.2 m) → elbow → terminal.
/// let r = Round::new(0.2).unwrap();
/// let mut net = Network::new("bom");
/// net.add("ahu", ComponentEnum::Source(Source::new("AHU"))).unwrap();
/// net.add("duct", ComponentEnum::RigidDuct(
///     RigidDuct::new("duct", r.area, r.hydraulic_diameter, 10.0, 0.0001).unwrap()
/// )).unwrap();
/// net.add("fit", ComponentEnum::TwoPortFitting(
///     TwoPortFitting::new("elbow", r.area, 0.5)
/// )).unwrap();
/// net.add("term", ComponentEnum::Terminal(
///     Terminal::new("term", 0.1, Some(r.area), 1.0)
/// )).unwrap();
/// net.connect("ahu", "duct").unwrap();
/// net.connect("duct", "fit").unwrap();
/// net.connect("fit", "term").unwrap();
///
/// let items = build_bom(&net).unwrap();
/// assert_eq!(items.len(), 4);
///
/// // The only non-zero-length item is the duct: 10 m, area = π·0.2·10 m².
/// let duct = items.iter().find(|i| i.component_id == "duct").unwrap();
/// assert_eq!(duct.length_m, 10.0);
/// assert!((duct.area_m2 - std::f64::consts::PI * 0.2 * 10.0).abs() < 1e-12);
/// assert!((total_length(&items) - 10.0).abs() < 1e-12);
///
/// // CSV round-trips the header and all rows.
/// let csv = bom_as_csv(&items);
/// assert_eq!(csv.lines().next().unwrap(), "component_id,kind,length_m,area_m2");
/// assert_eq!(csv.lines().count(), 5); // header + 4 items
/// ```
pub fn build_bom(network: &Network) -> Result<Vec<BomItem>> {
    let mut items = Vec::with_capacity(network.len());
    for (cid, comp) in network.iter_components() {
        let kind = comp.kind().to_string();
        let (length_m, area_m2) = match comp {
            ComponentEnum::RigidDuct(d) => {
                // Rigid ducts in this crate are round; recover the section from
                // the hydraulic diameter and reuse the fabrication surface area.
                let cs = CrossSection::Round(Round::new(d.hydraulic_diameter)?);
                let area = duct_surface_area_m2(&cs, d.length)?;
                (d.length, area)
            }
            ComponentEnum::FlexDuct(d) => {
                let cs = CrossSection::Round(Round::new(d.diameter)?);
                let area = duct_surface_area_m2(&cs, d.length)?;
                (d.length, area)
            }
            _ => (0.0, 0.0),
        };
        items.push(BomItem {
            component_id: cid.clone(),
            kind,
            length_m,
            area_m2,
        });
    }
    Ok(items)
}

/// Serialize a bill of materials as a CSV string with a header row.
///
/// Header is `component_id,kind,length_m,area_m2`. Values are written with
/// [`to_string`](std::string::ToString) so the raw floating-point representations
/// are preserved.
///
/// # Examples
///
/// ```
/// use venti::{BomItem, bom_as_csv};
///
/// let items = vec![
///     BomItem { component_id: "duct".into(), kind: "RigidDuct".into(),
///               length_m: 10.0, area_m2: 3.14 },
/// ];
/// let csv = bom_as_csv(&items);
/// assert!(
///     csv.lines().next().unwrap()
///         == "component_id,kind,length_m,area_m2"
/// );
/// assert!(csv.contains("duct,RigidDuct"));
/// ```
pub fn bom_as_csv(items: &[BomItem]) -> String {
    if items.is_empty() {
        return "component_id,kind,length_m,area_m2".to_string();
    }
    let mut lines = vec!["component_id,kind,length_m,area_m2".to_string()];
    for it in items {
        lines.push(format!(
            "{},{},{},{}",
            it.component_id, it.kind, it.length_m, it.area_m2
        ));
    }
    lines.join("\n")
}

/// Total straight-duct length across all BOM items [m].
pub fn total_length(items: &[BomItem]) -> f64 {
    items.iter().map(|i| i.length_m).sum()
}

/// Total fabricated surface area across all BOM items [m²].
pub fn total_area(items: &[BomItem]) -> f64 {
    items.iter().map(|i| i.area_m2).sum()
}

/// Build the standard Source → RigidDuct → TwoPortFitting → Terminal chain
/// used by the tests below.
#[cfg(test)]
fn chain_network() -> Network {
    let r = Round::new(0.2).unwrap();
    let mut net = Network::new("bom-chain");
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
    net
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::duct::FlexDuct;

    #[test]
    fn chain_yields_four_items() {
        let net = chain_network();
        let items = build_bom(&net).unwrap();
        assert_eq!(items.len(), 4);
    }

    #[test]
    fn duct_item_has_length_and_round_surface_area() {
        let net = chain_network();
        let items = build_bom(&net).unwrap();
        let duct = items.iter().find(|i| i.component_id == "duct").unwrap();
        assert_eq!(duct.kind, "RigidDuct");
        assert!((duct.length_m - 10.0).abs() < 1e-12);
        // D = 0.2, L = 10  →  A = π · 0.2 · 10.
        assert!((duct.area_m2 - PI * 0.2 * 10.0).abs() < 1e-12);
    }

    #[test]
    fn total_length_counts_only_the_duct() {
        let net = chain_network();
        let items = build_bom(&net).unwrap();
        assert!((total_length(&items) - 10.0).abs() < 1e-12);
    }

    #[test]
    fn csv_has_header_and_all_rows() {
        let net = chain_network();
        let items = build_bom(&net).unwrap();
        let csv = bom_as_csv(&items);
        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(lines[0], "component_id,kind,length_m,area_m2");
        assert_eq!(lines.len(), 5); // header + 4 items
                                    // Every row has exactly the 4 fields and includes the duct.
        assert!(lines.iter().skip(1).all(|l| l.split(',').count() == 4));
        assert!(lines.iter().any(|l| l.starts_with("duct,")));
    }

    #[test]
    fn non_duct_items_have_zero_length_and_area() {
        let net = chain_network();
        let items = build_bom(&net).unwrap();
        for it in items {
            if it.component_id == "duct" {
                continue;
            }
            assert_eq!(it.length_m, 0.0);
            assert_eq!(it.area_m2, 0.0);
        }
    }

    #[test]
    fn flex_duct_reports_round_surface_area() {
        let mut net = Network::new("flex");
        net.add(
            "flex",
            ComponentEnum::FlexDuct(FlexDuct::new("flex", 0.15, 8.0, 1.5, 10.0).unwrap()),
        )
        .unwrap();
        let items = build_bom(&net).unwrap();
        assert_eq!(items.len(), 1);
        let f = &items[0];
        assert_eq!(f.kind, "FlexDuct");
        assert!((f.length_m - 8.0).abs() < 1e-12);
        // A = π · 0.15 · 8.
        assert!((f.area_m2 - PI * 0.15 * 8.0).abs() < 1e-12);
        assert!((total_length(&items) - 8.0).abs() < 1e-12);
        assert!((total_area(&items) - PI * 0.15 * 8.0).abs() < 1e-12);
    }

    #[test]
    fn total_area_sums_duct_areas() {
        let net = chain_network();
        let items = build_bom(&net).unwrap();
        assert!((total_area(&items) - PI * 0.2 * 10.0).abs() < 1e-12);
    }

    use std::f64::consts::PI;
}
