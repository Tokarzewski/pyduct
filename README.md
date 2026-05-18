# wenta

Ductwork design library — sizing, pressure-drop, fitting losses, network
solving — being ported from Python to Mojo. The repo currently hosts both
implementations side by side; the Python package is production-ready, the
Mojo port covers the entire pure-math + sizing + fittings surface and is
diff-tested against it.

```
wenta/                 ← native-Mojo port (28 parity-tested kernels)
python/pyduct/            ← reference Python implementation (the oracle)
python/tests/             ← Python pytest suite       (184 tests)
wenta/tests/           ← Mojo TestSuite suite      (20 unit + 28 parity)
wenta/benchmarks/      ← Mojo↔Python speedup numbers
```

## Coverage status

| Module                        | Mojo (`wenta/`)        | Python (`python/pyduct/`) |
|-------------------------------|---------------------------|---------------------------|
| Cross-section geometry        | ✅                         | ✅                         |
| Fluid + altitude correction   | ✅                         | ✅                         |
| Friction & losses             | ✅                         | ✅                         |
| Flex-duct correction          | ✅                         | ✅                         |
| Unit converters               | ✅                         | ✅                         |
| EN standard sizes             | ✅                         | ✅                         |
| Sizing (velocity / EF / budget / NC / aspect-ratio) | ✅ (round + rect) | ✅           |
| Fittings library (9 correls)  | ✅                         | ✅                         |
| Component classes (Tee, …)    | —                          | ✅                         |
| Network / solver              | partial (Mojo critical-path) | ✅                       |
| Pydantic schemas + I/O        | —                          | ✅                         |
| Visualization                 | —                          | ✅                         |

The remaining gaps need Mojo struct types and Mojo collections; everything
below that line is native Mojo today.

## Quick start (Python — currently production)

```bash
uv sync --extra dev
just test-all     # Python + Mojo unit + Mojo parity
```

```python
from pyduct import (
    Network, Source, RigidDuct, Terminal, Round, solve,
    velocity_method, results_summary,
)

section, v = velocity_method(0.1, "round", target_velocity=4.0)
net = Network("example")
net.add("ahu",  Source("AHU"))
net.add("duct", RigidDuct("duct", section, length=20))
net.add("term", Terminal("terminal", flowrate=0.1))
net.connect("ahu", "duct"); net.connect("duct", "term")
print(net.solve(), "Pa")
```

Under the hood, `net.solve()` calls a Mojo critical-path DP kernel via
`mojo.importer`.

## Quick start (Mojo — direct, full speedup)

```mojo
from wenta.core.geometry import Round
from wenta.core.fluid import standard_air
from wenta.physics.friction import friction_factor, reynolds, relative_roughness
from wenta.physics.losses import straight_pressure_drop
from wenta.sizing import velocity_method_round, aspect_ratio_method

def main() raises:
    var section, v = velocity_method_round(0.1, target_velocity=4.0)
    print("sized D =", section.diameter, "m at v =", v, "m/s")

    # Or a flat rectangular duct:
    var rect, vr = aspect_ratio_method(0.2, target_velocity=4.0, aspect_ratio=2.5)
    print("flat duct:", rect.width, "×", rect.height, "v =", vr)
```

## Mojo kernel speedup vs Python reference

Measured on a recent laptop CPU, Mojo 26.2 stable. Numbers vary with
hardware but the ratios are stable. Run `just mojo-suite` to reproduce;
`uv run pyduct.sizing.velocity_method_batch` for the batch row.

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

`velocity_method_batch` sizes 200 ducts in a single Mojo call via a
zero-copy numpy ndarray — one Python↔Mojo boundary crossing for the
whole sweep instead of 200.

Caveat: per-call boundary cost between Python and Mojo is ~600 ns. For
single-call use from Python, the boundary dominates the math. The Mojo
speedup materialises when the kernel runs in a Mojo loop (the benchmark
shape) or when the network solver crosses the boundary once and lets
Mojo do the whole walk. The Python column reflects the *current* paths
through Python's `pyduct` module, which routes through the same Mojo
shims for everything except the per-component dispatch in the solver —
so the speedups are net of any boundary cost in the Python side too.

## Parity contract

The Python `pyduct` is the **reference oracle**. Every Mojo function is
diff-tested against its Python counterpart over a corpus of inputs, with
tolerance ≤ 1e-9 relative (1e-12 for non-transcendental closed forms).
The check runs as `just mojo-parity` and currently covers 28 functions.

## Layout

```
wenta/                       # Mojo port
├── core/
│   ├── geometry.mojo           # Round / Rectangular / equivalent_round_diameter
│   └── fluid.mojo              # Fluid / standard_air / air_at_altitude
├── physics/
│   ├── friction.mojo           # reynolds / friction_factor / Colebrook iterator
│   ├── losses.mojo             # straight & local pressure-drop
│   └── flex.mojo               # stretch_correction_factor
├── data/standard_sizes.mojo    # EN 1505/1506 sizes + nearest_round_size
├── units.mojo                  # cfm / inwc / ft / fpm / °F / ACH helpers
├── sizing.mojo                 # velocity / EF / budget / aspect / noise — round + rect
├── components/
│   └── fittings_library.mojo   # 9 loss correlations (full library parity)
├── network/solver.mojo         # critical_path_sum kernel
├── ext/solver_ext.mojo         # Python extension (Mojo callable from Python)
├── tests/
│   ├── test_core.mojo          # 20 unit tests with closed-form expected values
│   └── test_parity.mojo        # 28 parity tests vs. Python (std.python interop)
└── benchmarks/
    ├── bench_friction.mojo     # friction_factor + a couple of sizing fns
    └── bench_suite.mojo        # full kernel-by-kernel comparison table

python/pyduct/                  # Python implementation (the reference)
python/tests/                   # pytest suite (184 tests, mypy + ruff clean)

docs/                           # historical design notes from the Python redesign
```

## Command-line interface

After ``pip install`` the package exposes a ``wenta`` console script:

```bash
wenta solve   network.yaml                       # text table (default)
wenta solve   network.yaml --format markdown     # markdown report
wenta solve   network.yaml --format json         # machine-readable
wenta solve   network.yaml --format csv          # spreadsheet-friendly
wenta solve   network.yaml --output report.md    # write to file
wenta info    network.yaml                       # structural summary, no solve
wenta validate network.yaml                      # schema + structural check
```

Accepts YAML or JSON input (any input that ``Network.from_yaml`` or
``Network.from_json`` can load).

## Development

```bash
just check         # Python pytest (195)
just types         # mypy python/pyduct
just lint          # ruff
just mojo-test     # Mojo unit tests (20)
just mojo-parity   # Mojo parity tests (28)
just mojo-suite    # Full Mojo↔Python speedup table
just test-all      # Python + Mojo unit + Mojo parity
```

## Bibliography

- ASHRAE Handbook — Fundamentals
- Hendiger, Ziętek, Chludzińska: *Wentylacja i Klimatyzacja — Materiały pomocniczne do projektowania*
- Swamee & Jain (1976): *Explicit equations for pipe-flow problems*
- Colebrook–White equation (friction factor correlation)
- Idelchik: *Handbook of Hydraulic Resistance*

## License

MIT
