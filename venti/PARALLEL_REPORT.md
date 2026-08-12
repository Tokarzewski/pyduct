# venti — Parallel-Agent Build Session

Four `pi` sub-agents worked **in parallel**, each in its own git worktree /
branch on a distinct Phase-0 issue. All four were merged into `main` and the
integrated crate passes **76 unit + 2 io-roundtrip + 5 parity + 1 doctest**
tests, with `clippy -D warnings` and `cargo fmt --check` clean.

| Branch | Issue | Deliverable | New module/API |
|---|---|---|---|
| `venti/sound` | #5 | Sound-calculation engine | `venti::sound` |
| `venti/balance` | #6 | System-balancing core | `venti::balancing` |
| `venti/fittings` | #7 | Fittings-library expansion | 4 new correlations |
| `venti/io` | #4 | Network save round-trip | `venti::io` save + `venti save` CLI |

---

## Sound — Issue #5 (`venti::sound`, 10 tests)

Mirrors CADvent's "calculation of sound": regenerated duct noise, the room
equation (power → pressure), and NC compliance.

- `regenerated_noise_round(v, d, rho)` — `Lw = 10 + 10·log10(ρ/ρ₀) + 60·log10(v) − 20·log10(d)` (Lighthill v⁶ scaling; falls with duct size).
- `duct_pressure_level(lw, S, α)` — diffuse-field room equation `Lp = Lw + 10·log10(4(1−α)/(α·S))`.
- `nc_ok(space_type, level)` / `nc_ok_target(nc, level)` with `NOISE_LIMITS_NC` (studio/bedroom 25, office/classroom 35, retail 40, industrial 60).

## Balancing — Issue #6 (`venti::balancing`, 4 tests)

CADvent/VentPack-style balancing: the damper ζ (and open %) a branch needs to
hit its target flow.

- `required_zeta(dp, v, ρ) = 2·dp/(ρv²)`
- `balancing_zeta(total, avail, v, ρ) = 2·Δ/(ρv²)` (0 if branch already meets it)
- `damper_open_percentage(ζ)` — invert butterfly: `open = 100·(1 − √((ζ−0.1)/10))`, clamped
- `balancing_zeta_batch(...)` — per-branch vector

## Fittings — Issue #7 (`fittings_library.rs`, 7 tests)

Four new loss correlations in the existing `Result<f64, String>` style:

- `taper_transition` — two-port taper (ASHRAE F23 / Idelchik §4), reducer/expander blend by area ratio + angle factor
- `cross_fitting` — 4-way cross: `ζ_main = 0.12 + 0.3·ratio + 0.1·A`, `ζ_branch = 0.5 + 0.8·(1−A) + 0.5·ratio`
- `fire_damper(open%)` — `0.18 + 30·(1−open/100)²`
- `attenuator_open(open_frac)` (+ alias `attenuator`) — `0.35 + 8·(1−open)²`

## I/O — Issue #4 (save round-trip, `venti::io` + `venti save`, 2 tests)

Completed the LOAD→SAVE round-trip in the wenta YAML/JSON format:

- `save_network_to_path` / `save_network_to_json_string` (matching the loader's schema)
- `venti save <in> [--out <out>]` CLI command
- `Network::connections()` accessor + `network.rs` serialization support
- `tests/io_roundtrip.rs` — load `examples/network_yaml.yaml` → save → reload →
  assert network signature equal and critical-path ΔP preserved (84.0632 Pa).

---

### How the parallel run was set up
- Each agent ran `pi --print` in its own `git worktree` on a dedicated branch,
  launched simultaneously as background processes.
- Per-agent `agent.log` captured output; each wrote a per-worktree `REPORT.md`.
- After completion, branches were committed, merged into `main` (clean — the
  only conflict was per-agent `REPORT.md`, resolved by consolidating here), and
  the whole crate re-formatted + re-verified.

---

# Round 2 — Parallel-Agent Build Session (4 agents)

A second batch of 4 parallel `pi` agents (dedicated worktrees/branches) delivered:

| Branch | Issue | Deliverable |
|---|---|---|
| `venti/standards` | #8 | Configurable standards: `venti::standards` (Standard enum: EN/ASHRAE/DIN size tables + `nearest_round_size_for`) |
| `venti/bench` | #10 | Per-kernel benchmark binary `src/bin/bench.rs` (+ `just bench` + README table) |
| `venti/cabi` | #12 | C-ABI exports for sound + balancing cores in `ffi.rs` |
| `venti/example` | — | `examples/design_workflow.rs` — end-to-end pipeline demo (size → solve → schedule → save → sound → balance → fittings) |

Integrated result: **91 tests pass** (83 unit + 2 io + 5 parity + 1 doctest),
clippy `-D`-clean, fmt clean, WASM builds (217 KB) and runs from Node.

- **Standards (#8):** `Standard::{En1505_1506, AsHrae, Din}`; ASHRAE (inch-derived) + DIN (Renard R10/R20) tables in mm; `round_sizes_mm`, `rect_sizes_mm`, `nearest_round_size_for`; existing `data::standard_sizes` untouched.
- **Bench (#10):** `cargo run --release --bin bench` times friction/reynolds/drops/sizing/batch/network-solve; measured e.g. friction_factor ~18 M calls/s, local_pressure_drop ~1 G calls/s, velocity_method_round ~118 M calls/s, network build+solve ~22.7 k/s.
- **C-ABI (#12):** `venti_regenerated_noise_round`, `venti_duct_pressure_level`, `venti_nc_ok`, `venti_required_zeta`, `venti_balancing_zeta`, `venti_damper_open_percentage` — verified in `venti.wasm`; 2 ffi tests.
- **Example:** deterministic 7-step workflow printing size, ΔP=23.28 Pa, schedule, JSON, NC compliance, damper open %, and a fire-damper zeta — runs cleanly.

_(Note: the standards agent's `REPORT.md` was a copy/paste of the sound report; the code itself is a real, tested `standards` module — see `venti/src/standards.rs`.)_
---

# Fitting-library expansion (continued core work)

Expanded `venti::components::fittings_library` (now **21 correlations** + a
named-zeta catalog seed):

| New correlation | Notes |
|---|---|
| `elbow_round(R,d,angle)` | round elbow, `zeta = min(0.21/√(R/D),1)·(angle/90)`, R/D ≥ 0.5 |
| `reducer_rectangular(w,h,w',h',angle)` | contraction ref. outlet velocity, `(0.04+0.37(1−Aₒ/Aᵢ))·f(angle)` |
| `expander_rectangular(...)` | diffuser ref. inlet velocity, Borda–Carnot `(1−Aᵢ/Aₒ)²·f(angle)` |
| `louver_open(open%)` | `0.25 + 4·(1−open/100)³` |
| `filter_bank(open_frac)` | `0.12/open_frac²` (media clog) |
| `round_tap_branch(d_main,d_tap,split)` | tap into round main |
| `named_zeta(name)` / `NAMED_FITTING_ZETAS` | constant device zetas (FR-19 catalog seed) |

All are `Result<f64,String>` + validated + doc'd; 8 new tests (13 → 21).
Rounded out the C-ABI too: `venti_elbow_round`, `venti_reducer_rectangular`,
`venti_expander_rectangular`, `venti_louver_open`, `venti_filter_bank`,
`venti_round_tap_branch`, `venti_named_zeta` — all present in `venti.wasm`.
Suite: **96 unit** + 2 io + 5 parity + 1 doctest, clippy + fmt clean.

---

# Round 3 — Parallel-Agent Build Session (4 agents: room #44, fan #40, fire-dampers #42, electrical #41)

Four parallel `pi` agents, each in its own worktree/branch, delivered new core modules (merged, 146 unit tests):

| Branch | Issue | Deliverable |
|---|---|---|
| `venti/room` | #44 | `venti::room` — per-room supply/exhaust air balance (`RoomBalance`, `room_ach`, `RoomBalanceSet` + CSV) |
| `venti/fan` | #40 | `venti::fan` — fan curves, interpolation, duty-point selection (`FanPoint`, `FanCurve`, `fan_power`, `pick_fan`) |
| `venti/fdamp` | #42 | Fire-damper vendor library Trox/Mercor: `reference_catalog()` entries + `fire_damper_branded` + `examples/fire_dampers_trox_mercor.json` |
| `venti/elect` | #41 | `venti::electrical` — equipment electrical data model + schedule + CSV (`ElectricalData`, `ElectricalSchedule`, `electrical_as_csv`) |

All four: clippy `-D` clean, fmt clean, unit tests passing on the integrated `main`.

---

# Round 4 — Parallel-Agent Build Session (4 agents: development #43, fabrication #30, Lindab/Alnor #31, xlsx export #45)

Four parallel `pi` agents delivered (merged; integrated suite now **196 tests**):

| Branch | Issue | Deliverable |
|---|---|---|
| `venti/dev` | #43 | `venti::development` — sheet-metal flat patterns: `round_duct_development`, `round_elbow_development` (segmented strip), `reducer_cone_development` (trapezoid) |
| `venti/fab` | #30 | `venti::fabrication` — `duct_surface_area_m2`, `duct_weight_kg`, `FabricationBreakout`, `cutting_schedule` |
| `venti/lib` | #31 | Vendor catalogue: `examples/vendor_lindab_alnor.json` (8+ entries) + `ZetaCatalog::by_vendor` + `from_vendor_json` |
| `venti/xlsx` | #45 | `venti::export` (feature `export`, `rust_xlsxwriter`): `schedule_to_xlsx_bytes`, `electrical_schedule_to_xlsx`; core stays dependency-free; PDF is a documented follow-up |

All four: clippy `-D` clean, fmt clean, tests green (incl. `--features export` and `--no-default-features`).

---

# Round 5 — Parallel-Agent Build Session (2 agents: FreeCAD trace, PDF export)

| Branch | Issue | Deliverable |
|---|---|---|
| `venti/trace` | (FreeCAD) | `venti_topology_trace` C-ABI export (sketch polylines → network handle) + `venti_core.trace_network` (wasm+ctypes) + `VentiTrace` FreeCAD command + pytest (6 pass) |
| `venti/pdf` | #45 (finish) | `venti::export` PDF renderers (`schedule_to_pdf_bytes`, `electrical_schedule_to_pdf`) via printpdf, feature-gated; core stays dependency-free |

Integrated: **169 unit** + 2 io + 5 parity + 22 doctests; xlsx+PDF under `--features export`; clippy `-D` clean; FreeCAD pytest 6 passed.
