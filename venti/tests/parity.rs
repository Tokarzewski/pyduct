//! Parity tests: `venti` against the `wenta` Python reference.
//!
//! Two kinds of checks live here (SPEC NFR "parity/trust", roadmap #9):
//!
//! 1. **Golden values** — hard-coded numbers captured from the reference
//!    (`wenta`/`wentamojo`) for the shipped example network. These always run
//!    and guard against regressions.
//! 2. **Oracle differential** — if the environment variable
//!    `VENTI_PYTHON_ORACLE` points at a runnable `wenta` CLI (i.e. the Python+
//!    Mojo reference is available), `venti report --format json` output is
//!    diffed against the oracle's own JSON report field-by-field. Skipped when
//!    the oracle is not present so CI without Mojo still passes.

use std::path::Path;

use venti::core::fluid::Fluid;

fn example_network() -> venti::Network {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/network_yaml.yaml");
    venti::load_network_from_path(&path).expect("example network loads")
}

#[test]
fn golden_critical_path_pressure_drop() {
    let mut net = example_network();
    let fluid = Fluid::new(1.204, 1.825e-5).unwrap();
    let dp = net.solve(Some(&fluid)).unwrap();
    // Reference value from the wenta example solve.
    assert!((dp - 84.06320865255796).abs() < 1e-6, "dp = {dp}");
}

#[test]
fn golden_component_pressure_drops() {
    let mut net = example_network();
    let fluid = Fluid::new(1.204, 1.825e-5).unwrap();
    net.solve(Some(&fluid)).unwrap();

    let results = venti::extract_results(&net);
    let by_id: std::collections::HashMap<&str, &venti::ComponentResult> = results
        .iter()
        .map(|r| (r.component_id.as_str(), r))
        .collect();

    // Reference values from the wenta example solve.
    let expectations = [
        ("main_duct", 15.63444713910372f64),
        ("branch1_duct", 58.026f64),
        ("branch2_duct", 67.8166609515175),
        ("main_tee", 0.6121005619367282),
    ];
    for (id, expected_dp) in expectations {
        let res = by_id[id];
        assert!(
            (res.pressure_drop - expected_dp).abs() < 1e-3,
            "{id}: pressure_drop = {} (expected ~{expected_dp})",
            res.pressure_drop
        );
    }

    // Flows propagate: the source sees the total terminal demand.
    let source = by_id["ahu"];
    assert!((source.flowrate_out.unwrap() - 0.17).abs() < 1e-9);
}

#[test]
fn golden_velocity_method() {
    // velocity_method_round(0.1, 4.0) -> 200 mm duct, v = 0.1/area.
    let (section, v) = venti::velocity_method_round(0.1, 4.0).unwrap();
    let expected = 0.1 / (std::f64::consts::PI * 0.01);
    assert!((v - expected).abs() < 1e-9);
    let _ = section;
}

#[test]
fn golden_friction_factor() {
    // Swamee–Jain at Re=5e4, eps/D=9e-4 (reference).
    let f = venti::physics::friction::friction_factor(50_000.0, 0.0009);
    assert!((f - 0.023644603810775315).abs() < 1e-12);
}

#[cfg(feature = "cli")]
#[test]
fn oracle_differential_when_available() {
    use std::process::Command;

    let oracle = match std::env::var("VENTI_PYTHON_ORACLE") {
        Ok(p) if !p.is_empty() => p,
        _ => {
            eprintln!("skipping oracle differential (VENTI_PYTHON_ORACLE not set)");
            return;
        }
    };

    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/network_yaml.yaml");
    let path = path.to_str().unwrap();

    // venti JSON report
    let mut net = example_network();
    let fluid = Fluid::new(1.204, 1.825e-5).unwrap();
    net.solve(Some(&fluid)).unwrap();
    let mine = venti::report_json_string(&net);

    // oracle JSON report (wenta report --format json)
    let out = Command::new(&oracle)
        .args(["report", path, "--format", "json"])
        .output()
        .expect("run oracle");
    assert!(
        out.status.success(),
        "oracle failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let theirs = String::from_utf8_lossy(&out.stdout).to_string();

    let mine: serde_json::Value = serde_json::from_str(&mine).unwrap();
    let theirs: serde_json::Value = serde_json::from_str(&theirs).unwrap();

    let mine_rows = mine.as_array().unwrap();
    let theirs_rows = theirs.as_array().unwrap();
    assert_eq!(mine_rows.len(), theirs_rows.len(), "row count differs");

    for their_row in theirs_rows {
        let id = their_row["component_id"].as_str().unwrap();
        let mine_row = find_row(mine_rows, id).unwrap_or_else(|| panic!("missing {id} in venti"));
        for key in ["component_type", "pressure_drop"] {
            if key == "component_type" {
                assert_eq!(mine_row[key], their_row[key], "{id}.{key}");
            } else {
                let a = mine_row[key].as_f64().unwrap();
                let b = their_row[key].as_f64().unwrap();
                assert!((a - b).abs() < 1e-6, "{id}.{key}: {a} vs {b}");
            }
        }
    }
}

#[cfg(feature = "cli")]
fn find_row<'a>(rows: &'a [serde_json::Value], id: &str) -> Option<&'a serde_json::Value> {
    rows.iter().find(|r| r["component_id"] == id)
}
