# venti

**Ductwork design library** — sizing, pressure-drop, fitting losses, thermal
insulation, fan selection, room air balance, network solving, clash detection,
schedules (BOM) and report export — written in **Rust**, dependency-free at the
core. It is the Rust port of the **wenta** Python reference (and the
**wentamojo** Mojo port) in the `pyduct` repository, and it now goes well
beyond the reference with an HVAC-engineering feature set.

The core is embeddable three ways: as a Rust crate, as a **WASM core**
(`venti.wasm`, one cross-platform artifact), or as a native **`cdylib`**
(`libventi.so`/`venti.dll`), all exposing the same C ABI. A **FreeCAD
workbench** and a **ZWCAD .NET plugin** scaffold consume it.

## Module map

```
src/
├── core/          geometry (Round / Rectangular), Fluid, air_at_altitude
├── physics/       friction (Swamee–Jain + Colebrook), losses, flex-duct
├── data/          EN 1505/1506 sizes + sections + branch/transformation tables
├── standards.rs   selectable Standard (EN / ASHRAE / DIN) size tables
├── sizing.rs      velocity / EF / budget / noise / aspect-ratio (round+rect, batch)
├── units.rs       unit converters + air-changes-per-hour
├── components/    Source, Terminal, RigidDuct, FlexDuct, TwoPortFitting, Tee,
│                  ElbowRound + 23 fitting correlations (round/rect, device)
├── catalog.rs     ζ database (FR-19): reference catalog + vendor JSON merge
├── re.rs          Reynolds & size corrections for fitting ζ
├── network/       graph model + solver (flow propagation, ΔP, critical path,
│                  batch kernel, has_cycle, multi-source, large networks)
├── topology.rs    trace 2D polylines → Network (tees), flatten → segments
├── insulation.rs  duct insulation thickness (condensation, heat-loss) + materials
├── fan.rs         fan curves + duty-point selection + power
├── room.rs        per-room supply/exhaust balance + ACH
├── electrical.rs  equipment electrical data model + schedule
├── sound.rs       regenerated noise + room equation + NC compliance
├── balancing.rs   damper ζ / open-% for system balancing
├── analysis.rs    per-branch report (flow, velocity, ΔP, noise, balancing ζ)
├── marking.rs     branch numbering + ID marks
├── bom.rs         bill of materials (lengths, areas, totals)
├── clash.rs       duct segment clash detection (clearance)
├── development.rs sheet-metal flat patterns (ducts, elbows, reducer cones)
├── fabrication.rs surface area, weight, cutting schedule
├── settings.rs    ProjectSettings + JSON persistence
├── results.rs     per-component results / schedule / CSV / JSON
├── io.rs          network load/save (wenta YAML/JSON, feature-gated)
├── export.rs      xlsx + PDF report export (feature-gated)
├── ffi.rs         C-ABI exports (the WASM / cdylib symbol surface)
├── error.rs       unified venti::Error / venti::Result
├── bin/bench.rs   per-kernel benchmarks
└── main.rs        CLI (solve / report / info / validate / save / catalog /
                   bom / clash / settings)
```

## Coverage vs. the reference

| Capability | `venti` | Notes |
|---|---|---|
| Cross-section geometry, fluids, friction, losses | ✅ | |
| Sizing (velocity/EF/budget/NC/aspect), round+rect, batch | ✅ | |
| EN/ASHRAE/DIN standard tables | ✅ | `standards.rs` |
| Fittings — 23 correlations + round-elbow table | ✅ | round/rect, tees, dampers, louvers, filters, silencers |
| ζ database + vendor catalog (FR-19) | ✅ | serde JSON, merge by key (Lindab/Alnor, Trox/Mercor) |
| Re/size-corrected fitting ζ | ✅ | `re.rs` |
| Network graph + solver (critical path, cycles, multi-source) | ✅ | self-contained, robust |
| Topology: trace polylines → network, flatten → draw | ✅ | M3 core |
| Thermal insulation | ✅ | condensation + heat-loss, materials, selection |
| Fan selection & curves | ✅ | duty-point, power |
| Room air balance + ACH | ✅ | |
| Electrical data + schedule | ✅ | |
| Sound (regenerated noise, NC) | ✅ | |
| Balancing (damper ζ / open-%) | ✅ | |
| Per-branch analysis report | ✅ | |
| Branch marking | ✅ | |
| BOM / schedules | ✅ | |
| Clash detection | ✅ | |
| Sheet-metal developments | ✅ | |
| Fabrication breakout / weight | ✅ | |
| Results + CSV/JSON | ✅ | |
| Network I/O (YAML/JSON) | ✅ | `io.rs` (feature-gated) |
| Report export (xlsx + PDF) | ✅ | `export.rs` (feature-gated) |
| Settings persistence | ✅ | `settings.rs` (serde) |
| WASM core + native cdylib + C ABI | ✅ | 50+ `venti_*` exports |
| FreeCAD workbench | ✅ | `freecad/Mod/VentiDuct` |
| ZWCAD .NET plugin | ✅ scaffold | `plugin/` (builds on Windows + ZWCAD SDK) |

## Use as a library

```rust
use venti::{Network, ComponentEnum, Source, RigidDuct, Terminal, Round, velocity_method_round};

// Size a round duct for 0.1 m^3/s at a target velocity of 4 m/s.
let (section, v) = velocity_method_round(0.1, 4.0)?;
assert!(v <= 4.0);

// Solve a small network end to end.
let r = Round::new(0.2)?;
let mut net = Network::new("example");
net.add("ahu",  ComponentEnum::Source(Source::new("AHU")))?;
net.add("duct", ComponentEnum::RigidDuct(RigidDuct::new(
    "duct", r.area, r.hydraulic_diameter, 20.0, 0.0001,
)?))?;
net.add("term", ComponentEnum::Terminal(Terminal::new(
    "terminal", 0.1, Some(r.area), 1.0,
)))?;
net.connect("ahu", "duct")?;
net.connect("duct", "term")?;
let dp_pa = net.solve(None)?;
assert!(dp_pa > 0.0);
```

Every fallible function returns `venti::Result<T>` = `Result<T, venti::Error>`
(a `Message(String)`), convertible from `&str`/`String` with `.into()`.

## CLI

```bash
venti solve   examples/network_yaml.yaml [--format text|markdown|json|csv]
venti report  examples/network_yaml.yaml [--format text|json|csv]   # per-component schedule
venti info    examples/network_yaml.yaml            # structural summary
venti validate examples/network_yaml.yaml           # structural check
venti save    examples/network_yaml.yaml [--out out.json]   # round-trip serialize
venti catalog [--vendor examples/vendor_lindab_alnor.json]  # ζ database (text/csv/json)
venti bom     examples/network_yaml.yaml           # bill of materials + totals
venti clash                                     # clash-detection demo fixture
venti settings                                  # default ProjectSettings (json/text)
```

Accepts YAML or JSON input in the same `wenta` network format.

## Feature flags

```toml
default = ["cli"]          # CLI + network I/O (serde/clap)
cli                       # serde, serde_json, serde_yaml, clap, catalog I/O
export                    # xlsx (rust_xlsxwriter) + PDF (printpdf) report export
```

The **core is always dependency-free** — `cargo build --no-default-features
--lib` builds the library with zero third-party crates.

## Development

```bash
cargo build                       # library + CLI
cargo test                        # unit + io + parity + regression + doctests
cargo build --no-default-features --lib    # core only (dependency-free)
cargo build --features export      # + xlsx/PDF export
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
cargo run --release --bin bench    # per-kernel benchmark table
```

## Embed as a WebAssembly core

`venti` compiles to a small self-contained **WASM core** exposing a clean C ABI
(the same functions as the Rust crate), callable from any host that embeds a
WASM runtime — Node, Python `wasmtime`, .NET `Wasmtime`, a browser, or a
WASM-capable CAD scripting host.

```bash
rustup target add wasm32-wasip1
./scripts/build-wasm.sh --release
# -> target/wasm32-wasip1/release/venti.wasm  (exports 50+ `venti_*` fns + memory)
```

The `.wasm` exports only the `venti_*` extern-C functions plus `memory`, with
plain f64/i32 signatures. Multi-value results use caller-allocated `*mut f64`
out-params; fallible functions return an `i32` status (`0` = ok). Highlights:

```text
venti_friction_factor(re, eps) -> f64
venti_velocity_method_round(flowrate, target, &diam_m, &v) -> i32
venti_equal_friction_method_round(flowrate, target, eps, rho, mu, …) -> i32
venti_elbow_round_loss(R, d, angle, v, rho, mu) -> f64          # Re/size-corrected
venti_batch_compute(types, n, params, port_idx, flows, p, rho, nu, &v, &dp) -> i32
venti_critical_path_sum(dp, n, pred_counts, pred_offsets, pred_flat) -> f64
venti_topology_trace(points, n, poly_lens, n_polys) -> handle   # sketch → network
venti_network_create/add/connect/solve/results_row/free        # handle-based API
venti_alloc(len) / venti_free(ptr, len)                        # host buffers in WASM heap
```

Ready-to-run hosts: `host/wasm_node_example.js` (Node → WASM) and
`host/cdylib_python_example.py` (CPython ctypes → native cdylib) — both
produce identical numbers.

Alternative artifact: the **host `cdylib`** (`.so`/`.dll`/`.dylib`) via
`cargo build --release --no-default-features --lib`, for P/Invoke / `ctypes` /
`dlopen` when you don't want a WASM runtime (see "Why is this a `cdylib`?").

## FreeCAD workbench

`freecad/Mod/VentiDuct/` exposes the core as workbench commands (size / solve /
insulation / **trace a sketch into a duct network**). The math runs through
`venti.wasm` + `wasmtime` or the native cdylib via `ctypes`; `venti_core.py` is
pure Python and unit-tested standalone (`python3 -m pytest`). See
`freecad/Mod/VentiDuct/README.md`.

## ZWCAD / AutoCAD plugin scaffold

`plugin/` is a C# `.NET` scaffold (net48) that references the ZWCAD managed API
(`ZwSoft.ZwCAD.*`) and loads `venti.wasm` (Wasmtime) or P/Invokes the cdylib
behind a host-agnostic `IVentiCore` facade (native + wasm backends). Commands:
`VENTI`, `VENTI_SIZE`, `VENTI_SOLVE`. Builds on a Windows machine with the ZWCAD
SDK — see `plugin/README.md`. A testable, ZWCAD-independent `Venti.Core`
(netstandard2.0) carries the bindings + headless xunit tests.

## Docs

- `docs/DESIGN_GUIDE.md` — how to build/solve a network, size ducts, use
  fittings, insulation, fan selection, room balance, clash, BOM, export.
- `docs/ZETA-SOURCES.md` — curated sources of duct-fitting loss coefficients
  (ASHRAE, SMACNA, Idelchik, CIBSE) and the ζ data conventions.
- `tests/regression.rs` — 8 golden design cases guarding stability.
- `tests/parity.rs` — golden reference values + optional Python-oracle
  differential (`VENTI_PYTHON_ORACLE`).
- `PARALLEL_REPORT.md` — records of the parallel-agent build sessions (Rounds
  1–7) that grew the feature set.
- The CAD-plugin plan/roadmap is tracked **as GitHub issues** on
  `ModelTok/pyduct` (milestones M1–M6, priorities `p0`–`p3`, phase labels).

## Why is this a `cdylib`?

`crate-type = ["rlib", "cdylib"]` builds the same source two ways:

| `crate-type` | Artifact | Purpose |
|---|---|---|
| `rlib` | `.rlib` | Rust-only, for other Rust crates (`cargo test`, `use venti::…`) |
| `cdylib` | `.so` / `.dll` / `.dylib` / `.wasm` | C-compatible shared lib exposing the `extern "C"` symbols — callable from C, C++, Python, C#, Node, etc. |

A `cdylib` exports **only** the `#[no_mangle] extern "C"` functions, with the C
calling convention, so any language that can call C can use them. On
`wasm32-wasip1` the same `cdylib` becomes a `*.wasm` — identical symbols but a
single cross-platform artifact (no per-OS/arch builds).

## Notes & intentional differences from the reference

- **Round elbow**: Python uses scipy `RectBivariateSpline` over its ζ table;
  `venti` uses bilinear interpolation (exact at vertices, within a few percent
  between). `re.rs` adds Re/size corrections beyond the reference.
- **Network graph**: Python uses NetworkX; `venti` ships a self-contained
  adjacency graph with the same topology convention, so the solver kernels are
  drop-in parity targets.
- **Vendor ζ data** (Lindab/Alnor, Trox/Mercor) is representative reference
  data for design-tool estimates; production values should come from the
  vendors' official catalogues (the FR-19 JSON schema supports that).

## License

MIT (matches the `pyduct` repository).
