"""Python extension: batch compute_pressure_drops kernel."""

from std.os import abort
from std.python import Python, PythonObject
from std.python.bindings import PythonModuleBuilder

from wenta.network.compute_batch import batch_compute as _batch_compute


def _list_int(py: PythonObject) raises -> List[Int]:
    var out = List[Int]()
    for item in py:
        out.append(Int(py=item))
    return out^


def _list_float(py: PythonObject) raises -> List[Float64]:
    var out = List[Float64]()
    for item in py:
        out.append(Float64(py=item))
    return out^


def batch_compute(
    types: PythonObject,
    params: PythonObject,
    port_idx: PythonObject,
    flows: PythonObject,
    density: PythonObject,
    kinematic_viscosity: PythonObject,
) raises -> PythonObject:
    """Return ``(velocities, dps)`` as a pair of Python lists, length = P."""
    var r = _batch_compute(
        _list_int(types),
        _list_float(params),
        _list_int(port_idx),
        _list_float(flows),
        Float64(py=density),
        Float64(py=kinematic_viscosity),
    )
    var n = len(r[0])
    var py_v: PythonObject = []
    var py_d: PythonObject = []
    for i in range(n):
        py_v.append(r[0][i])
        py_d.append(r[1][i])
    return Python.tuple(py_v, py_d)


@export
def PyInit_compute_batch_ext() -> PythonObject:
    try:
        var m = PythonModuleBuilder("compute_batch_ext")
        m.def_function[batch_compute]("batch_compute")
        return m.finalize()
    except e:
        abort(String("failed to create compute_batch_ext module: ", e))
        return PythonObject()
