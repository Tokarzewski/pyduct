//! Dependency-free micro-benchmark of the `venti` kernels.
//!
//! Times each kernel with `std::time::Instant` over many iterations and prints
//! a table (kernel, n, total_ms, per_call_s, calls_per_sec). Results are kept
//! alive with `std::hint::black_box` so the compiler cannot optimise away the
//! work. There is no stable `#[bench]`/`criterion` dependency — this is just a
//! binary you run directly:
//!
//! ```sh
//! cargo run --release --bin bench
//! ```
//!
//! The Python/Mojo reference rows (left column) in the README table come from
//! the existing `wentamojo/benchmarks/bench_suite.mojo` table in the `pyduct`
//! repository README (the "Mojo kernel speedup vs Python reference" table).

use std::hint::black_box;
use std::time::Instant;

use venti::core::fluid::STANDARD_AIR;
use venti::core::geometry::Round;
use venti::physics::friction::{friction_factor, reynolds};
use venti::physics::losses::{local_pressure_drop, straight_pressure_drop};
use venti::sizing::{
    aspect_ratio_method, equal_friction_method_round, velocity_method_batch, velocity_method_round,
};
use venti::{ComponentEnum, Network, RigidDuct, Source, Tee, Terminal};

/// Row in the output table.
struct BenchResult {
    kernel: &'static str,
    n: u64,
    total_ms: f64,
}

/// Time `f()` over `n` iterations, black-boxing the output so nothing is
/// optimised away. Returns total elapsed time in milliseconds.
fn time_it<F: FnMut() -> O, O>(n: u64, mut f: F) -> f64 {
    // Warm up the CPU / caches / branch predictors first.
    let mut warm = black_box(f());
    for _ in 0..10_000 {
        warm = black_box(f());
    }
    black_box(&warm);

    let start = Instant::now();
    for _ in 0..n {
        black_box(f());
    }
    start.elapsed().as_secs_f64() * 1000.0
}

fn print_table(lang: &str, header: &str, rows: &[BenchResult]) {
    println!("\n== {lang} == {header}");
    println!(
        "{:<34} {:>12} {:>12} {:>14} {:>16}",
        "kernel", "n", "total_ms", "per_call_s", "calls_per_sec"
    );
    for r in rows {
        let per_call_s = (r.total_ms / 1000.0) / r.n as f64;
        let calls_per_sec = 1.0 / per_call_s;
        println!(
            "{:<34} {:>12} {:>12.3} {:>14.3e} {:>16.1}",
            r.kernel, r.n, r.total_ms, per_call_s, calls_per_sec
        );
    }
}

/// Assemble the 3-zone supply network from `examples/network_yaml.yaml` so the
/// build+solve benchmark exercises a realistic topology.
fn build_three_zone() -> Network {
    let mut net = Network::new("Example 3-Zone Supply Network");

    net.add("ahu", ComponentEnum::Source(Source::new("AHU")))
        .unwrap();

    // main trunk: round 0.25 m
    let main = Round::new(0.25).unwrap();
    net.add(
        "main_duct",
        ComponentEnum::RigidDuct(
            RigidDuct::new(
                "Main Trunk",
                main.area,
                main.hydraulic_diameter,
                25.0,
                0.0001,
            )
            .unwrap(),
        ),
    )
    .unwrap();

    net.add(
        "main_tee",
        ComponentEnum::Tee(Tee::new("Main Tee", main.area, 0.0, 0.5)),
    )
    .unwrap();

    // branch 1: round 0.15 m
    let b1 = Round::new(0.15).unwrap();
    net.add(
        "branch1_duct",
        ComponentEnum::RigidDuct(
            RigidDuct::new(
                "Branch 1 Duct",
                b1.area,
                b1.hydraulic_diameter,
                20.0,
                0.0001,
            )
            .unwrap(),
        ),
    )
    .unwrap();

    // branch 2: round 0.12 m
    let b2 = Round::new(0.12).unwrap();
    net.add(
        "branch2_duct",
        ComponentEnum::RigidDuct(
            RigidDuct::new(
                "Branch 2 Duct",
                b2.area,
                b2.hydraulic_diameter,
                15.0,
                0.0001,
            )
            .unwrap(),
        ),
    )
    .unwrap();

    net.add(
        "zone1_terminal",
        ComponentEnum::Terminal(Terminal::new("Zone 1 Diffuser", 0.10, Some(b1.area), 0.3)),
    )
    .unwrap();
    net.add(
        "zone2_terminal",
        ComponentEnum::Terminal(Terminal::new("Zone 2 Diffuser", 0.07, Some(b2.area), 0.3)),
    )
    .unwrap();

    net.connect("ahu", "main_duct").unwrap();
    net.connect("main_duct", "main_tee.combined").unwrap();
    net.connect("main_tee.straight", "branch1_duct").unwrap();
    net.connect("branch1_duct", "zone1_terminal").unwrap();
    net.connect("main_tee.branch", "branch2_duct").unwrap();
    net.connect("branch2_duct", "zone2_terminal").unwrap();

    net
}

fn main() {
    let fluid = STANDARD_AIR;

    // Representative inputs (typical mid-range duct values).
    let re = 50_000.0;
    let eps = 0.0009;
    let v = 4.0;
    let diameter = 0.2;
    let length = 10.0;
    let q = 0.1;

    let mut rows: Vec<BenchResult> = Vec::new();

    // friction_factor — 1e6 calls.
    rows.push(BenchResult {
        kernel: "friction_factor",
        n: 1_000_000,
        total_ms: time_it(1_000_000, || friction_factor(black_box(re), black_box(eps))),
    });

    // reynolds — 1e6 calls.
    rows.push(BenchResult {
        kernel: "reynolds",
        n: 1_000_000,
        total_ms: time_it(1_000_000, || {
            reynolds(black_box(v), black_box(diameter), black_box(1.5e-5))
        }),
    });

    // local_pressure_drop — 1e6 calls.
    rows.push(BenchResult {
        kernel: "local_pressure_drop",
        n: 1_000_000,
        total_ms: time_it(1_000_000, || {
            local_pressure_drop(black_box(1.0), black_box(v), black_box(fluid.density))
        }),
    });

    // straight_pressure_drop — 1e6 calls.
    rows.push(BenchResult {
        kernel: "straight_pressure_drop",
        n: 1_000_000,
        total_ms: time_it(1_000_000, || {
            straight_pressure_drop(
                black_box(0.02),
                black_box(length),
                black_box(diameter),
                black_box(v),
                black_box(fluid.density),
            )
        }),
    });

    // velocity_method_round — 5e4 calls.
    rows.push(BenchResult {
        kernel: "velocity_method_round",
        n: 50_000,
        total_ms: time_it(50_000, || {
            black_box(velocity_method_round(black_box(q), black_box(v)))
        }),
    });

    // equal_friction_method_round — 5e4 calls.
    rows.push(BenchResult {
        kernel: "equal_friction_method_round",
        n: 50_000,
        total_ms: time_it(50_000, || {
            black_box(equal_friction_method_round(
                black_box(q),
                black_box(1.0),
                black_box(0.0001),
                &fluid,
            ))
        }),
    });

    // aspect_ratio_method — 5e4 calls.
    rows.push(BenchResult {
        kernel: "aspect_ratio_method",
        n: 50_000,
        total_ms: time_it(50_000, || {
            black_box(aspect_ratio_method(
                black_box(0.08),
                black_box(v),
                black_box(2.0),
            ))
        }),
    });

    // batch velocity sizing — 1000 batches x 200 ducts = 200_000 sizings.
    let flows: Vec<f64> = (0..200).map(|i| 0.005 + 0.001 * i as f64 * 0.1).collect();
    rows.push(BenchResult {
        kernel: "velocity_method_batch (200 ducts)",
        n: 1_000,
        total_ms: time_it(1_000, || {
            black_box(velocity_method_batch(flows.iter(), black_box(v)))
        }),
    });

    // network build + solve (3-zone) — 1000 times.
    rows.push(BenchResult {
        kernel: "network build+solve (3-zone)",
        n: 1_000,
        total_ms: time_it(1_000, || {
            let mut net = build_three_zone();
            black_box(net.solve(Some(&fluid)).unwrap())
        }),
    });

    print_table("venti (Rust)", "kernel micro-benchmarks", &rows);
}
