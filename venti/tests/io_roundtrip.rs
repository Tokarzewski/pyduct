//! Round-trip integration test: load a wenta YAML network, save it back out,
//! reload it, and assert the two networks are equal AND that the critical-path
//! pressure drop is preserved (issue #4 — SAVE / serialization round-trip).
//!
//! `examples/network_yaml.yaml` is a 3-zone supply network whose known
//! critical-path ΔP is 84.0632 Pa.

use std::path::{Path, PathBuf};

use venti::{load_network_from_path, save_network_to_json_string, save_network_to_path};

/// The 4-decimal rounded critical-path ΔP of the 3-zone supply network.
const ANCHOR_DP: f64 = 84.0632;

/// True when `dp` matches the known critical-path anchor within a relative
/// 1e-6 tolerance (the anchor printed the solve to 4 decimals).
fn matches_anchor(dp: f64) -> bool {
    (dp - ANCHOR_DP).abs() <= ANCHOR_DP * 1e-6
}

fn example_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/network_yaml.yaml")
}

/// Stable signature of a network for equality comparison: sorted
/// `(component_id, type)` pairs plus sorted qualified `(source, target)` edges.
/// Stable signature of a network for equality comparison: sorted
/// `(component_id, type)` pairs plus sorted qualified `(source, target)` edges.
type Signature = (Vec<(String, String)>, Vec<(String, String)>);

fn signature(net: &venti::Network) -> Signature {
    let mut comps: Vec<(String, String)> = net
        .iter_components()
        .map(|(id, c)| (id.clone(), c.kind().to_string()))
        .collect();
    comps.sort();

    let mut edges: Vec<(String, String)> = net.connections();
    edges.sort();

    (comps, edges)
}

#[test]
fn yaml_roundtrip_preserves_critical_path() {
    let mut net = load_network_from_path(&example_path()).expect("load example");
    let dp_before = net.solve(None).expect("solve original");
    assert!(
        matches_anchor(dp_before),
        "expected critical-path ΔP ≈ 84.0632, got {dp_before}"
    );

    // Save to a temp file (default extension selects YAML), then reload it.
    let tmp = std::env::temp_dir().join(format!(
        "venti_io_roundtrip_yaml_{}.yaml",
        std::process::id()
    ));
    save_network_to_path(&net, &tmp).expect("save YAML");

    // The network must survive the save/load round-trip structurally...
    let mut reloaded = load_network_from_path(&tmp).expect("reload YAML");
    assert_eq!(
        signature(&net),
        signature(&reloaded),
        "networks must be equal"
    );

    // ...and the solve result must match to the same critical-path ΔP.
    let dp_after = reloaded.solve(None).expect("solve reloaded");
    assert!(
        matches_anchor(dp_after),
        "reloaded critical-path ΔP drifted: got {dp_after}"
    );
    assert!(
        (dp_after - dp_before).abs() < 1e-9,
        "round-trip changed critical-path ΔP: {dp_before} -> {dp_after}"
    );

    std::fs::remove_file(&tmp).ok();
}

#[test]
fn json_roundtrip_preserves_critical_path() {
    let mut net = load_network_from_path(&example_path()).expect("load example");
    let dp_before = net.solve(None).expect("solve original");

    // Same round-trip through the JSON serializer.
    let json = save_network_to_json_string(&net).expect("serialize JSON");
    let reloaded = venti::load_network_from_str(&json).expect("load JSON string");
    assert_eq!(
        signature(&net),
        signature(&reloaded),
        "networks must be equal"
    );

    let mut reloaded = reloaded;
    let dp_after = reloaded.solve(None).expect("solve reloaded JSON");
    assert!(
        (dp_after - dp_before).abs() < 1e-9,
        "JSON round-trip changed critical-path ΔP: {dp_before} -> {dp_after}"
    );
    assert!(matches_anchor(dp_after));
}
