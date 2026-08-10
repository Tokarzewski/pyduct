#!/usr/bin/env python3
"""Embed the venti host `cdylib` from CPython via ctypes.

Usage (after `cargo build --release --no-default-features --lib`):

    python host/cdylib_python_example.py

The cdylib exposes the same `venti_*` C ABI as the WASM core, so the same
signatures work from C# P/Invoke.
"""
import ctypes
import sys
from pathlib import Path

LIB = Path(__file__).resolve().parent.parent / "target" / "release" / "libventi.so"


def load():
    if not LIB.exists():
        sys.exit(f"missing {LIB}; run: cargo build --release --no-default-features --lib")
    lib = ctypes.CDLL(str(LIB))
    lib.venti_friction_factor.argtypes = [ctypes.c_double, ctypes.c_double]
    lib.venti_friction_factor.restype = ctypes.c_double
    lib.venti_standard_air_density.restype = ctypes.c_double
    lib.venti_local_pressure_drop.argtypes = [ctypes.c_double, ctypes.c_double, ctypes.c_double]
    lib.venti_local_pressure_drop.restype = ctypes.c_double

    # Out-param sizing functions
    lib.venti_velocity_method_round.argtypes = [
        ctypes.c_double, ctypes.c_double,
        ctypes.POINTER(ctypes.c_double), ctypes.POINTER(ctypes.c_double),
    ]
    lib.venti_velocity_method_round.restype = ctypes.c_int
    return lib


def main():
    lib = load()
    print(f"friction_factor(5e4, 9e-4) = {lib.venti_friction_factor(50000, 0.0009):.6f}")
    print(f"local_pressure_drop(1,4,1.204) = {lib.venti_local_pressure_drop(1.0, 4.0, 1.204):.3f}")
    print(f"standard_air_density = {lib.venti_standard_air_density()}")

    d = ctypes.c_double(); v = ctypes.c_double()
    st = lib.venti_velocity_method_round(0.1, 4.0, ctypes.byref(d), ctypes.byref(v))
    print(f"velocity_method_round(0.1,4.0) -> status {st} | D = {d.value:.4f} m | v = {v.value:.4f} m/s")

    # Network via the handle API (same surface as the WASM core).
    def s(x):
        b = ctypes.create_string_buffer(x.encode())
        return ctypes.cast(b, ctypes.c_void_p), len(x)

    nptr, nlen = s("Supply")
    lib.venti_network_create.argtypes = [ctypes.c_void_p, ctypes.c_size_t]
    lib.venti_network_create.restype = ctypes.c_int
    lib.venti_network_add.argtypes = [ctypes.c_int, ctypes.c_void_p, ctypes.c_size_t,
                                      ctypes.c_void_p, ctypes.c_size_t, ctypes.c_int,
                                      ctypes.POINTER(ctypes.c_double)]
    lib.venti_network_add.restype = ctypes.c_int
    lib.venti_network_connect.argtypes = [ctypes.c_int, ctypes.c_void_p, ctypes.c_size_t,
                                          ctypes.c_void_p, ctypes.c_size_t]
    lib.venti_network_connect.restype = ctypes.c_int
    lib.venti_network_solve.argtypes = [ctypes.c_int, ctypes.c_double, ctypes.c_double]
    lib.venti_network_solve.restype = ctypes.c_double
    lib.venti_results_count.argtypes = [ctypes.c_int]
    lib.venti_results_count.restype = ctypes.c_int
    lib.venti_results_row.argtypes = [ctypes.c_int, ctypes.c_int,
                                      ctypes.POINTER(ctypes.c_double), ctypes.POINTER(ctypes.c_int),
                                      ctypes.POINTER(ctypes.c_double), ctypes.POINTER(ctypes.c_int),
                                      ctypes.POINTER(ctypes.c_double)]
    lib.venti_results_row.restype = ctypes.c_int
    lib.venti_network_free.argtypes = [ctypes.c_int]
    lib.venti_network_free.restype = ctypes.c_int
    net = lib.venti_network_create(nptr, nlen)

    comps = [
        ("ahu",  "AHU",        0, (0, 0, 0, 0, 0, 0)),
        ("duct", "Main Duct",  2, (3.1416e-2, 0.2, 20.0, 0.0001, 0, 0)),
        ("fit",  "Elbow",      4, (3.1416e-2, 0.5, 0, 0, 0, 0)),
        ("term", "Terminal",   1, (3.1416e-2, 1.0, 0.1, 0, 0, 0)),
    ]
    for cid, cname, ctype, prms in comps:
        ip_, il = s(cid); cm_, cl = s(cname)
        parr = (ctypes.c_double * 6)(*prms)
        rc = lib.venti_network_add(net, ip_, il, cm_, cl, ctype, parr)
        assert rc == 0, f"add {cid} rc={rc}"
    for a, b in [("ahu", "duct"), ("duct", "fit"), ("fit", "term")]:
        ap_, al = s(a); bp_, bl = s(b)
        rc = lib.venti_network_connect(net, ap_, al, bp_, bl)
        assert rc == 0, f"connect {a}->{b} rc={rc}"

    dp = lib.venti_network_solve(net, 1.204, 1.825e-5)
    print(f"network critical-path DP = {dp:.3f} Pa")

    nrows = lib.venti_results_count(net)
    print(f"results rows: {nrows}")
    for i in range(nrows):
        fin = ctypes.c_double(); fset = ctypes.c_int()
        vin = ctypes.c_double(); vset = ctypes.c_int()
        dp_ = ctypes.c_double()
        lib.venti_results_row(net, i, ctypes.byref(fin), ctypes.byref(fset),
                              ctypes.byref(vin), ctypes.byref(vset), ctypes.byref(dp_))
        print(f"  row {i}: Q_in set={fset.value} value={fin.value:.3f} | DP={dp_.value:.3f}")
    lib.venti_network_free(net)


if __name__ == "__main__":
    main()
