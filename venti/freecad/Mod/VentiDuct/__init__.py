# VentiDuct — a FreeCAD workbench for ductwork design powered by `venti`.
#
# The computational core is the Rust `venti` library, reached through the
# WASM core (venti.wasm + wasmtime) or the native cdylib (libventi.so /
# venti.dll). This package is importable without FreeCAD (venti_core.py is
# pure Python) so it can be unit-tested standalone.

from .venti_core import VentiCore, WasmCore, NativeCore, TraceResult, get_core

__all__ = ["VentiCore", "WasmCore", "NativeCore", "TraceResult", "get_core"]
__version__ = "0.1.0"