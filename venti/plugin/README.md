# Venti.Plugin — ZWCAD .NET plugin skeleton (issue #13)

A minimal ZWCAD (AutoCAD-compatible) .NET plugin around the `venti`
ductwork-design core. Right now it’s a **loadable scaffold**: it defines the
`VENTI` / `VENTI_SIZE` commands and the csproj that references the ZWCAD
managed API + stages `venti.wasm`. Real functionality lands over issues
**#14 → #16**.

## Layout

```
plugin/
├── Venti.Plugin.csproj   # net48 class library → Venti.Plugin.dll
└── src/
    ├── Commands.cs        # [CommandMethod] VENTI / VENTI_SIZE entry points
    ├── IVentiCore.cs      # host-agnostic facade (issue #14)
    ├── NativeCore.cs      # P/Invoke into the venti cdylib (issue #14)
    ├── WasmCore.cs        # embed venti.wasm via Wasmtime (issue #14)
    └── VentiCoreFactory.cs# backend selection (issue #14)
```

## Backends (issue #14)

`IVentiCore` abstracts the compute core so command code never touches the WASM
FDIn; pick a backend at runtime with `VentiCoreFactory.Create(...)`:

- **Native** (default) — `NativeCore` P/Invokes the C-ABI cdylib
  (`venti.dll` / `libventi.so`) with `[DllImport]`.
- **Wasm** — `WasmCore` embeds `venti.wasm` via the `Wasmtime` NuGet package
  (single cross-platform artifact; no native lib).

## Build

On a machine with ZWCAD installed (managed `ZwCAD.dll` ships in the ZWCAD
folder) and .NET Framework 4.8 tooling:

```bat
msbuild Venti.Plugin.csproj /p:Configuration=Release ^
       /p:ZwCADHome="C:\Program Files\ZWSOFT\ZWCAD 2026"
```

Output: `bin/Release/Venti.Plugin.dll` (with `venti.wasm` beside it, staged by
the csproj’s `None` item — the build already copies the WASM core from
`../../target/wasm32-wasip1/release/venti.wasm`; run `scripts/build-wasm.sh
--release` in the crate first, or pass an existing artifact).

## Load & test

1. Start ZWCAD.
2. Command `NETLOAD` → select `Venti.Plugin.dll`.
3. Command `VENTI` → prints the scaffold banner + backend.
4. Command `VENTI_SIZE` → prompts for a flowrate; sizes a round duct by the
   velocity method and reports D + velocity + friction factor (issue #16).
5. Command `VENTI_SOLVE` → solves a Source→Duct→Fitting→Terminal chain and
   reports critical-path static pressure (issue #16).

## Notes

- **Namespace convention:** the ZWCAD .NET API uses `ZwSoft.ZwCAD.*`
  (`ApplicationServices`, `EditorInput`, `Runtime.CommandMethodAttribute`),
  mirroring `Autodesk.AutoCAD.*`. The same source can be compiled for AutoCAD
  with a namespace alias under `#if ACAD`.
- The ZWCAD reference is resolved from `$(ZwCADHome)\ZwCAD.dll` and marked
  `Private=false` (loaded by the host).
- Follow-on issues: **#14** `IVentiCore` (WASM-binding interface), **#15**
  `VENTI_SIZE` real sizing, **#16** `VENTI_SOLVE`, **#17** staging script, **#18**
  binding unit tests.
