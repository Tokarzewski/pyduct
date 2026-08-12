# venti_core — pure-Python facade over the venti computational core.
#
# No FreeCAD import here, so this module is unit-testable standalone. It
# exposes a stable Python API over one of three backends:
#   1. wasmtime  — embeds venti.wasm (single cross-platform artifact)
#   2. ctypes    — P/Invokes the native libventi.so / venti.dll cdylib
#   3. cli       — shells out to the `wasmtime run` CLI (JSON) [fallback]
#
# Backend selection: $VENTI_BACKEND in {"wasm","native","cli"} overrides;
# otherwise try wasmtime, then ctypes.

import ctypes
import json
import os
import struct
import subprocess
import sys
from pathlib import Path

_HERE = Path(__file__).resolve().parent
_BIN = _HERE / "bin"


# ---------------------------------------------------------------------------
# Stable Python API surface (same numbers as the Rust/Node/ctypes references)
# ---------------------------------------------------------------------------

class VentiCore:
    """Common interface: each backend implements these methods."""

    def friction_factor(self, re, rel_roughness):
        raise NotImplementedError

    def velocity_method_round(self, flowrate, target_velocity=4.0):
        """Return (diameter_m, velocity_m_s)."""
        raise NotImplementedError

    def equal_friction_method_round(self, flowrate, target_pa_per_m=1.0,
                                    roughness=0.0001, density=1.204,
                                    viscosity=1.825e-5):
        raise NotImplementedError

    def elbow_round_loss(self, bend_radius, diameter, angle_deg, velocity,
                         density=1.204, viscosity=1.825e-5):
        raise NotImplementedError

    def regenerated_noise_round(self, velocity, diameter, density=1.204):
        raise NotImplementedError

    def required_zeta(self, dp_pa, velocity, density=1.204):
        raise NotImplementedError

    def damper_open_percentage(self, zeta):
        raise NotImplementedError

    def insulation_condensation(self, air_c, dew_c, amb_c, conductivity,
                                 d_m, hi=10.0, he=8.0):
        raise NotImplementedError

    def close(self):
        pass


# ---------------------------------------------------------------------------
# WASM backend (wasmtime)
# ---------------------------------------------------------------------------

class WasmCore(VentiCore):
    def __init__(self, wasm_path=None):
        import wasmtime
        self._wasmtime = wasmtime
        wasm_path = wasm_path or _BIN / "venti.wasm"
        if not Path(wasm_path).exists():
            raise FileNotFoundError(f"venti.wasm not found at {wasm_path}")
        self._engine = wasmtime.Engine()
        self._store = wasmtime.Store(self._engine)
        wasi = wasmtime.WasiConfig()
        wasi.inherit_stdout()
        self._store.set_wasi(wasi)
        module = wasmtime.Module.from_file(self._engine, str(wasm_path))
        linker = wasmtime.Linker(self._engine)
        linker.define_wasi()
        self._inst = linker.instantiate(self._store, module)
        self._exports = self._get_exports()

    def _get_exports(self):
        e = self._inst.exports
        if callable(e):
            try:
                return e(self._store)
            except TypeError:
                try:
                    return e()
                except TypeError:
                    return e
        return e

    def _call(self, name, *args):
        fn = self._exports[name]
        try:
            return fn(self._store, *args)
        except TypeError:
            return fn(*args)

    def _alloc(self, nbytes):
        return self._call("venti_alloc", nbytes)

    def _free(self, ptr, nbytes):
        if "venti_free" in self._exports:
            try:
                self._call("venti_free", ptr, nbytes)
            except Exception:
                pass

    def _mem(self):
        return self._exports["memory"]

    def _mem_write(self, ptr, data):
        mem = self._mem()
        try:
            mem.write(self._store, data, ptr)
        except TypeError:
            mem.write(data, ptr)

    def _mem_read(self, ptr, size):
        mem = self._mem()
        try:
            return bytes(mem.read(self._store, ptr, ptr + size))
        except TypeError:
            return bytes(mem.read(ptr, ptr + size))

    # ---- scalar exports ----
    def friction_factor(self, re, rel_roughness):
        return self._call("venti_friction_factor", re, rel_roughness)

    def elbow_round_loss(self, bend_radius, diameter, angle_deg, velocity,
                         density=1.204, viscosity=1.825e-5):
        return self._call("venti_elbow_round_loss", bend_radius, diameter,
                          angle_deg, velocity, density, viscosity)

    def regenerated_noise_round(self, velocity, diameter, density=1.204):
        return self._call("venti_regenerated_noise_round", velocity, diameter, density)

    def required_zeta(self, dp_pa, velocity, density=1.204):
        return self._call("venti_required_zeta", dp_pa, velocity, density)

    def damper_open_percentage(self, zeta):
        return self._call("venti_damper_open_percentage", zeta)

    def insulation_condensation(self, air_c, dew_c, amb_c, conductivity,
                                d_m, hi=10.0, he=8.0):
        # venti::insulation is not (yet) on the C ABI, so compute the
        # condensation thickness directly in Python (same cylindrical model).
        import math
        step = 0.001
        t = 0.0
        while t <= 0.25:
            t += step
            do = d_m + 2 * t
            r_in = 1 / (hi * math.pi * d_m)
            r_ins = math.log(do / d_m) / (2 * math.pi * conductivity)
            r_out = 1 / (he * math.pi * do)
            q = (air_c - amb_c) / (r_in + r_ins + r_out)
            ts = amb_c + q * r_out
            if ts >= dew_c:
                return t
        return None

    # ---- out-param sizing ----
    def velocity_method_round(self, flowrate, target_velocity=4.0):
        ptr = self._alloc(2 * 8)
        try:
            st = self._call("venti_velocity_method_round", flowrate,
                            target_velocity, ptr, ptr + 8)
            if st != 0:
                raise ValueError(f"venti sizing failed (status {st})")
            d, v = struct.unpack("2d", self._mem_read(ptr, 16))
            return d, v
        finally:
            self._free(ptr, 2 * 8)

    def equal_friction_method_round(self, flowrate, target_pa_per_m=1.0,
                                    roughness=0.0001, density=1.204,
                                    viscosity=1.825e-5):
        ptr = self._alloc(3 * 8)
        try:
            st = self._call("venti_equal_friction_method_round", flowrate,
                            target_pa_per_m, roughness, density, viscosity,
                            ptr, ptr + 8, ptr + 16)
            if st != 0:
                raise ValueError(f"venti sizing failed (status {st})")
            d, v, dp_m = struct.unpack("3d", self._mem_read(ptr, 24))
            return d, v, dp_m
        finally:
            self._free(ptr, 3 * 8)

    def close(self):
        try:
            self._store.close()
            self._engine.close()
        except Exception:
            pass


# ---------------------------------------------------------------------------
# Native backend (ctypes)
# ---------------------------------------------------------------------------

def _find_native_lib():
    candidates = [
        _BIN / "libventi.so",
        _BIN / "venti.dll",
        _BIN / "libventi.dylib",
        Path(os.environ.get("VENTI_LIB_DIR", "")) / "libventi.so",
    ]
    for c in candidates:
        if Path(c).exists():
            return str(c)
    # let the OS search (venti.dll / libventi.so on PATH)
    return "venti"


class NativeCore(VentiCore):
    def __init__(self, lib_path=None):
        self._lib = ctypes.CDLL(lib_path or _find_native_lib())

        def _f64(name, nargs):
            fn = getattr(self._lib, name)
            fn.restype = ctypes.c_double
            fn.argtypes = [ctypes.c_double] * nargs
            return fn

        self._f = {
            "friction_factor": _f64("venti_friction_factor", 2),
            "elbow_round_loss": _f64("venti_elbow_round_loss", 6),
            "regenerated_noise_round": _f64("venti_regenerated_noise_round", 3),
            "required_zeta": _f64("venti_required_zeta", 3),
            "damper_open_percentage": _f64("venti_damper_open_percentage", 1),
        }
        self._v = self._lib.venti_velocity_method_round
        self._v.restype = ctypes.c_int
        self._v.argtypes = [ctypes.c_double, ctypes.c_double,
                            ctypes.POINTER(ctypes.c_double),
                            ctypes.POINTER(ctypes.c_double)]
        self._ef = self._lib.venti_equal_friction_method_round
        self._ef.restype = ctypes.c_int
        self._ef.argtypes = [ctypes.c_double, ctypes.c_double, ctypes.c_double,
                             ctypes.c_double, ctypes.c_double,
                             ctypes.POINTER(ctypes.c_double),
                             ctypes.POINTER(ctypes.c_double),
                             ctypes.POINTER(ctypes.c_double)]

    def friction_factor(self, re, rel_roughness):
        return self._f["friction_factor"](re, rel_roughness)

    def elbow_round_loss(self, bend_radius, diameter, angle_deg, velocity,
                         density=1.204, viscosity=1.825e-5):
        return self._f["elbow_round_loss"](bend_radius, diameter, angle_deg,
                                           velocity, density, viscosity)

    def regenerated_noise_round(self, velocity, diameter, density=1.204):
        return self._f["regenerated_noise_round"](velocity, diameter, density)

    def required_zeta(self, dp_pa, velocity, density=1.204):
        return self._f["required_zeta"](dp_pa, velocity, density)

    def damper_open_percentage(self, zeta):
        return self._f["damper_open_percentage"](zeta)

    def insulation_condensation(self, air_c, dew_c, amb_c, conductivity,
                                d_m, hi=10.0, he=8.0):
        # same Python model as WasmCore (module not on C ABI yet)
        import math
        step = 0.001
        t = 0.0
        while t <= 0.25:
            t += step
            do = d_m + 2 * t
            r_in = 1 / (hi * math.pi * d_m)
            r_ins = math.log(do / d_m) / (2 * math.pi * conductivity)
            r_out = 1 / (he * math.pi * do)
            q = (air_c - amb_c) / (r_in + r_ins + r_out)
            ts = amb_c + q * r_out
            if ts >= dew_c:
                return t
        return None

    def velocity_method_round(self, flowrate, target_velocity=4.0):
        d = ctypes.c_double()
        v = ctypes.c_double()
        if self._v(flowrate, target_velocity, ctypes.byref(d), ctypes.byref(v)) != 0:
            raise ValueError("venti sizing failed")
        return d.value, v.value

    def equal_friction_method_round(self, flowrate, target_pa_per_m=1.0,
                                    roughness=0.0001, density=1.204,
                                    viscosity=1.825e-5):
        d = ctypes.c_double(); v = ctypes.c_double(); dp = ctypes.c_double()
        if self._ef(flowrate, target_pa_per_m, roughness, density, viscosity,
                    ctypes.byref(d), ctypes.byref(v), ctypes.byref(dp)) != 0:
            raise ValueError("venti sizing failed")
        return d.value, v.value, dp.value


# ---------------------------------------------------------------------------
# Factory
# ---------------------------------------------------------------------------

def get_core(backend=None):
    """Return a VentiCore for the requested backend.

    backend: "wasm" | "native" | "cli" | None (auto). $VENTI_BACKEND overrides
    the default when backend is None.
    """
    backend = backend or os.environ.get("VENTI_BACKEND", "auto")
    if backend == "wasm":
        return WasmCore()
    if backend == "native":
        return NativeCore()
    if backend == "cli":
        return _CliCore()
    # auto
    try:
        return WasmCore()
    except Exception:
        try:
            return NativeCore()
        except Exception as exc:
            raise RuntimeError(
                "no venti backend available (need venti.wasm + wasmtime, or "
                "libventi.so/dll): %s" % exc
            ) from exc


class _CliCore(VentiCore):
    """Subprocess fallback: wasmtime run <venti.wasm> --invoke <fn> ..."""

    def __init__(self, wasm_path=None):
        self._wasm = str(wasm_path or _BIN / "venti.wasm")
        if not Path(self._wasm).exists():
            raise FileNotFoundError(self._wasm)

    def _invoke(self, name, args):
        cmd = ["wasmtime", "run", "--invoke", name, self._wasm] + [str(a) for a in args]
        out = subprocess.run(cmd, capture_output=True, text=True, check=True)
        text = out.stdout.strip()
        return float(text.split()[-1])

    def friction_factor(self, re, rel_roughness):
        return self._invoke("venti_friction_factor", [re, rel_roughness])

    def velocity_method_round(self, flowrate, target_velocity=4.0):
        # CLI prints the f64 result of a single export; sizing uses out-params,
        # which the CLI invocation can't marshal — document as unsupported.
        raise NotImplementedError("cli backend supports scalar exports only")

    def required_zeta(self, dp_pa, velocity, density=1.204):
        return self._invoke("venti_required_zeta", [dp_pa, velocity, density])

    def damper_open_percentage(self, zeta):
        return self._invoke("venti_damper_open_percentage", [zeta])


if __name__ == "__main__":
    # quick self-test (usable without FreeCAD)
    core = get_core()
    print("backend:", type(core).__name__)
    print("friction_factor(5e4,9e-4) =", round(core.friction_factor(50000.0, 0.0009), 6))
    d, v = core.velocity_method_round(0.1, 4.0)
    print("velocity_method_round(0.1,4.0) -> D =", round(d, 4), "v =", round(v, 4))
    print("elbow_round_loss =", round(core.elbow_round_loss(0.2, 0.2, 90.0, 4.0), 4))
    print("insulation_condensation =", round(core.insulation_condensation(8.0, 15.8, 24.0, 0.035, 0.2), 4), "m")
    core.close()
