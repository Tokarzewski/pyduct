# venti — ductwork design guide

`venti` is a Rust library for HVAC duct design: sizing, pressure-drop and
fitting-loss calculations, network solving, insulation, fan selection, room air
balance, clash detection and reporting. All quantities are SI — flow in
**m³/s**, pressure in **Pa**, dimensions in **m**, power in **W** — with units
converters re-exported at the crate root. Snippets import from the crate root
unless a module is named (e.g. `venti::insulation`, `venti::clash`, `venti::components::fittings_library`);
the fully worked example is in `examples/design_workflow.rs`.

---

## 1. Build and solve a network

A `Network` is a directed graph of `Source` (fan/AHU), `RigidDuct`/`FlexDuct`,
fittings (`TwoPortFitting`, `Tee`) and `Terminal` (diffuser/grille/cap).
Terminal demands propagate upstream, per-component ΔP is computed and `solve`
returns the **critical-path** pressure drop (longest weighted source→terminal
path).
```rust
let r = Round::new(0.2).unwrap();                       // 200 mm round duct
let mut net = Network::new("office");
net.add("ahu", ComponentEnum::Source(Source::new("AHU"))).unwrap();
net.add("duct", ComponentEnum::RigidDuct(RigidDuct::new("main", r.area, r.hydraulic_diameter, 20.0, 0.0001).unwrap())).unwrap();
net.add("term", ComponentEnum::Terminal(Terminal::new("diffuser", 0.1, Some(r.area), 1.0))).unwrap();
net.connect("ahu", "duct").unwrap(); net.connect("duct", "term").unwrap();
let dp_pa = net.solve(None).unwrap();                  // 23.3 Pa for this chain
```

Per-component results: `extract_results(&net)`, `results_summary`,
`results_as_csv`.
## 2. Size ducts

Sizing returns a standard EN 1505/1506 cross-section, the actual velocity and
(method-dependent) the per-metre pressure drop. `…_round` / `…_rectangular`
variants exist for each method.

```rust
let (section, v) = velocity_method_round(0.1, 4.0).unwrap();          // v ≤ 4 m/s
let (section2, v2, dp_per_m) =
    equal_friction_method_round(0.1, 1.0, 0.0001, &STANDARD_AIR).unwrap(); // ΔP/m ≤ 1.0
let (section3, v3) = velocity_method_rectangular(0.1, 4.0).unwrap();
```

Related: `pressure_drop_budget_round` (size against a total ΔP budget),
`noise_limit_method`, `aspect_ratio_method`, `nearest_round_size`.

## 3. Use fittings

Fittings add local losses `ΔP = ζ·ρv²/2`: `TwoPortFitting(name, area, ζ)`,
`Tee` (straight + branch ζ), or a ζ from the fittings library (elbows, reducers,
tees, dampers; provenance in `docs/ZETA-SOURCES.md`).

```rust
let z = elbow_round_loss(0.2, 0.2, 90.0, 4.0, 1.204, 1.825e-5).unwrap(); // elbow ζ, Re/size-corrected
let zz = fire_damper(100.0).unwrap();
let z3 = named_zeta("pressed_branch_tee").unwrap_or(0.5);
```

## 4. Insulation

Size insulation for **condensation prevention** (cold supply air in a warm,
humid space — outer surface must stay above dew point) or a **heat-loss limit**
(W/m target), per EN ISO 12241. Returns thickness in metres; `select_thickness`
snaps to standard steps.

```rust
let lam = material_conductivity("mineral_wool").unwrap();                       // 0.035 W/(m·K)
let t_cond = required_thickness_condensation(8.0, 15.8, 24.0, lam, 0.2, 10.0, 8.0).unwrap();
let t_loss = required_thickness_heat_loss(60.0, 20.0, 10.0, lam, 0.2, 10.0, 8.0).unwrap();
```

## 5. Fan selection

Model a vendor fan by its static-pressure curve (`FanCurve` polyline), pick
the first fan meeting the duty point (design flow, required static) and
compute shaft power.

```rust
let fans = [FanCurve::new("td-350", vec![
    FanPoint { flow_m3s: 0.0, static_pressure_pa: 220.0 }, FanPoint { flow_m3s: 0.2, static_pressure_pa: 40.0 },
]).unwrap()];
let idx = pick_fan(&fans, 0.1, 130.0).unwrap();      // first fan meeting the duty
let power_w = fan_power(0.1, 130.0, 0.6).unwrap();   // P = Q·p/η
```

`margin_pa` reports the fan's pressure headroom at the duty point.

## 6. Room air balance

Track per-room supply/exhaust pairs, nets and imbalance; check the overall
balance; compute air changes per hour.

```rust
let mut set = RoomBalanceSet::new();
set.add_with_volume("office", RoomBalance::new(0.15, 0.15).unwrap(), 120.0);
set.add("wc", RoomBalance::new(0.02, 0.05).unwrap());
let balanced = set.is_balanced(0.01);              // |Σnet| ≤ tolerance
let ach = room_ach(0.05, 50.0).unwrap();           // 3.6 ACH
println!("{}", set.csv_render());
```

## 7. Clash detection

Trace 2D duct centrelines into segments and detect clashes (centreline
distance within combined radii plus clearance).

```rust
let traced = trace(&[Polyline::new(vec![(0.0, 0.0), (5.0, 0.0)])], &TraceOptions {
    snap: 1e-3, default_diameter: 0.2, diameters: Default::default(), flows: Default::default(), }).unwrap();
let clashes = find_clashes(&traced.flatten(), 0.1).unwrap();
let n = clash_count(&clashes);   // clashes_as_csv renders a report
```

## 8. BOM / analysis

Derive a bill of materials from a solved network — per-component length,
surface area and weight — plus fabricator cutting patterns.

```rust
let items = build_bom(&net).unwrap();
let len = total_length(&items);                   // total duct length [m]
let area = total_area(&items);                    // total sheet-metal area [m²]
let kg = duct_weight_kg(area, Some(0.6), None).unwrap();  // steel, 0.6 mm gauge
println!("{}", bom_as_csv(&items));
```

`fabrication::round_duct_development` / `round_elbow_development` lay out flat
patterns.

## 9. Export

Serialize networks (JSON/YAML) and export schedule tables to Excel or PDF
(`export` feature; JSON/YAML I/O behind `cli`).

```rust
#[cfg(feature = "export")]
{
    let rows: Vec<Vec<String>> = extract_results(&net)
        .iter().map(|r| vec![r.component_id.clone(), format!("{:.2}", r.pressure_drop)]).collect();
    let xlsx = schedule_to_xlsx_bytes(&["id", "dp_pa"], &rows).unwrap();           // .xlsx bytes
    let _pdf = schedule_to_pdf_bytes(&["id", "dp_pa"], &rows).unwrap();            // PDF bytes
}
```

`electrical_schedule_to_xlsx` / `_pdf` cover the electrical schedule.

---
## Cheat sheet

| Step | Entry point | Output |
|---|---|---|
| Size a duct | `velocity_method_round` / `equal_friction_method_round` | section, v, ΔP/m |
| Solve a network | `Network::solve(None)` | critical-path ΔP [Pa] |
| Fitting ζ | `components::fittings_library::*`, `elbow_round_loss` | ζ |
| Insulation | `insulation::required_thickness_*` | thickness [m] |
| Fan duty | `pick_fan`, `margin_pa`, `fan_power` | fan index, headroom, power |
| Room balance | `RoomBalanceSet`, `room_ach` | nets, ACH, CSV |
| Clashes | `find_clashes`, `clash_count` | clash list |
| BOM | `build_bom`, `bom_as_csv` | items, lengths, areas |
| Export | `schedule_to_xlsx_bytes` / `_pdf` | bytes / files |