"""Python extension: batch compute_pressure_drops kernel — numpy edition.

Inputs and outputs are numpy ``float64`` / ``int64`` ndarrays. The Mojo
side reads/writes their raw memory via ``UnsafePointer.unsafe_from_address``
— zero-copy across the Python↔Mojo boundary, no per-element PyFloat
boxing. The earlier list-based draft hit a ~2× solver regression because
of that boxing; this rewrite is the cure.
"""

from std.math import exp
from std.os import abort
from std.python import Python, PythonObject
from std.python.bindings import PythonModuleBuilder

from wenta.physics.friction import friction_factor, relative_roughness, reynolds


comptime TAG_SOURCE = 0
comptime TAG_TERMINAL = 1
comptime TAG_RIGID = 2
comptime TAG_FLEX = 3
comptime TAG_FITTING = 4
comptime TAG_TEE = 5


def _f64_ptr(arr: PythonObject) raises -> UnsafePointer[Float64, MutExternalOrigin]:
    var addr = Int(py=arr.ctypes.data)
    return UnsafePointer[Float64, MutExternalOrigin](unsafe_from_address=addr)


def _i64_ptr(arr: PythonObject) raises -> UnsafePointer[Int64, MutExternalOrigin]:
    var addr = Int(py=arr.ctypes.data)
    return UnsafePointer[Int64, MutExternalOrigin](unsafe_from_address=addr)


def batch_compute(
    types: PythonObject,        # int64[N]
    params: PythonObject,       # float64[6*N]
    port_idx: PythonObject,     # int64[3*N]
    flows: PythonObject,        # float64[P], pre-populated
    velocities: PythonObject,   # float64[P], output
    dps: PythonObject,          # float64[P], output
    density: PythonObject,
    kinematic_viscosity: PythonObject,
) raises -> PythonObject:
    var n = Int(py=types.shape[0])
    var t_ptr = _i64_ptr(types)
    var p_ptr = _f64_ptr(params)
    var i_ptr = _i64_ptr(port_idx)
    var f_ptr = _f64_ptr(flows)
    var v_ptr = _f64_ptr(velocities)
    var d_ptr = _f64_ptr(dps)
    var rho = Float64(py=density)
    var nu = Float64(py=kinematic_viscosity)

    for i in range(n):
        var tag = Int(t_ptr[i])
        var p0 = p_ptr[i * 6 + 0]
        var p1 = p_ptr[i * 6 + 1]
        var p2 = p_ptr[i * 6 + 2]
        var p3 = p_ptr[i * 6 + 3]
        var p4 = p_ptr[i * 6 + 4]
        var ix0 = Int(i_ptr[i * 3 + 0])
        var ix1 = Int(i_ptr[i * 3 + 1])
        var ix2 = Int(i_ptr[i * 3 + 2])

        if tag == TAG_SOURCE:
            pass
        elif tag == TAG_TERMINAL:
            if p0 > 0.0:
                var v = f_ptr[ix0] / p0
                v_ptr[ix0] = v
                d_ptr[ix0] = p1 * rho * v * v * 0.5
        elif tag == TAG_RIGID:
            var v = f_ptr[ix0] / p0
            var re = reynolds(v, p1, nu)
            var eps = relative_roughness(p3, p1)
            var f = friction_factor(re, eps)
            v_ptr[ix0] = v
            v_ptr[ix1] = v
            d_ptr[ix0] = f * (p2 / p1) * rho * v * v * 0.5
        elif tag == TAG_FLEX:
            var v = f_ptr[ix0] / p0
            var beta = 0.557 * (100.0 - p4) * exp(-4.93 * p1) + 1.0
            v_ptr[ix0] = v
            v_ptr[ix1] = v
            d_ptr[ix0] = p3 * p2 * beta
        elif tag == TAG_FITTING:
            var v = f_ptr[ix0] / p0
            v_ptr[ix0] = v
            v_ptr[ix1] = v
            d_ptr[ix1] = p1 * rho * v * v * 0.5
        elif tag == TAG_TEE:
            var inv_a = 1.0 / p0
            var v_s = f_ptr[ix1] * inv_a
            var v_b = f_ptr[ix2] * inv_a
            var v_c = f_ptr[ix0] * inv_a
            v_ptr[ix0] = v_c
            v_ptr[ix1] = v_s
            v_ptr[ix2] = v_b
            d_ptr[ix1] = p1 * rho * v_s * v_s * 0.5
            d_ptr[ix2] = p2 * rho * v_b * v_b * 0.5
    return Python.none()


@export
def PyInit_compute_batch_ext() -> PythonObject:
    try:
        var m = PythonModuleBuilder("compute_batch_ext")
        m.def_function[batch_compute]("batch_compute")
        return m.finalize()
    except e:
        abort(String("failed to create compute_batch_ext module: ", e))
        return PythonObject()
