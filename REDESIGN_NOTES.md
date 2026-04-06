# pyduct redesign — `pyduct2`

A breaking redesign of `pyduct` lives in the new top-level package `pyduct2/`,
side-by-side with the original `pyduct/` (which is untouched). All new tests
are under `tests/v2/`.

## Why a rewrite

While reading the original code I found several real bugs and a number of
design issues that could not be fixed incrementally:

1. **Mutable class-attribute bug.** In `RigidDuct`, `FlexDuct`, `OneWayFitting`,
   `TwoWayFitting`, `ThreeWayFitting` and `Ductwork`, fields like
   `connectors = [Connector(), Connector()]` and `graph = nx.DiGraph()` were
   declared as class attributes (no `field(default_factory=...)`). Every
   instance shared the same list / graph. The existing `Ductwork.add_object`
   even did `obj = replace(obj)`, which produced a new dataclass instance —
   but `connectors` is a class attribute, so the "copy" still pointed at the
   same list. This is the kind of bug that bites silently.
2. **`elbow_round.__post_init__` overwrites its own method:** `self.dzeta =
   self.dzeta(self)` turns the method into a float and breaks any later
   call.
3. **`RigidDuctType` mixed round and rectangular shapes** in one dataclass
   with eight optional fields and a `shape` literal. Adding a new shape
   meant editing every method.
4. **Hard-coded magic numbers** in fittings (`area=1, dzeta=0.5`) and a
   global `rho=1.2` baked into the friction module — no way to vary fluid
   properties.
5. **Wildcard imports**, missing type hints in many places, no docstrings,
   filenames with typos (`crititcal`, `defintion`) and a literal space in
   `example6_calculate velocity.py`.
6. **Unused heavy dependency `coolprop`** in `pyproject.toml`.
7. **Counter-intuitive flow direction**: in the original `Ductwork`, graph
   edges pointed *backwards* — from terminals (graph sources) to AHU (graph
   sink) — so that `nx.dag_longest_path` could be used as the critical-path
   algorithm. It worked, but it required users to mentally invert physical
   airflow when wiring up the network.

## New layout

```
pyduct2/
├── core/
│   ├── fluid.py              # Fluid (frozen dataclass) + STANDARD_AIR
│   └── geometry.py           # CrossSection ABC: Round, Rectangular
├── physics/
│   ├── friction.py           # reynolds, friction_factor, friction_factor_colebrook
│   ├── losses.py             # straight_pressure_drop, local_pressure_drop
│   └── flex.py               # stretch_correction_factor
├── components/
│   ├── base.py               # Component ABC, Port (with direction: in|out)
│   ├── duct.py               # RigidDuct, FlexDuct
│   ├── fitting.py            # Source, Terminal, TwoPortFitting, Tee
│   └── elbow.py              # ElbowRound (zeta from EN tabulation)
├── data/
│   └── standard_sizes.py     # EN 1505/1506 + nearest_round_size()
├── network/
│   ├── network.py            # Network: components, ports, connect()
│   └── solver.py             # propagate_flowrates / compute / critical_path / solve
└── examples/
    └── small_supply.py       # equivalent of the old small_example_network.py
```

## Key design decisions

- **Physical flow direction**: graph edges follow real airflow (Source → ducts
  → Tee → Terminals). Building a network reads naturally:
  ```python
  net.connect("ahu", "duct1")
  net.connect("duct1", "tee1.combined")
  net.connect("tee1.straight", "term1")
  net.connect("tee1.branch", "term2")
  ```
- **Ports as first-class graph nodes** with explicit `name` and
  `direction`. The solver references `Tee` legs by name (`tee.branch`),
  not by 1-based index.
- **Pure-function solver** lives in its own module. `Network` just owns the
  graph; `propagate_flowrates`, `compute_pressure_drops`, `critical_path`
  and `solve` are stateless functions you can call separately or via the
  one-line `solve(net)`.
- **`Fluid` value object** is passed into every `compute()`. Default is
  `STANDARD_AIR` (20 °C, 101 325 Pa). No more module-level `rho = 1.2`.
- **`CrossSection` is a strategy object**: `Round(diameter)` and
  `Rectangular(width, height)`. Frozen dataclasses with validation in
  `__post_init__`.
- **Reproducible Colebrook**: `friction_factor_colebrook` is now a pure
  fixed-point iteration (no `scipy.optimize.root`), so the friction module
  has no SciPy dependency for the reference solver.
- **Scipy is still used by `ElbowRound`** for the bivariate spline lookup.
  That's the only reason `scipy` stays as a top-level dependency.
- **`coolprop` dropped** from required dependencies. The two air properties
  it was used for are baked into `STANDARD_AIR` as constants matching its
  output to four significant figures.
- **`matplotlib` is now optional** (`pyduct[plot]`). The library no longer
  imports it at module load.
- **Critical path uses edge weights**, not node weights. The original code
  passed `weight="pressure_drop"` to `nx.dag_longest_path`, but that
  argument is for *edges*, not nodes — meaning every edge defaulted to
  weight 1 and the "critical path" was just the longest-by-count path. The
  new solver copies node pressure drops onto target-side edge weights so
  that `dag_longest_path` actually maximises total static pressure.

## Public API at a glance

```python
from pyduct2 import (
    Network, Source, RigidDuct, FlexDuct, TwoPortFitting, Tee, Terminal,
    Round, Rectangular, ElbowRound, STANDARD_AIR, solve,
)

section = Round(diameter=0.2)

net = Network("supply")
net.add("ahu", Source("AHU"))
net.add("d1", RigidDuct("d1", section, length=10))
net.add("tee", Tee("tee", section))
net.add("t1", Terminal("term1", flowrate=0.05))
net.add("t2", Terminal("term2", flowrate=0.07))

net.connect("ahu", "d1")
net.connect("d1", "tee.combined")
net.connect("tee.straight", "t1")
net.connect("tee.branch",   "t2")

total_dp = solve(net)        # one-line solve
print(f"{total_dp:.1f} Pa")
```

## Tests

`tests/v2/` contains 55 tests across five files:

| File                | Coverage                                                       |
|---------------------|----------------------------------------------------------------|
| `test_geometry.py`  | `Round`, `Rectangular`, area / D_h, validation                 |
| `test_friction.py`  | `reynolds`, `friction_factor`, Swamee–Jain vs Colebrook (16 cases) |
| `test_components.py`| Each component in isolation, including the per-instance ports regression |
| `test_data.py`      | Sorted standard sizes, `nearest_round_size`                    |
| `test_network.py`   | Build, validation, propagation, end-to-end `solve`, critical path |

Both old and new test suites pass together: `pytest tests/ tests/v2/`
reports **57 passed**.

## Tooling

`pyproject.toml` now includes:

- `requires-python = ">=3.10"` (was 3.11; 3.10 is the minimum needed for
  `dict[str, ...]` PEP 585 generics under `from __future__ import annotations`).
- Optional `[plot]` extra for `matplotlib`.
- Optional `[dev]` extra with `pytest`, `ruff`, `mypy`.
- `[tool.ruff.lint]`, `[tool.mypy]` and `[tool.pytest.ini_options]` blocks.
- The unused `coolprop` dependency is gone.

## Migration notes

Because the old package still exists, you can migrate at your own pace:

| Old (`pyduct`)                            | New (`pyduct2`)                                |
|-------------------------------------------|------------------------------------------------|
| `RigidDuctType(shape="round", diameter=…)`| `Round(diameter=…)`                            |
| `RigidDuctType(shape="rectangular", …)`   | `Rectangular(width=…, height=…)`               |
| `RigidDuct(name, duct_type, length)`      | `RigidDuct(name, cross_section, length, …)`    |
| `OneWayFitting(name, flowrate=…)`         | `Terminal(name, flowrate=…)` *or* `Source(name)` |
| `TwoWayFitting(name, fitting_type)`       | `TwoPortFitting(name, cross_section, zeta)`    |
| `ThreeWayFitting(name)`                   | `Tee(name, cross_section, zeta_straight, zeta_branch)` |
| `elbow_round(...).dzeta` (a float)        | `ElbowRound(...).zeta` (a property)            |
| `Ductwork(...)` + manual graph editing    | `Network(...)` + `connect()`                   |
| `pass_flowrate_through_graph()`           | `propagate_flowrates(net)`                     |
| `calculate_pressure_drops()`              | `compute_pressure_drops(net, fluid)`           |
| `critical_path_pressure_drop()`           | `critical_path_pressure_drop(net)` or `solve(net)` |
| port indexed by `c.1`, `c.2`, `c.3`       | port named `combined`, `straight`, `branch`    |
| graph edges go terminal → AHU             | graph edges go AHU → terminal                  |

## What's not done yet

- Sizing-by-velocity / sizing-by-pressure-drop method on `Network` is not
  ported (the original `placeholder_calculate_dimmensions` was an empty
  stub anyway).
- Acoustic calculations: still future work as in the original README.
- A full library of fitting `zeta` correlations beyond `ElbowRound`.
- Reverse flow handling on terminals (still required to be `>= 0`).
