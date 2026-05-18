"""Python extension exposing `wenta.physics.friction` to Python."""

from std.os import abort
from std.python import PythonObject
from std.python.bindings import PythonModuleBuilder

from wenta.physics.friction import (
    friction_factor as _friction_factor,
    friction_factor_colebrook as _friction_factor_colebrook,
    relative_roughness as _relative_roughness,
    reynolds as _reynolds,
)


def reynolds(v: PythonObject, dh: PythonObject, nu: PythonObject) raises -> PythonObject:
    return PythonObject(_reynolds(Float64(py=v), Float64(py=dh), Float64(py=nu)))


def relative_roughness(eps: PythonObject, dh: PythonObject) raises -> PythonObject:
    return PythonObject(_relative_roughness(Float64(py=eps), Float64(py=dh)))


def friction_factor(re: PythonObject, eps: PythonObject) raises -> PythonObject:
    return PythonObject(_friction_factor(Float64(py=re), Float64(py=eps)))


def friction_factor_colebrook(re: PythonObject, eps: PythonObject) raises -> PythonObject:
    return PythonObject(_friction_factor_colebrook(Float64(py=re), Float64(py=eps)))


@export
def PyInit_friction_ext() -> PythonObject:
    try:
        var m = PythonModuleBuilder("friction_ext")
        m.def_function[reynolds]("reynolds")
        m.def_function[relative_roughness]("relative_roughness")
        m.def_function[friction_factor]("friction_factor")
        m.def_function[friction_factor_colebrook]("friction_factor_colebrook")
        return m.finalize()
    except e:
        abort(String("failed to create friction_ext: ", e))
        return PythonObject()
