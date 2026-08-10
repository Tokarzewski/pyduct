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
