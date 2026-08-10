//! End-to-end DESIGN-WORKFLOW demonstration of every core `venti` capability.
//!
//! This single executable walks the full HVAC duct-design workflow in order:
//! size a duct, build + solve a network, extract a component schedule, round-trip
//! the network to JSON, check duct-generated noise against an NC target, size a
//! balancing damper, and query the fittings library — all with no external crates.
//!
//! Run with:
//!
//! ```text
//! cargo run --example design_workflow
//! ```
//!
//! Everything here is deterministic (fixed input values, standard air) and the
//! program never panics: a `Result`-returning `run()` is used and any error is
//! printed gracefully in `main`.

use venti::{
    balancing_zeta, cross_fitting, damper_open_percentage, duct_pressure_level, extract_results,
    fire_damper, nc_ok, regenerated_noise_round, required_zeta, save_network_to_json_string,
    velocity_method_round, ComponentEnum, CrossSection, Network, RigidDuct, Source, Terminal,
    TwoPortFitting, STANDARD_AIR,
};

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    println!("══════════════════════════════════════════════════════════════");
    println!("  venti — end-to-end ductwork design workflow");
    println!("══════════════════════════════════════════════════════════════");

    // -----------------------------------------------------------------
    // 1. Size a supply duct for 0.1 m³/s at 4 m/s (velocity method).
    // -----------------------------------------------------------------
    println!("\n[1] Duct sizing (velocity method, round)");
    println!("    Design flow: 0.1 m³/s @ target 4 m/s");
    let (section, v) = velocity_method_round(0.1, 4.0)?;
    let diameter = match section {
        CrossSection::Round(r) => r.diameter,
        other => other.width(),
    };
    println!(
        "    Sized duct  →  D = {:.0} mm, actual velocity = {:.2} m/s",
        diameter * 1000.0,
        v
    );

    // -----------------------------------------------------------------
    // 2. Build + solve a small network.
    //    Source -> RigidDuct(D=0.2, L=20) -> TwoPortFitting(ζ=0.5)
    //             -> Terminal(0.1)
    // -----------------------------------------------------------------
    println!("\n[2] Network build + solve");
    let round = match section {
        CrossSection::Round(r) => r,
        _ => unreachable!("velocity_method_round returns a round section"),
    };
    let mut net = Network::new("design_example");
    net.add("ahu", ComponentEnum::Source(Source::new("AHU")))?;
    net.add(
        "duct",
        ComponentEnum::RigidDuct(RigidDuct::new(
            "main duct",
            round.area,
            round.hydraulic_diameter,
            20.0,
            0.0001,
        )?),
    )?;
    net.add(
        "fit",
        ComponentEnum::TwoPortFitting(TwoPortFitting::new("elbow", round.area, 0.5)),
    )?;
    net.add(
        "term",
        ComponentEnum::Terminal(Terminal::new("diffuser", 0.1, Some(round.area), 1.0)),
    )?;
    net.connect("ahu", "duct")?;
    net.connect("duct", "fit")?;
    net.connect("fit", "term")?;
    let dp_pa = net.solve(Some(&STANDARD_AIR))?;
    println!("    Topology: Source → RigidDuct(0.2 m, 20 m) → Fitting(ζ=0.5) → Terminal(0.1 m³/s)");
    println!("    Critical-path ΔP = {:.2} Pa   ({dp_pa} Pa raw)", dp_pa);

    // -----------------------------------------------------------------
    // 3. Extract a per-component schedule.
    // -----------------------------------------------------------------
    println!("\n[3] Component schedule (venti::extract_results)");
    println!(
        "    {:<10} {:<16} {:<14} {:>8} {:>8} {:>9}",
        "ID", "Type", "Name", "Q_in", "V_in", "ΔP [Pa]"
    );
    for r in extract_results(&net) {
        let q = r.flowrate_in.unwrap_or(0.0);
        let v = r.velocity_in.unwrap_or(0.0);
        println!(
            "    {:<10} {:<16} {:<14} {:>8.3} {:>8.2} {:>9.2}",
            r.component_id, r.component_type, r.name, q, v, r.pressure_drop
        );
    }

    // -----------------------------------------------------------------
    // 4. Save the network to JSON and print it.
    // -----------------------------------------------------------------
    println!("\n[4] Network serialization (venti::save_network_to_json_string)");
    let json = save_network_to_json_string(&net)?;
    for line in json.lines() {
        println!("    {line}");
    }

    // -----------------------------------------------------------------
    // 5. Duct noise + NC compliance for the sized duct.
    // -----------------------------------------------------------------
    println!("\n[5] Acoustics — regenerated noise + NC compliance");
    // Room: a medium open-plan office.
    let room_area = 120.0; // total internal surface area [m²]
    let absorption = 0.25; // average Sabine absorption coefficient
    let lw = regenerated_noise_round(v, diameter, None)?;
    println!(
        "    Regenerated sound power (D={:.0} mm, v={:.2} m/s): {:.1} dB re 1e-12 W",
        diameter * 1000.0,
        v,
        lw
    );
    let lp = duct_pressure_level(lw, room_area, absorption)?;
    println!("    Room sound pressure level (S={room_area} m², α={absorption}): {lp:.1} dB");
    let office_nc = 35.0;
    let compliant = nc_ok("office", lp)?;
    println!(
        "    NC target (office, NC{office_nc:.0}): {}",
        if compliant { "PASS ✓" } else { "FAIL ✗" }
    );

    // -----------------------------------------------------------------
    // 6. Balancing — damper ζ and open % a branch would need.
    // -----------------------------------------------------------------
    println!("\n[6] Balancing");
    let velocity = 4.0; // mean velocity in the oversized branch [m/s]
    let density = STANDARD_AIR.density;
    let total_req = 30.0; // pressure the terminal needs [Pa]
    let available = 10.736; // pressure actually available at the branch [Pa]
    let z_req = required_zeta(total_req - available, velocity, density);
    let z_bal = balancing_zeta(total_req, available, velocity, density);
    let open = damper_open_percentage(z_bal);
    println!(
        "    Surplus pressure to absorb: {:.2} Pa",
        total_req - available
    );
    println!("    Required damper ζ = {z_req:.2}   (balancing_zeta = {z_bal:.2})");
    println!(
        "    Damper setting     = {:.1}% open  (fully open at 100%)",
        open
    );

    // -----------------------------------------------------------------
    // 7. Fittings library — show the loss coefficient of a new fitting.
    // -----------------------------------------------------------------
    println!("\n[7] Fittings library");
    let fd_zeta = fire_damper(100.0)?; // fully-open fire damper
    let (zm, zb) = cross_fitting(0.3, 0.2, 0.4)?;
    println!("    fire_damper(100%)      → ζ = {fd_zeta:.3}   (fully-open section loss)");
    println!("    cross_fitting(0.3,0.2,0.4) → ζ_main = {zm:.3}, ζ_branch = {zb:.3}");

    // Also demonstrate the top-level sizing APIs touched along the way.
    let (rect_section, rect_v) = venti::velocity_method_rectangular(0.1, 4.0)?;
    let c = match rect_section {
        CrossSection::Rectangular(r) => format!(
            "{:.0}×{:.0} mm (v={rect_v:.2} m/s)",
            r.width * 1000.0,
            r.height * 1000.0
        ),
        _ => String::new(),
    };
    println!("\n    Bonus — rectangular velocity-method sizing: {c}");

    println!("\n══════════════════════════════════════════════════════════════");
    println!("  Design workflow complete ✓");
    println!("══════════════════════════════════════════════════════════════");
    Ok(())
}
