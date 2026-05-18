"""Python extension exposing `wenta.physics.flex` to Python."""

from std.os import abort
from std.python import PythonObject
from std.python.bindings import PythonModuleBuilder

from wenta.physics.flex import stretch_correction_factor as _scf


def stretch_correction_factor(
    diameter: PythonObject, stretch_percentage: PythonObject
) raises -> PythonObject:
    return PythonObject(_scf(Float64(py=diameter), Float64(py=stretch_percentage)))


@export
def PyInit_flex_ext() -> PythonObject:
    try:
        var m = PythonModuleBuilder("flex_ext")
        m.def_function[stretch_correction_factor]("stretch_correction_factor")
        return m.finalize()
    except e:
        abort(String("failed to create flex_ext: ", e))
        return PythonObject()
