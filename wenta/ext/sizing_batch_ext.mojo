"""Python extension: batch sizing kernels.

Sized over numpy ndarrays of flowrates with zero-copy raw-pointer access —
same pattern as ``compute_batch_ext``. For N flowrates the cost is one
Python↔Mojo boundary crossing total, regardless of N.
"""

from std.math import pi
from std.os import abort
from std.python import Python, PythonObject
from std.python.bindings import PythonModuleBuilder

from wenta.data.standard_sizes import _round_sizes_mm


def _f64_ptr(arr: PythonObject) raises -> UnsafePointer[Float64, MutExternalOrigin]:
    return UnsafePointer[Float64, MutExternalOrigin](
        unsafe_from_address=Int(py=arr.ctypes.data)
    )


def velocity_method_round_batch(
    flowrates: PythonObject,   # float64[N]
    diameters: PythonObject,   # float64[N], output
    velocities: PythonObject,  # float64[N], output
    target_velocity: PythonObject,
) raises -> PythonObject:
    """For each flowrate, pick the smallest EN-1506 size whose velocity ≤
    target. Writes diameters [m] and actual velocities [m/s] into the
    pre-allocated output buffers. Falls back to the largest size if none
    meets the target."""
    var n = Int(py=flowrates.shape[0])
    var f_ptr = _f64_ptr(flowrates)
    var d_ptr = _f64_ptr(diameters)
    var v_ptr = _f64_ptr(velocities)
    var target = Float64(py=target_velocity)

    var sizes = _round_sizes_mm()
    var m = len(sizes)

    # Pre-compute areas to avoid recomputing per-flowrate.
    var areas = List[Float64](length=m, fill=0.0)
    var diam_m = List[Float64](length=m, fill=0.0)
    for j in range(m):
        var d = Float64(sizes[j]) * 0.001
        diam_m[j] = d
        areas[j] = pi * d * d * 0.25

    for i in range(n):
        var q = f_ptr[i]
        if q <= 0.0:
            raise Error("flowrate must be positive")
        # Linear scan — 22 standard sizes, faster than a bisect dance.
        var chosen = m - 1
        for j in range(m):
            if q / areas[j] <= target:
                chosen = j
                break
        d_ptr[i] = diam_m[chosen]
        v_ptr[i] = q / areas[chosen]

    return Python.none()


@export
def PyInit_sizing_batch_ext() -> PythonObject:
    try:
        var mod = PythonModuleBuilder("sizing_batch_ext")
        mod.def_function[velocity_method_round_batch]("velocity_method_round_batch")
        return mod.finalize()
    except e:
        abort(String("failed to create sizing_batch_ext: ", e))
        return PythonObject()
