//! `venti` command-line interface.
//!
//! Loads a ductwork network from YAML/JSON (the same format `wenta solve`
//! accepts) via the library `venti::io` module, then solves / reports / info /
//! validates.

use std::path::PathBuf;
use venti::Result;

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
    /// Dump the ζ database (built-in reference, optionally merged with a
    /// vendor catalogue JSON).
    Catalog {
        /// Optional vendor catalogue JSON to merge in.
        #[arg(long)]
        vendor: Option<PathBuf>,
        /// Output format: text | csv | json
        #[arg(long, default_value = "text")]
        format: String,
    },
    /// Load a network file and write it back out (wenta YAML/JSON round-trip).
    Save {
        file: PathBuf,
        /// Output path (default: `<input>.saved.json`)
        #[arg(long, short, default_value = "")]
        out: String,
    },
    /// Solve a network and print the bill of materials (total length/area + parts).
    Bom {
        file: PathBuf,
        /// Output format: text | csv | json
        #[arg(long, default_value = "text")]
        format: String,
    },
    /// Detect clashes on a small fixture of two crossing duct centreline polylines.
    Clash {},
    /// Print the project default settings (ProjectSettings::default()).
    Settings {
        /// Output format: json | text
        #[arg(long, default_value = "json")]
        format: String,
    },
}

fn standard_fluid() -> Result<Fluid> {
    Fluid::new(1.204, 1.825e-5)
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Commands::Solve { file, format } => cmd_solve(&file, &format),
        Commands::Report { file, format } => cmd_report(&file, &format),
        Commands::Info { file } => cmd_info(&file),
        Commands::Validate { file } => cmd_validate(&file),
        Commands::Catalog { vendor, format } => cmd_catalog(vendor.as_deref(), &format),
        Commands::Save { file, out } => cmd_save(&file, &out),
        Commands::Bom { file, format } => cmd_bom(&file, &format),
        Commands::Clash {} => cmd_clash(),
        Commands::Settings { format } => cmd_settings(&format),
    };
    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn cmd_solve(file: &std::path::Path, format: &str) -> Result<()> {
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

fn cmd_report(file: &std::path::Path, format: &str) -> Result<()> {
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

fn cmd_info(file: &std::path::Path) -> Result<()> {
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

fn cmd_validate(file: &std::path::Path) -> Result<()> {
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

fn cmd_save(file: &std::path::Path, out: &str) -> Result<()> {
    let net = venti::load_network_from_path(file)?;
    let out_path = if out.is_empty() {
        PathBuf::from(format!("{}.saved.json", file.display()))
    } else {
        PathBuf::from(out)
    };
    venti::save_network_to_path(&net, &out_path)?;
    let kind = if out_path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase() == "json")
        .unwrap_or(false)
    {
        "JSON"
    } else {
        "YAML"
    };
    println!(
        "saved {}=\u{203a} {} ({kind})",
        file.display(),
        out_path.display()
    );
    Ok(())
}

fn cmd_catalog(vendor: Option<&std::path::Path>, format: &str) -> Result<()> {
    let mut cat = venti::reference_catalog();
    if let Some(vpath) = vendor {
        let vc = venti::vendor_catalog_from_file(vpath.to_str().ok_or("invalid vendor path")?)?;
        cat.merge(&vc.to_catalog());
    }
    match format {
        "json" => {
            let rows: Vec<serde_json::Value> = cat
                .iter()
                .map(|e| {
                    serde_json::json!({
                        "key": e.key, "name": e.name,
                        "category": format!("{:?}", e.category),
                        "zeta": e.zeta,
                        "reference_velocity": format!("{:?}", e.reference_velocity),
                        "source": e.source, "size_mm": e.size_mm,
                    })
                })
                .collect();
            println!(
                "{}",
                serde_json::to_string_pretty(&rows).map_err(|e| e.to_string())?
            );
        }
        "csv" => {
            println!("key,category,zeta,reference_velocity,source,size_mm");
            for e in cat.iter() {
                let sz = e.size_mm.map(|x| x.to_string()).unwrap_or_default();
                println!(
                    "{},{:?},{},{:?},{},{}",
                    e.key, e.category, e.zeta, e.reference_velocity, e.source, sz
                );
            }
        }
        _ => {
            println!(
                "{:<28} {:<9} {:>6} {:<9} {:<18} size",
                "key", "cat", "zeta", "ref", "source"
            );
            for e in cat.iter() {
                let sz = e.size_mm.map(|x| format!("{x} mm")).unwrap_or_default();
                println!(
                    "{:<28} {:<9} {:>6.2} {:<9} {:<18} {}",
                    e.key,
                    format!("{:?}", e.category),
                    e.zeta,
                    format!("{:?}", e.reference_velocity),
                    e.source,
                    sz
                );
            }
            println!();
            println!("{} entries in database.", cat.len());
        }
    }
    Ok(())
}

fn cmd_bom(file: &std::path::Path, format: &str) -> Result<()> {
    let mut net = venti::load_network_from_path(file)?;
    net.solve(Some(&standard_fluid()?))?;

    let items = venti::build_bom(&net)?;
    let total_len = venti::total_length(&items);
    let total_area = venti::total_area(&items);

    match format {
        "json" => {
            let obj = serde_json::json!({
                "name": net.name,
                "total_length_m": total_len,
                "total_area_m2": total_area,
                "items": items.iter().map(|i| serde_json::json!({
                    "component_id": i.component_id,
                    "kind": i.kind,
                    "length_m": i.length_m,
                    "area_m2": i.area_m2,
                })).collect::<Vec<_>>(),
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&obj).map_err(|e| e.to_string())?
            );
        }
        "csv" => {
            println!("{}", venti::bom_as_csv(&items));
            println!("total_length_m,{total_len}");
            println!("total_area_m2,{total_area}");
        }
        _ => {
            println!("{}", net.name);
            println!(
                "{:<16} {:<14} {:>12} {:>12}",
                "component_id", "kind", "length [m]", "area [m2]"
            );
            for i in &items {
                println!(
                    "{:<16} {:<14} {:>12.3} {:>12.3}",
                    i.component_id, i.kind, i.length_m, i.area_m2
                );
            }
            println!();
            println!("Total length: {total_len:.3} m");
            println!("Total area:   {total_area:.3} m^2");
        }
    }
    Ok(())
}

fn cmd_clash() -> Result<()> {
    // Build a small fixture of TWO CROSSING duct centreline runs using
    // venti::topology::trace. The trunk D0 runs along y = 0; a branch run D4
    // drops from the upper junction and crosses the trunk at (2, 0). The extra
    // polylines merely form the tees that keep the tree connected (trace only
    // admits trees; a bare X would be a degree-4 junction and is rejected).
    let polylines = vec![
        venti::topology::Polyline::new(vec![(0.0, 0.0), (4.0, 0.0)]),
        venti::topology::Polyline::new(vec![(4.0, 0.0), (2.0, 2.0)]),
        venti::topology::Polyline::new(vec![(4.0, 0.0), (6.0, 2.0)]),
        venti::topology::Polyline::new(vec![(2.0, 2.0), (-2.0, 2.0)]),
        venti::topology::Polyline::new(vec![(2.0, -2.0), (2.0, 2.0)]),
    ];
    let sys = venti::topology::trace(&polylines, &venti::topology::TraceOptions::default())?;
    let segments = sys.flatten();
    let clearance_m = 0.05;
    let clashes = venti::clash::find_clashes(&segments, clearance_m)?;

    println!(
        "Fixture: two crossing centreline runs (∅ 0.200 m), clearance {:.3} m",
        clearance_m
    );
    for s in &segments {
        println!(
            "  {}  ({:.1},{:.1}) → ({:.1},{:.1})",
            s.component_id, s.start.0, s.start.1, s.end.0, s.end.1
        );
    }
    if clashes.is_empty() {
        println!("No clashes within {:.3} m clearance.", clearance_m);
    } else {
        println!(
            "{} clash(es) within {:.3} m clearance:",
            clashes.len(),
            clearance_m
        );
        for c in &clashes {
            println!(
                "  {} <-> {}   centreline distance {:.4} m",
                c.a, c.b, c.distance_m
            );
        }
    }
    Ok(())
}

fn cmd_settings(format: &str) -> Result<()> {
    let s = venti::ProjectSettings::default();
    match format {
        "json" => println!("{}", venti::settings_to_json(&s)?),
        _ => {
            println!("ProjectSettings (defaults)");
            println!("  standard:              {:?}", s.standard);
            println!("  default_diameter_mm:   {}", s.default_diameter_mm);
            println!("  absolute_roughness_m:  {}", s.absolute_roughness_m);
            println!("  target_velocity_ms:    {}", s.target_velocity_ms);
            println!("  target_pa_per_m:       {}", s.target_pa_per_m);
            println!("  noise_space:           {}", s.noise_space);
            println!("  units:                 {:?}", s.units);
        }
    }
    Ok(())
}

fn type_name(comp: &ComponentEnum) -> String {
    comp.kind().to_string()
}
