"""Python extension exposing `wentamojo.physics.losses` to Python."""

from std.os import abort
from std.python import PythonObject
from std.python.bindings import PythonModuleBuilder

from wentamojo.physics.losses import (
    local_pressure_drop as _local_pressure_drop,
    straight_pressure_drop as _straight_pressure_drop,
)


def straight_pressure_drop(
    f: PythonObject, length: PythonObject, dh: PythonObject,
    v: PythonObject, rho: PythonObject,
) raises -> PythonObject:
    return PythonObject(_straight_pressure_drop(
        Float64(py=f), Float64(py=length), Float64(py=dh),
        Float64(py=v), Float64(py=rho),
    ))


def local_pressure_drop(
    zeta: PythonObject, v: PythonObject, rho: PythonObject
) raises -> PythonObject:
    return PythonObject(_local_pressure_drop(
        Float64(py=zeta), Float64(py=v), Float64(py=rho),
    ))


@export
def PyInit_losses_ext() -> PythonObject:
    try:
        var m = PythonModuleBuilder("losses_ext")
        m.def_function[straight_pressure_drop]("straight_pressure_drop")
        m.def_function[local_pressure_drop]("local_pressure_drop")
        return m.finalize()
    except e:
        abort(String("failed to create losses_ext: ", e))
        return PythonObject()
