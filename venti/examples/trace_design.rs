//! Host-agnostic topology example (M3 core): trace 2D polylines into a `Network`,
//! solve, and project the result back to drawable segments. No CAD required —
//! this is the geometry core a ZWCAD adapter will reuse.
//! Run: `cargo run --example trace_design`.

use std::collections::HashMap;

use venti::core::fluid::Fluid;
use venti::topology::{Polyline, TraceOptions};

fn main() {
    // A small tee layout (metres): a 5 m trunk feeding two 3 m branches.
    let polylines = vec![
        Polyline::new(vec![(0.0, 0.0), (2.5, 0.0), (5.0, 0.0)]),
        Polyline::new(vec![(5.0, 0.0), (8.0, 0.0)]),
        Polyline::new(vec![(5.0, 0.0), (5.0, -3.0)]),
    ];
    let flows = HashMap::from([("term0".to_string(), 0.06), ("term1".to_string(), 0.04)]);
    let diameters = HashMap::from([
        ("duct0".to_string(), 0.25), // trunk
        ("duct1".to_string(), 0.15), // branch 1
        ("duct2".to_string(), 0.15), // branch 2
    ]);
    let opts = TraceOptions {
        flows,
        diameters,
        ..Default::default()
    };

    let mut sys = venti::topology::trace(&polylines, &opts).unwrap();
    let fluid = Fluid::new(1.204, 1.825e-5).unwrap();
    let dp = sys.network.solve(Some(&fluid)).unwrap();

    println!(
        "Traced {} ducts ({} m total), critical-path ΔP = {:.2} Pa",
        sys.chains.len(),
        sys.total_length_m(),
        dp
    );

    println!("\nDraw primitives (flatten):");
    for seg in sys.flatten() {
        println!(
            "  {}  {:.3} m ∅  ({:.2},{:.2}) → ({:.2},{:.2})",
            seg.component_id, seg.diameter, seg.start.0, seg.start.1, seg.end.0, seg.end.1
        );
    }

    println!("\nSchedule (solved):");
    for (id, comp) in sys.network.iter_components() {
        let c = comp.as_component();
        let max_dp = c
            .ports()
            .iter()
            .map(|p| p.pressure_drop)
            .fold(0.0, f64::max);
        let kind = type_name(comp);
        println!("  {id:<8} {kind:<10} dp = {max_dp:8.3} Pa");
    }
}

fn type_name(comp: &venti::network::ComponentEnum) -> &'static str {
    use venti::network::ComponentEnum::{
        FlexDuct, RigidDuct, Source, Tee, Terminal, TwoPortFitting,
    };
    match comp {
        Source(_) => "Source",
        Terminal(_) => "Terminal",
        RigidDuct(_) => "RigidDuct",
        FlexDuct(_) => "FlexDuct",
        TwoPortFitting(_) => "TwoPortFitting",
        Tee(_) => "Tee",
    }
}
