# VentiDuct — FreeCAD workbench for ductwork design (venti core)

A minimal FreeCAD extension that exposes the `venti` ductwork engine
(sizing, pressure drop, insulation, air balance) as workbench commands.
The math lives in the Rust core, reached through **either**:

- **WASM** (`bin/venti.wasm` + `wasmtime` Python package) — one cross-platform
  artifact, or
- **native cdylib** (`bin/libventi.so` / `bin/venti.dll` + `ctypes`) — no
  runtime dependency, per-OS build.

`venti_core.py` is pure Python (no FreeCAD import) and unit-testable standalone.

## Layout

```
Mod/VentiDuct/
├── __init__.py        # package + re-exports (importable without FreeCAD)
├── InitGui.py         # workbench registration (FreeCAD GUI)
├── commands.py        # VentiSize / VentiSolve / VentiInsulation / VentiTrace
├── venti_core.py      # backends: WasmCore (wasmtime) / NativeCore (ctypes) / CLI
├── bin/               # staged artifacts (gitignored): venti.wasm, libventi.so
└── stage_artifacts.*  # copies the built artifacts into bin/
```

## Install

1. **Build the artifacts** (from the `venti` crate root):
   ```bash
   ./scripts/build-wasm.sh --release          # -> target/wasm32-wasip1/release/venti.wasm
   cargo build --release --no-default-features --lib   # -> target/release/libventi.so
   ```
2. **Stage them** into the workbench:
   ```bash
   cd freecad/Mod/VentiDuct && ./stage_artifacts.sh /home/bart/github/pyduct/venti
   ```
3. **Copy the workbench** into FreeCAD's `Mod/` directory:
   - Linux: `~/.local/share/FreeCAD/Mod/VentiDuct` (or `$HOME/.FreeCAD/Mod`)
   - Windows: `%APPDATA%\FreeCAD\Mod\VentiDuct`
   - macOS: `~/Library/Preferences/FreeCAD/Mod`
   (or symlink the repo folder there).
4. **WASM backend (recommended):** install `wasmtime` into FreeCAD's Python:
   - Linux (system FreeCAD): `pip install wasmtime`
   - Windows / AppImage: `freecad_python -m pip install wasmtime --target <workbench>/libs`
   - If wasmtime is unavailable, the workbench falls back to the native cdylib.
5. Restart FreeCAD → Workbenches → **Venti Ductwork**.

## Commands

- **Size round duct (velocity)** — sizes a duct for the configured flowrate
  (default 0.1 m³/s @ 4 m/s; edit under
  `Tools → Edit parameters → User parameter:BaseApp/Preferences/Mod/VentiDuct`).
- **Solve example network** — solves a Source→Duct→Fitting→Terminal chain and
  reports the critical-path static pressure.
- **Insulation thickness (condensation)** — required duct insulation thickness
  for a cold-air duct (EN ISO 12241-style cylindrical model).
- **Trace sketch to duct network** — traces the selected object's edges
  (sketch / Shape edges) into a `venti` duct network via the host-agnostic
  topology module (`venti_topology_trace`): each edge becomes a polyline of
  (x, y) points, the polylines are coalesced into a network
  (Source / RigidDuct / Tee / Terminal), and the command prints the component
  count and critical-path ΔP to the console. The geometry-to-polyline
  conversion is a defensive skeleton (Line edges use their endpoints, curved
  edges are discretized when the API offers it); select an object with edges
  first, then run the command.

  The same tracing is available headless through `venti_core`:

  ```python
  from venti_core import get_core
  with get_core() as core:
      res = core.trace_network([[(0.0, 0.0), (5.0, 0.0)]])
      print(res.component_count(), res.solve())  # 3, 0.0 Pa
      res.free()
  ```

## Backend selection

`$VENTI_BACKEND` ∈ {`wasm`, `native`, `cli`}; default = auto (wasmtime → ctypes).

## Standalone self-test (no FreeCAD)

```bash
python venti_core.py
# backend: WasmCore
# friction_factor(5e4,9e-4) = 0.023645
# velocity_method_round(0.1,4.0) -> D = 0.2000 v = 3.1831
```
