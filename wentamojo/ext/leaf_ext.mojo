"""Python extension exposing miscellaneous leaf math functions to Python.

Collected here to avoid file proliferation: each Python module that uses
just one or two of these adds a thin shim importing from this single ext.
"""

from std.os import abort
from std.python import Python, PythonObject
from std.python.bindings import PythonModuleBuilder

from wentamojo.core.fluid import air_at_altitude as _air_at_altitude
from wentamojo.core.geometry import equivalent_round_diameter as _equivalent_round_diameter
from wentamojo.data.standard_sizes import nearest_round_size as _nearest_round_size


def air_at_altitude(
    altitude_m: PythonObject, temperature_c: PythonObject
) raises -> PythonObject:
    """Return ``(density, dynamic_viscosity)``; Python constructs the Fluid."""
    var f = _air_at_altitude(Float64(py=altitude_m), Float64(py=temperature_c))
    return Python.tuple(f.density, f.dynamic_viscosity)


def equivalent_round_diameter(
    width: PythonObject, height: PythonObject
) raises -> PythonObject:
    return PythonObject(
        _equivalent_round_diameter(Float64(py=width), Float64(py=height))
    )


def nearest_round_size(
    diameter_mm: PythonObject, round_up: PythonObject
) raises -> PythonObject:
    return PythonObject(
        _nearest_round_size(Float64(py=diameter_mm), round_up=Bool(py=round_up))
    )


@export
def PyInit_leaf_ext() -> PythonObject:
    try:
        var m = PythonModuleBuilder("leaf_ext")
        m.def_function[air_at_altitude]("air_at_altitude")
        m.def_function[equivalent_round_diameter]("equivalent_round_diameter")
        m.def_function[nearest_round_size]("nearest_round_size")
        return m.finalize()
    except e:
        abort(String("failed to create leaf_ext: ", e))
        return PythonObject()
