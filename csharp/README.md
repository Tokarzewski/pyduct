# Wenta.Core — the C# port of the wenta library

C# is the language of the CAD/BIM plugin stack (ZWCAD, AutoCAD, Revit are all
.NET hosts). `Wenta.Core` is the complete port of the `wenta` ductwork
library — sizing, friction, fittings, network solver, catalogs, BOM —
dependency-free, compiled with the bare-csc toolchain, loadable in ZWCAD.

`python/wenta` + `wentamojo` remain the **math oracle** (unchanged); this
port is verified against it by parity vectors.

## Layout

```
Wenta.Core/           pure C# port (compiles standalone, net48-compatible)
  Units.cs Fluid.cs Geometry.cs Physics.cs StandardSizes.cs
  FittingsLibrary.cs Elbow.cs Components.cs Network.cs Solver.cs
  Sizing.cs Catalog.cs Bom.cs
Wenta.Core.Tests/     console parity runner (Program.cs, no xUnit needed)
vectors/             CSV parity vectors (generated, see below)
tools/gen_vectors.py vector generator (transcribes the wentamojo kernels)
tools/.venv          scipy for the ElbowRound spline ground truth
catalogs/            example-generic.json — open zeta-catalog format
build.cmd            builds core + tests, copies vectors
```

## Build & parity

```cmd
build.cmd
bin\Wenta.Core.Tests.exe        # => "==== 551 passed, 0 failed ===="
```

Regenerate vectors (after changing wentamojo/wenta math):

```cmd
cd tools
.venv\Scripts\python.exe gen_vectors.py
```

## How the vectors stay honest

- The Windows dev box **cannot run the oracle** — `mojo==0.26.2` ships no
  Windows wheels, and wenta's Python math is a shim over it. The generator
  therefore transcribes the canonical formula source (`wentamojo`, which is
  itself parity-tested against `python/wenta` on Linux/CI) and spot-anchors
  to `python/tests` expectations. The elbow spline ground truth is scipy
  (`RectBivariateSpline`), which is exactly what `wenta.components.elbow`
  uses in Python — it runs natively on Windows.
- Tolerances: 1e-12 relative everywhere; elbow grid points 1e-9 (spline
  passes through knots), elbow intermediate points 2e-4 (separable
  not-a-knot bicubic vs FITPACK).
- Full Python-oracle vector generation should run on **CI (Linux)** where
  mojo wheels exist — `gen_vectors.py` then becomes a thin `import wenta`
  instead of transcriptions (same CSV schema).

## Beyond the wenta reference surface

Two competitive-scope modules land here first:

- **`Catalog.cs`** — the open ζ-catalog format (JSON): pluggable
  manufacturer loss tables with provenance and KNR codes, vendor-mergeable.
  The open answer to Wentyle's sponsored libraries / Ventpack's PartShelf24.
- **`Bom.cs`** — bill of materials with KNR-ready rows from a solved
  network (lengths, areas, per-item KNR estimate codes).

The `venti` (Rust) sibling implements a wider feature set (sound,
balancing, fans, insulation, rooms, fabrication) — port those here
module-by-module with the same vector methodology (see
`zwcad-plugin/ROADMAP.md`, Phase 4).
