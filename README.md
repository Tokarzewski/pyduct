# mojoduct

Ductwork design library — sizing, pressure-drop, fitting losses, network
solving — being ported from Python to Mojo. The repo currently hosts both
implementations side by side.

```
mojoduct/                 ← native-Mojo port (in progress)
python/pyduct/            ← reference Python implementation (production-ready)
python/tests/             ← Python test suite (184 tests, mypy & ruff clean)
mojoduct/tests/           ← Mojo test suite (20 tests today)
```

## Hybrid plan

The Mojo port lands in layers. Math/physics primitives that need no graph
or schema libraries are native Mojo today; the higher-level graph solver
and serialization stay in the Python package and will be called from Mojo
via Python interop (`std.python`) until a Mojo-native replacement is ready.

| Module                      | Mojo (`mojoduct/`) | Python (`python/pyduct/`) |
|-----------------------------|--------------------|---------------------------|
| Cross-section geometry      | ✅                  | ✅                         |
| Fluid + altitude correction | ✅                  | ✅                         |
| Friction & losses           | ✅                  | ✅                         |
| Unit converters             | ✅                  | ✅                         |
| EN standard sizes           | —                  | ✅                         |
| Components (RigidDuct, Tee…) | —                  | ✅                         |
| Network / solver            | — (Python interop) | ✅                         |
| Pydantic schemas + I/O      | — (Python interop) | ✅                         |
| Visualization               | — (Python interop) | ✅                         |

## Quick start (Python — currently production)

```bash
uv sync --extra dev
uv run --extra dev pytest python/tests
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

## Quick start (Mojo — native math/physics)

Requires the Mojo toolchain (already vendored via `uv add mojo --prerelease allow`).

```mojo
from mojoduct.core.geometry import Round
from mojoduct.core.fluid import standard_air
from mojoduct.physics.friction import friction_factor, reynolds, relative_roughness
from mojoduct.physics.losses import straight_pressure_drop

def main() raises:
    var section = Round(0.2)            # 200 mm round duct
    var air = standard_air()
    var v = 0.1 / section.area
    var re = reynolds(v, section.hydraulic_diameter, air.kinematic_viscosity)
    var f = friction_factor(re, relative_roughness(0.0001, section.hydraulic_diameter))
    var dp = straight_pressure_drop(f, 20.0, section.hydraulic_diameter, v, air.density)
    print("dp =", dp, "Pa")
```

Run the Mojo test suites:

```bash
just mojo-test       # 20 unit tests   (closed-form expected values)
just mojo-parity     # 11 parity tests (every function diff-tested vs. Python)
just test-all        # Python suite + both Mojo suites
```

## Parity contract

The Python `pyduct` is the **reference oracle**. The Mojo port follows the
Branch-C pattern from the `migration-to-python-mojo` skill: every Mojo
function is diff-tested against its Python counterpart over a corpus of
inputs, with tolerance ≤ 1e-9 relative (1e-12 for non-transcendental
closed forms). If the Mojo side ever drifts, `just mojo-parity` fails
before anything ships.

This means: the Python implementation is allowed to evolve, the Mojo
implementation must stay numerically equivalent (within tolerance), and
the Mojo side is a verified performance escalation rather than a free-
form rewrite.

## Layout

```
mojoduct/                     # Mojo port
├── core/
│   ├── geometry.mojo         # Round / Rectangular / equivalent_round_diameter
│   └── fluid.mojo            # Fluid / standard_air / air_at_altitude
├── physics/
│   ├── friction.mojo         # reynolds / friction_factor / Colebrook iterator
│   └── losses.mojo           # straight & local pressure-drop
├── units.mojo                # cfm / inwc / ft / fpm / °F / ACH helpers
└── tests/
    ├── test_core.mojo        # 20 unit tests
    └── test_parity.mojo      # 11 parity tests vs. Python via std.python interop

python/pyduct/                # Python implementation (the reference)
python/tests/                 # pytest suite (184 tests)

docs/                         # historical design notes from the Python redesign
```

## Development

```bash
uv run --extra dev pytest python/tests     # Python suite (184 tests)
uv run --extra dev mypy python/pyduct      # type-check the Python side
uv run --extra dev ruff check .             # lint
uv run mojo run mojoduct/tests/test_core.mojo  # Mojo suite (20 tests)
```

## Bibliography

- ASHRAE Handbook — Fundamentals
- Hendiger, Ziętek, Chludzińska: *Wentylacja i Klimatyzacja — Materiały pomocniczne do projektowania*
- Swamee & Jain (1976): *Explicit equations for pipe-flow problems*
- Colebrook–White equation (friction factor correlation)

## License

MIT
