"""Python extension exposing the fused component-compute kernels."""

from std.os import abort
from std.python import Python, PythonObject
from std.python.bindings import PythonModuleBuilder

from wenta.components.compute import (
    duct_pressure_drop as _duct,
    fitting_pressure_drop as _fitting,
    flex_duct_pressure_drop as _flex,
)


def duct_pressure_drop(
    flowrate: PythonObject,
    area: PythonObject,
    hydraulic_diameter: PythonObject,
    length: PythonObject,
    absolute_roughness: PythonObject,
    kinematic_viscosity: PythonObject,
    density: PythonObject,
) raises -> PythonObject:
    var r = _duct(
        Float64(py=flowrate), Float64(py=area), Float64(py=hydraulic_diameter),
        Float64(py=length), Float64(py=absolute_roughness),
        Float64(py=kinematic_viscosity), Float64(py=density),
    )
    return Python.tuple(r[0], r[1])


def fitting_pressure_drop(
    flowrate: PythonObject, area: PythonObject,
    zeta: PythonObject, density: PythonObject,
) raises -> PythonObject:
    var r = _fitting(
        Float64(py=flowrate), Float64(py=area),
        Float64(py=zeta), Float64(py=density),
    )
    return Python.tuple(r[0], r[1])


def flex_duct_pressure_drop(
    flowrate: PythonObject, diameter: PythonObject, length: PythonObject,
    pressure_drop_per_meter: PythonObject, stretch_percentage: PythonObject,
) raises -> PythonObject:
    var r = _flex(
        Float64(py=flowrate), Float64(py=diameter), Float64(py=length),
        Float64(py=pressure_drop_per_meter), Float64(py=stretch_percentage),
    )
    return Python.tuple(r[0], r[1])


@export
def PyInit_compute_ext() -> PythonObject:
    try:
        var m = PythonModuleBuilder("compute_ext")
        m.def_function[duct_pressure_drop]("duct_pressure_drop")
        m.def_function[fitting_pressure_drop]("fitting_pressure_drop")
        m.def_function[flex_duct_pressure_drop]("flex_duct_pressure_drop")
        return m.finalize()
    except e:
        abort(String("failed to create compute_ext module: ", e))
        return PythonObject()
