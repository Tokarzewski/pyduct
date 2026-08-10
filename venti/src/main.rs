//! `venti` command-line interface.
//!
//! Loads a ductwork network from YAML/JSON (the same format `wenta solve`
//! accepts) via the library `venti::io` module, then solves / reports / info /
//! validates.

use std::path::PathBuf;

use clap::{Parser, Subcommand};

use venti::core::fluid::Fluid;
use venti::network::ComponentEnum;

#[derive(Parser)]
#[command(
    name = "venti",
    version,
    about = "Ductwork design — sizing, pressure-drop, network solving (Rust port of wenta)"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Solve a network file and print results.
    Solve {
        file: PathBuf,
        /// Output format: text | markdown | json | csv
        #[arg(long, default_value = "text")]
        format: String,
    },
    /// Extract a per-component results table / schedule (FR-16).
    Report {
        file: PathBuf,
        /// Output format: text | json | csv
        #[arg(long, default_value = "text")]
        format: String,
    },
    /// Print structural summary, no solve.
    Info { file: PathBuf },
    /// Validate a network file structurally.
    Validate { file: PathBuf },
}

fn standard_fluid() -> Result<Fluid, &'static str> {
    Fluid::new(1.204, 1.825e-5)
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Commands::Solve { file, format } => cmd_solve(&file, &format),
        Commands::Report { file, format } => cmd_report(&file, &format),
        Commands::Info { file } => cmd_info(&file),
        Commands::Validate { file } => cmd_validate(&file),
    };
    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn cmd_solve(file: &std::path::Path, format: &str) -> Result<(), String> {
    let mut net = venti::load_network_from_path(file)?;
    let dp = net.solve(Some(&standard_fluid()?))?;

    let mut rows: Vec<(String, String, f64, f64)> = Vec::new();
    for (cid, comp) in net.iter_components() {
        let c = comp.as_component();
        let kind = type_name(comp);
        let mut max_drop = 0.0f64;
        let mut max_v = 0.0f64;
        for p in c.ports() {
            max_drop = max_drop.max(p.pressure_drop);
            max_v = max_v.max(p.velocity);
        }
        rows.push((cid.clone(), kind, max_v, max_drop));
    }

    let path = venti::critical_path(&net)?;

    match format {
        "json" => {
            let obj = serde_json::json!({
                "name": net.name,
                "critical_path_pressure_drop_pa": dp,
                "critical_path": path,
                "components": rows.iter().map(|(cid, kind, v, d)| serde_json::json!({
                    "id": cid, "type": kind, "velocity_m_s": v, "pressure_drop_pa": d
                })).collect::<Vec<_>>(),
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&obj).map_err(|e| e.to_string())?
            );
        }
        "csv" => {
            println!("id,type,velocity_m_s,pressure_drop_pa");
            for (cid, kind, v, d) in &rows {
                println!("{cid},{kind},{v},{d}");
            }
            println!("critical_path_pressure_drop_pa,{dp}");
        }
        "markdown" => {
            println!("## {} — solve", net.name);
            println!();
            println!("| id | type | velocity [m/s] | dp [Pa] |");
            println!("|---|---|---|---|");
            for (cid, kind, v, d) in &rows {
                println!("| {cid} | {kind} | {v:.3} | {d:.3} |");
            }
            println!();
            println!("**Critical-path ΔP: {dp:.2} Pa**");
            println!();
            println!("**Critical path:** {}", path.join(" → "));
        }
        _ => {
            println!("{}", net.name);
            println!(
                "{:<16} {:<16} {:>12} {:>12}",
                "id", "type", "v [m/s]", "dp [Pa]"
            );
            for (cid, kind, v, d) in &rows {
                println!("{cid:<16} {kind:<16} {v:>12.3} {d:>12.3}");
            }
            println!();
            println!("Critical-path pressure drop: {dp:.2} Pa");
            println!("Critical path: {}", path.join(" → "));
        }
    }
    Ok(())
}

fn cmd_report(file: &std::path::Path, format: &str) -> Result<(), String> {
    let mut net = venti::load_network_from_path(file)?;
    net.solve(Some(&standard_fluid()?))?;

    match format {
        "json" => println!(
            "{}",
            serde_json::to_string_pretty(&venti::results_as_json_rows(&net))
                .map_err(|e| e.to_string())?
        ),
        "csv" => println!("{}", venti::results_as_csv(&net, ',')),
        _ => println!("{}", venti::results_summary(&net)),
    }
    Ok(())
}

fn cmd_info(file: &std::path::Path) -> Result<(), String> {
    let net = venti::load_network_from_path(file)?;
    let terminals = net.terminals().len();
    let sources = net.sources().len();
    let total_flow: f64 = net.terminals().iter().map(|t| t.flowrate_demand).sum();
    println!("name: {}", net.name);
    println!("components: {}", net.len());
    println!("sources: {sources}");
    println!("terminals: {terminals}");
    println!("connections: {}", net.connection_count());
    println!("total terminal flowrate [m3/s]: {total_flow:.4}");
    Ok(())
}

fn cmd_validate(file: &std::path::Path) -> Result<(), String> {
    let net = venti::load_network_from_path(file)?;
    let problems = net.validate();
    if problems.is_empty() {
        println!("OK: network is structurally valid.");
    } else {
        println!("{} problem(s):", problems.len());
        for p in &problems {
            println!("  - {p}");
        }
    }
    Ok(())
}

fn type_name(comp: &ComponentEnum) -> String {
    comp.kind().to_string()
}
