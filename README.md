# pyduct

Open-source Python library for ductwork design: sizing, pressure-drop calculations,
fitting losses, and network solving. Intended for building-services engineers.

## Features

- **Duct sizing** — velocity method, equal-friction method, pressure-drop budget.
- **Fitting library** — round reducers/expanders, tee junctions, dampers, diffusers, grilles.
- **Network solver** — graph-based, pure-function pipeline with a true pressure-weighted
  critical path.
- **Round & rectangular** cross sections with EN 1505/1506 standard sizes.
- **Pydantic schemas** for validated YAML/JSON network I/O.
- **Results export** — pretty tables, CSV, dicts.
- **Visualization** — graph drawing with critical-path highlighting (optional).

## Installation

```bash
pip install -e .                # core
pip install -e ".[plot]"        # + matplotlib visualization
pip install -e ".[yaml]"        # + YAML I/O
pip install -e ".[dev]"         # testing, mypy, ruff, plot, yaml
```

Requires Python 3.10+.

## Quick start

```python
from pyduct import (
    Network, Source, RigidDuct, Terminal, Round, solve,
    velocity_method, results_summary,
)

section, v = velocity_method(0.1, "round", target_velocity=4.0)
print(f"Sized to {section.diameter:.3f} m, velocity {v:.2f} m/s")

net = Network("example")
net.add("ahu",  Source("AHU"))
net.add("duct", RigidDuct("duct", section, length=20))
net.add("term", Terminal("terminal", flowrate=0.1))
net.connect("ahu", "duct")
net.connect("duct", "term")

dp = solve(net)
print(results_summary(net))
```

More examples in `pyduct/examples/`:

```bash
python -m pyduct.examples.small_supply
python -m pyduct.examples.complete_design
python -m pyduct.examples.load_network_from_yaml
```

## Layout

```
pyduct/
├── core/              Fluid, geometry (Round, Rectangular)
├── physics/           Friction & loss correlations
├── components/        Duct, terminal, fittings, elbow library
├── data/              EN standard sizes
├── network/           Network model & pure-function solver
├── sizing.py          Duct sizing methods
├── schemas.py         Pydantic validation schemas
├── io.py              YAML/JSON serialization
├── results.py         Result extraction & export
├── visualization.py   Optional matplotlib plotting
└── examples/
```

## Development

```bash
just check          # pytest
just lint           # ruff
uv run mypy pyduct  # type-check
```

## Design docs

Background and history of the redesign live in [`docs/`](docs/):

- `docs/redesign-notes.md`
- `docs/implementation-summary.md`
- `docs/pydantic-extension.md`

## Bibliography

- ASHRAE Handbook — Fundamentals
- Hendiger, Ziętek, Chludzińska: *Wentylacja i Klimatyzacja — Materiały pomocniczne do projektowania*
- Swamee & Jain (1976): *Explicit equations for pipe-flow problems*
- Colebrook–White equation (friction factor correlation)

## License

MIT
