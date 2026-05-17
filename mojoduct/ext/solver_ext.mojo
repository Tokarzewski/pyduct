"""Python extension: critical-path DP kernel on flat int/float lists.

Python passes ``topo`` (list[int]), ``preds`` (list[list[int]]), and
``dp`` (list[float]). Mojo converts each into native ``List`` once, runs
the DP, and returns a single Python float. One boundary crossing per
solve — not per math op — so the per-call overhead pays off.
"""

from std.os import abort
from std.python import Python, PythonObject
from std.python.bindings import PythonModuleBuilder

from mojoduct.network.solver import (
    critical_path_sum as _critical_path_sum,
    propagate_flows as _propagate_flows,
)


def _list_int_from_py(py: PythonObject) raises -> List[Int]:
    var out = List[Int]()
    for item in py:
        out.append(Int(py=item))
    return out^


def _list_list_int_from_py(py: PythonObject) raises -> List[List[Int]]:
    var out = List[List[Int]]()
    for inner in py:
        out.append(_list_int_from_py(inner))
    return out^


def _list_float_from_py(py: PythonObject) raises -> List[Float64]:
    var out = List[Float64]()
    for item in py:
        out.append(Float64(py=item))
    return out^


def critical_path_sum(
    topo: PythonObject, preds: PythonObject, dp: PythonObject
) raises -> PythonObject:
    return PythonObject(
        _critical_path_sum(
            _list_int_from_py(topo),
            _list_list_int_from_py(preds),
            _list_float_from_py(dp),
        )
    )


def propagate_flows(
    topo: PythonObject, preds: PythonObject, flows: PythonObject
) raises -> PythonObject:
    """Run the flow walk and return the resulting flows as a Python list."""
    var out = _propagate_flows(
        _list_int_from_py(topo),
        _list_list_int_from_py(preds),
        _list_float_from_py(flows),
    )
    var py_out: PythonObject = []
    for i in range(len(out)):
        py_out.append(out[i])
    return py_out^


@export
def PyInit_solver_ext() -> PythonObject:
    try:
        var m = PythonModuleBuilder("solver_ext")
        m.def_function[critical_path_sum]("critical_path_sum")
        m.def_function[propagate_flows]("propagate_flows")
        return m.finalize()
    except e:
        abort(String("failed to create solver_ext module: ", e))
        return PythonObject()
