# Wenta → C# / ZWCAD Plugin — Roadmap

**Decision (2026-09): C# is the language of the stack.** Every CAD/BIM
plugin host — ZWCAD, AutoCAD, BricsCAD, Revit, Civil 3D — speaks .NET.
`wenta`/`venti` stop being *the product's runtime* and become **the math
oracle**: `python/wenta` (+`wentamojo`) stays the verified reference and
generates parity vectors; **all shipping code is C#**.

```
python/wenta + wentamojo (oracle, unchanged)     ← runs on Linux/CI only
        │  gen_vectors.py → CSV vectors (formula-transcribed, test-anchored)
        ▼
csharp/Wenta.Core (pure C#, net48, zero deps)    ← ALL engineering math
        │                                ▲
        ▼                                │
zwcad-plugin (WentaZwcad.dll, ZWCAD 2021) ── parity suite runs here
```

Ground rules:

- **No runtime Python/Mojo/Rust/WASM in the plugin.** One DLL (`Wenta.Core.dll`)
  next to one plugin DLL. No FFI, no cdylib staging, no Wasmtime.
- **Every engineering function lands in `Wenta.Core` with vectors first.**
  Vectors are transcribed from the canonical formula source (wentamojo,
  which is parity-tested against python/wenta) and spot-anchored to
  `python/tests` expectations. Full Python-oracle vector generation runs
  on CI (mojo wheels are Linux/macOS-only — the Windows dev box cannot run
  the oracle).
- C# never invents math: a new feature = new Mojo/Python kernels or
  vectors, then the C# port, then the parity suite goes green.
- Everything learned about ZWCAD 2021 (API DLLs, registry install, CUIX
  ribbon, evidence-log testing) stays the deployment layer — see README.

Competitive frame (verified against vendor pages + this session):

| | Wentyle (ALNOR, on ZWCAD) | Ventpack (Fluid Desk, BricsCAD) | **Wenta C# plugin** |
|---|---|---|---|
| Host | ZWCAD, bundled via ALNOR | BricsCAD + own platform | ZWCAD first |
| Math | basic ΔP | hydraulic + sound + reports | **oracle-tested, open (MIT)** |
| Fitting data | sponsored vendor libs (closed) | PartShelf24 (closed) | **open JSON catalog format** |
| BOM / estimating | KNR + vendor BOM | material schedules | BOM + fabrication + **KNR export** |
| Drawing UX | simple 2D | **best-in-class** (routing, sections) | dedicated phase below |
| 3D / clash | 3D MASTER add-on | 3D + HLR + clash | phase 7 |
| Multi-storey | partial | ✅ multi-drawing | **phase 6 (PL-market item)** |

---

## Phase 0 — Foundation (DONE, keep green)

✅ build pipeline (csc, x64), registry auto-load (`LOADCTRLS=14`),
`WENTAHELLO`/`WENTADUCT`/`WENTAPANEL`, palette, headless evidence-log
harness. 🔨 `Wenta.CUIX` ribbon package built — **MENULOAD test pending**.

## Phase 1 — `Wenta.Core` C# port (the stack conversion) ← CURRENT

Port the entire wenta library surface to `csharp/Wenta.Core`
(dependency-free, compiles with the same csc build chain, loads in ZWCAD):

| C# module | Source | Status |
|---|---|---|
| `Units` | units constants + ACH | port |
| `Fluid` (StandardAir, AirAtAltitude) | core/fluid | port |
| `Geometry` (Round, Rect, EquivalentRoundDiameter) | core/geometry | port |
| `Friction` (Re, Swamee–Jain, Colebrook) | physics/friction | port |
| `Losses`, `Flex` | physics/losses, flex | port |
| `StandardSizes` (EN 1505/1506 + nearest) | data/standard_sizes | port |
| `FittingsLibrary` (9 correlations) | components/fittings_library | port |
| `ElbowRound` (not-a-knot bicubic of Hendiger table) | components/elbow | port |
| `Components` (Port, Source, Terminal, RigidDuct, FlexDuct, TwoPortFitting, Tee) | components/* | port |
| `Network` + `Solver` (propagate, compute, critical-path DP) | network/* | port |
| `Sizing` (velocity, equal-friction, budget, NC, aspect-ratio) | sizing.py | port |
| `Schemas/Dtos` | schemas.py (pydantic ↔ C# DTOs) | port |
| **`Catalog`** (open ζ-catalog JSON, vendor merge) | new (venti FR-19 design) | port |
| **`Bom`** (bill of materials + KNR rows) | new (venti bom.rs design) | port |

> Phase 1 done: all 13 modules ported, **551/551 parity green** (vectors + inline catalog/bom/balancing/room runner).

Deliverables: `csharp/` tree, `tools/gen_vectors.py`, console parity
runner `Wenta.Core.Tests`, `build.cmd`, **parity green on this machine**.

## Phase 2 — Plugin becomes real (drawing ⇄ math) 

- `WENTADUCT` sizes via `Sizing` (flow prompt → velocity/EF/NC method →
  EN size → drawn polyline + label with flow/velocity/Δp/m; XData carries
  the full design record).
- `WENTASIZE` batch over selection sets; live re-size on edit (UX).
- `WENTAPRESSURE` — `Network`/`Solver` on traced drawing; critical-path ΔP
  reported per source.
- `WENTAREAD` / `WENTAWRITE` — wenta-JSON round-trip (pydantic-compatible
  DTOs), stable IDs in XData.

## Phase 3 — UX: the Ventpack-class drawing experience

Where Ventpack wins today; a full phase, not an afterthought:

1. **Continuous routing** — single/multi-run drawing with mid-run diameter
   + elevation changes, offset tracing from walls (XData drives it).
2. **Quick-connect** — auto-insert reducers/transitions/flexes/spacers on
   join (EN transformation tables from `StandardSizes`).
3. **Auto-network recognition** — trace polylines → `Network` (topology,
   tee detection) so pressure results come from geometry, not menus.
4. **Smart annotation** — branch numbering (`marking` semantics), size/
   flow/velocity labels as parametric blocks, auto-update on edit.
5. **Intelligent sections** — section view at any angle, section entities
   excluded from BOM (Phase 5).
6. Palette = modeless sizing form (live preview), ribbon = command
   surface; both persisted.

## Phase 4 — Beyond-wenta engineering (C# ports of venti's feature set)

Port, with vectors, the modules `venti` proved out — now in C#:
sound (`sound`), balancing (damper ζ/open-% — the "VentPack-style" one),
fan selection (curves, duty point), insulation (EN ISO 12241), room
balance/ACH, per-branch analysis, marking. Each: vectors → C# → plugin
command (`WENTASOUND`, `WENTABALANCE`, `WENTAFAN`, `WENTAInsulate`…).

| venti module | C# file | Status |
|---|---|---|
| **`Balancing`** (damper ζ/open-%, branch surplus-Δ logic) | `Balancing.cs` | port ✓ |
| **`Room`** (balance/ACH, RoomBalanceSet + CSV) | `Room.cs` | port ✓ |
| `Sound` | — | to port |
| `Fan` (curves, duty point) | — | to port |
| `Insulation` (EN ISO 12241) | — | to port |
| `Analysis` / `Marking` | — | to port |

## Phase 5 — PL-market items (item 3 of the competitive review)

1. **Open library feed** — the ζ-catalog JSON format (Phase 1 `Catalog`) +
   one shipped example catalog + published format spec; vendors/users
   contribute data without code (Wentyle's sponsored-library moat, opened).
2. **KNR-ready BOM** — `Bom` produces KNR-formatted estimate rows,
   per-vendor schedules, and dimensioned fabrication drawings (rect
   fittings) as drawing tables + CSV/XLSX export.
3. **Multi-drawing / multi-storey projects** — drawing-scope IDs in the
   network DTOs (schema designed in Phase 2), cross-drawing connection
   registry, storey manager palette; a network spans drawings via
   stable-GUID links.

## Phase 6 — Distribution & quality

WiX MSI (DLL + CUIX + registry + example catalog), semver, CI on a
Windows+ZWCAD2021 runner (evidence-log assert + screenshot diff), crash
discipline (no unhandled exception in ZWCAD, `wenta.log`), docs EN/PL,
`WENTAHELP`.

## Phase 7 — 3D, clash, IFC (full-suite scope)

Elevation → solids/fittings 3D, sections/isometrics, clash check,
IFC export (C# toolkit — no Python side), BIM views. After M3.

---

## Milestones

| | Phases | Outcome |
|---|---|---|
| M1 | 0+1 | full wenta math in C#, parity green, ribbon installed |
| M2 | 2 | a drawing sized/pressurized/round-tripped by the plugin |
| M3 | 3 | Ventpack-class drawing UX demo on a floor plan |
| M4 | 4+5 | sound/balancing/fan/insulation + KNR BOM + catalog feed + multi-storey |
| M5 | 6+7 | installer + CI; 3D/IFC scope |

## Non-goals (explicit)

- Runtime Python/Rust/WASM anywhere in the shipped plugin.
- Certified vendor loss data (we ship the *format*).
- Native Revit integration (later, cheaply — C# API is shared).
- Linux support (the oracle lives there; the product lives in ZWCAD).
