# venti

**Ductwork design library** — sizing, pressure-drop, fitting losses, and
network solving — written in **Rust**. This is the Rust port of the **wenta**
Python reference and the **wentamojo** Mojo port that live in the `pyduct`
repository.

The crate mirrors the reference module layout so the three implementations
can be diff-tested against each other over a shared corpus of inputs.

```
venti/
├── src/
│   ├── core/       geometry (Round / Rectangular) + fluid properties
│   ├── physics/    friction, losses, flex-duct corrections
│   ├── data/       EN 1505/1506 standard sizes
│   ├── units.rs    unit converters + air-changes-per-hour
│   ├── sizing.rs   velocity / EF / budget / noise / aspect-ratio sizing
│   ├── components/ ducts, fittings, terminals, round elbow + fittings
│   ├── catalog/    ζ database (FR-19): reference + vendor merge, serde JSON
│   ├── network/    graph model + solver (critical-path DP, batch kernel)
│   ├── topology/   M3 core: trace polylines → Network, flatten → draw segments
│   ├── ffi.rs      C-ABI exports (the WASM / cdylib symbol surface)
│   └── main.rs     CLI (`venti solve|info|validate <network.[yaml|json]>`)
├── host/           language-example hosts (Node->WASM, Python ctypes->cdylib)
├── scripts/        build-wasm.sh
├── examples/
│   └── network_yaml.yaml   (the same 3-zone example wenta ships)
└── tests/…  (47 unit tests + doctest; all green)
```

## Coverage vs. the reference

| Module                        | Python `wenta` | Mojo `wentamojo` | Rust `venti` |
|-------------------------------|:---:|:---:|:---:|
| Cross-section geometry        | ✅ | ✅ | ✅ |
| Fluid + altitude correction   | ✅ | ✅ | ✅ |
| Friction & losses             | ✅ | ✅ | ✅ |
| Flex-duct correction          | ✅ | ✅ | ✅ |
| Unit converters               | ✅ | ✅ | ✅ |
| EN standard sizes             | ✅ | ✅ | ✅ |
| Sizing (velocity/EF/budget/NC/aspect) | ✅ | ✅ | ✅ |
| Fittings library (correls)      | ✅ | ✅ | ✅ |
| ζ database / vendor catalog (FR-19) | partial | — | ✅ (reference + serde vendor JSON) |
| Component classes (Source/Terminal/Duct/Flex/Fitting/Tee) | ✅ | ✅ | ✅ |
| Round elbow (spline)          | ✅ (scipy) | — | ✅ (bilinear) |
| Network graph model + solver  | ✅ (NetworkX) | partial | ✅ (self-contained) |
| Geometry topology (trace/flatten) | partial | — | ✅ (host-agnostic, M3 core) |
| Critical-path DP kernel       | ✅ | ✅ | ✅ |
| Batch pressure-drop kernel    | ✅ | ✅ | ✅ |
| Pydantic schemas + visualisation | ✅ | — | — (CLI tables) |

## Use as a library

```rust
use venti::{Network, ComponentEnum, Source, RigidDuct, Terminal, Round, velocity_method_round};

// Size a round duct for 0.1 m^3/s at a target velocity of 4 m/s.
let (section, v) = velocity_method_round(0.1, 4.0).unwrap();
assert!(v <= 4.0);

// Or solve a small network end to end.
let r = Round::new(0.2).unwrap();
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
# Ok::<(), Box<dyn std::error::Error>>(())
```

## CLI

```bash
venti solve   examples/network_yaml.yaml            # text table (default)
venti solve   examples/network_yaml.yaml --format markdown
venti solve   examples/network_yaml.yaml --format json
venti solve   examples/network_yaml.yaml --format csv
venti info    examples/network_yaml.yaml            # structural summary, no solve
venti validate examples/network_yaml.yaml           # structural check
venti catalog                              # dump the ζ database (text/csv/json)
venti catalog --vendor examples/vendor_catalog.json  # merge a vendor catalogue
venti report  examples/network_yaml.yaml           # per-component schedule table
venti report  examples/network_yaml.yaml --format json|csv
```

Accepts YAML or JSON input in the same `wenta` network format.

## Roadmap

The plan to turn `venti` into a CAD ductwork-design plugin is tracked **as
GitHub issues** on `ModelTok/pyduct` (milestones **M1–M6**, `p0`–`p3` + phase
labels). `venti::results` (schedule engine) and `venti::io` (network
serialization) ship in the crate; see the issues for the active build-out.

## Development

```bash
cargo build            # library + cli (server/clap)
cargo test             # 47 unit tests + doctest
cargo build --no-default-features --lib   # library only (no serde/clap)
```

## Per-kernel benchmarks

`venti` ships a dependency-free benchmark binary (no criterion — just
`std::time::Instant` + `std::hint::black_box`) that times each kernel over many
iterations and prints a (kernel, n, total_ms, per_call_s, calls_per_sec) table.

```bash
cargo run --release --bin bench
# or, if you have `just` (see the repo-root justfile):
just bench
```

Example output (single machine, `--release`, varies by hardware):

```
== venti (Rust) == kernel micro-benchmarks
kernel                                        n     total_ms     per_call_s    calls_per_sec
friction_factor                         1000000       57.546       5.755e-8       17377452.6
reynolds                                1000000        1.208       1.208e-9      827567258.5
local_pressure_drop                     1000000        0.968      9.677e-10     1033378113.1
straight_pressure_drop                  1000000        1.533       1.533e-9      652225949.3
velocity_method_round                     50000        0.422       8.435e-9      118559267.8
equal_friction_method_round               50000       26.343       5.269e-7        1898059.3
aspect_ratio_method                       50000       27.169       5.434e-7        1840337.7
velocity_method_batch (200 ducts)          1000        1.409       1.409e-6         709632.0
network build+solve (3-zone)               1000       43.965       4.397e-5          22745.1
```

For reference, the Python/Mojo numbers below are from the existing
`wentamojo/benchmarks/bench_suite.mojo` table in the `pyduct` README (same
hardware class, Mojo 26.2); they are **not** measured by this crate:

| Kernel                          | n         | Mojo     | Python  | Speedup |
|---------------------------------|-----------|----------|---------|---------|
| `friction_factor`               | 1 000 000 |  49 ms   |  653 ms | **13×** |
| `local_pressure_drop`           | 1 000 000 | 1.6 ms   |  726 ms | **441×**|
| `velocity_method_round`         |    50 000 | 2.8 ms   |   81 ms | **29×** |
| `velocity_method_rectangular`   |    50 000 | 7.1 ms   |  112 ms | **16×** |
| `equal_friction_method_round`   |    50 000 |  35 ms   |  652 ms | **19×** |
| `aspect_ratio_method`           |    50 000 |  29 ms   |  562 ms | **20×** |
| `rectangular_elbow`             |   100 000 | 6.0 ms   |   94 ms | **16×** |
| `velocity_method_batch` (×200)  |  1 000 ×  |  10 ms   |  409 ms | **40×** |

Use these only as a rough relative sense of where the native core sits; rerun
`cargo run --release --bin bench` on your own machine for venti's actual
numbers.

## Embed as a WebAssembly core

`venti` compiles to a small (~86 KB) self-contained **WASM core** exposing a
clean C ABI — the same functions you call from Rust, now callable from any
host that embeds a WASM runtime (Node, Python `wasmtime`, .NET `Wasmtime`,
C++'s wasmtime C API, a browser, or a WASM-capable CAD scripting host).

```bash
rustup target add wasm32-wasip1
./scripts/build-wasm.sh --release
# -> target/wasm32-wasip1/release/venti.wasm  (exports memory + 28 `venti_*` fns)
```

The `.wasm` exports only the `venti_*` extern-C functions plus `memory`, all
with plain f64/i32 signatures. Multi-value results use caller-allocated `*mut
f64` out-params (WASM can't return multi-word structs by value). Every
fallible function returns an `i32` status (`0` = ok).

Key entries (full list in `src/ffi.rs`):

```text
venti_friction_factor(re, eps) -> f64
venti_velocity_method_round(flowrate, target, &diam_m, &v) -> i32
venti_equal_friction_method_round(flowrate, target, eps, rho, mu, …) -> i32
venti_batch_compute(types, n, params, port_idx, flows, p, rho, nu, &v, &dp) -> i32
venti_critical_path_sum(dp, n, pred_counts, pred_offsets, pred_flat) -> f64
```

The `.wasm` also ships a **handle-based network API** so a host can build a
network, solve it, and read per-component results without any Rust code:

```text
venti_network_create(name, len) -> handle         # non-negative handle
venti_network_add(handle, id, id_len, name, name_len, type, params[6]) -> status
venti_network_connect(handle, src, src_len, tgt, tgt_len) -> status
venti_network_solve(handle, density, dynamic_viscosity) -> critical-path ΔP
venti_network_component_count(handle) -> i32
venti_network_validate(handle, &problem_count) -> status
venti_results_count(handle) -> rows (one per component)
venti_results_row(handle, idx, &q_in, &q_set, &v_in, &v_set, &dp) -> status
venti_results_field_string(handle, idx, field, buf, cap, &len) -> status
venti_network_free(handle) -> status
venti_alloc(len) -> ptr      # safely allocate host buffers in the WASM heap
venti_free(ptr, len)
```

Component `type` tags match the solver (`0` Source, `1` Terminal, `2` RigidDuct,
`3` FlexDuct, `4` TwoPortFitting, `5` Tee) with a 6-`f64` param array per type
(see `src/ffi.rs`). `host/wasm_node_example.js` builds, solves, and reads a
network end-to-end this way.

Example host call from Node:

```js
const { instance } = await WebAssembly.instantiate(readFileSync("venti.wasm"), {
  wasi_snapshot_preview1: new (require("node:wasi").WASI)({ version: "preview1" }).wasiImport,
});
instance.exports.venti_friction_factor(50000, 0.0009)   // ~0.02364
```

Ready-to-run host examples live in `host/`: `wasm_node_example.js` (Node →
WASM) and `cdylib_python_example.py` (CPython `ctypes` → host `.so`). Both
work end-to-end and produce identical numbers (`friction_factor = 0.02365`,
`velocity_method_round(0.1, 4.0) -> D = 0.2 m, v = 3.18 m/s`).

Alternative embeddable artifact: the **host `cdylib`** (`.so`/`.dll`/`.dylib`)
produced by `cargo build --release --no-default-features --lib`, exposing the
same `venti_*` symbols for P/Invoke from C# or `ctypes` from CPython — the
WASM route avoids per-OS builds entirely.

## Docs

The plan for turning `venti` into a CAD ductwork-design plugin (copycating
CADvent / VentPack / Wentyle) is tracked **as GitHub issues** on
`ModelTok/pyduct`: milestones **M1 – M6** map to the roadmap phases and each
issue has goal + acceptance criteria + priority (`p0`–`p3`) + phase labels.
The library source itself is the source of truth for behaviour.

## Why is this a `cdylib`?

The crate sets `crate-type = ["rlib", "cdylib"]`, so the same source builds
two shapes:

| `crate-type` | Artifact | Purpose |
|---|---|---|
| `rlib` | `.rlib` | A Rust-only static archive, consumed by **other Rust crates** (`cargo test`, `use venti::…`). Not loadable outside Rust. |
| `cdylib` | `.so` / `.dll` / `.dylib` / `.wasm` | A **C-compatible shared library** exposing the `extern "C"` symbols via the C ABI — loadable from C, C++, Python, C#, Node, mobile, etc. |

A `cdylib` exports **only** the `#[no_mangle] extern "C"` functions (the 28
`venti_*` entries in `src/ffi.rs`), with plain `f64`/`i32`/pointer arguments.
Because those use the C calling convention, any language that can call C code
can call them: `[DllImport]` in C#, `ctypes` in Python, `dlopen`/link in
C/C++. This is the whole point for `venti`: it exists to be embedded into
other-language hosts (e.g. a C# ZWCAD plugin), and a `.rlib` could *only* be
consumed by Rust.

On the `wasm32-wasip1` target the same `cdylib` becomes a `*.wasm` instead of
a `.so` — identical C-ABI symbols, but a single portable artifact that runs on
any OS/CPU via a WASM runtime (no per-mac/linux/windows or per-architecture
builds). So:

- **host `cdylib`** (`.so`/`.dll`/`.dylib`) — one build per OS, 
  P/Invoke / `ctypes`.
- **WASM `cdylib`** (`venti.wasm`) — one build total, embed from anything
  that has a WASM runtime.

## Notes & intentional differences from the reference

- **Round elbow**: the Python reference uses scipy's `RectBivariateSpline`
  over its zeta table. To stay dependency-free `venti` uses **bilinear**
  interpolation over the same table — exact at table vertices, within a few
  percent between them. Swap in a cubic if you need spline parity.
- **Network graph**: the Python uses NetworkX; `venti` ships a tiny
  self-contained adjacency graph (the topology convention is identical), so
  `critical_path` and the batch/sum kernels are drop-in parity targets.

## License

MIT (matches the `pyduct` repository).
