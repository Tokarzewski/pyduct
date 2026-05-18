"""Python extension exposing `wentamojo.components.fittings_library`."""

from std.os import abort
from std.python import Python, PythonObject
from std.python.bindings import PythonModuleBuilder

from wentamojo.components.fittings_library import (
    damper_butterfly as _damper_butterfly,
    diffuser_ceiling as _diffuser_ceiling,
    expander_round as _expander_round,
    grille_return as _grille_return,
    junction_tee_branch as _junction_tee_branch,
    junction_tee_combine as _junction_tee_combine,
    mitered_elbow as _mitered_elbow,
    rectangular_elbow as _rectangular_elbow,
    reducer_round as _reducer_round,
)


def reducer_round(
    d_in: PythonObject, d_out: PythonObject, angle: PythonObject
) raises -> PythonObject:
    return PythonObject(
        _reducer_round(Float64(py=d_in), Float64(py=d_out), Float64(py=angle))
    )


def expander_round(
    d_in: PythonObject, d_out: PythonObject, angle: PythonObject
) raises -> PythonObject:
    return PythonObject(
        _expander_round(Float64(py=d_in), Float64(py=d_out), Float64(py=angle))
    )


def junction_tee_branch(
    d_main: PythonObject, d_branch: PythonObject,
    q_main: PythonObject, q_branch: PythonObject,
) raises -> PythonObject:
    var r = _junction_tee_branch(
        Float64(py=d_main), Float64(py=d_branch),
        Float64(py=q_main), Float64(py=q_branch),
    )
    return Python.tuple(r[0], r[1])


def junction_tee_combine(
    d_main: PythonObject, d_branch: PythonObject,
    q_main: PythonObject, q_branch: PythonObject,
) raises -> PythonObject:
    var r = _junction_tee_combine(
        Float64(py=d_main), Float64(py=d_branch),
        Float64(py=q_main), Float64(py=q_branch),
    )
    return Python.tuple(r[0], r[1])


def damper_butterfly(open_percentage: PythonObject) raises -> PythonObject:
    return PythonObject(_damper_butterfly(Float64(py=open_percentage)))


def diffuser_ceiling(area_throw: PythonObject) raises -> PythonObject:
    return PythonObject(_diffuser_ceiling(Float64(py=area_throw)))


def grille_return(blockage: PythonObject) raises -> PythonObject:
    return PythonObject(_grille_return(Float64(py=blockage)))


def rectangular_elbow(
    width: PythonObject, height: PythonObject,
    bend_radius: PythonObject, angle: PythonObject,
) raises -> PythonObject:
    return PythonObject(
        _rectangular_elbow(
            Float64(py=width), Float64(py=height),
            Float64(py=bend_radius), Float64(py=angle),
        )
    )


def mitered_elbow(angle: PythonObject, vaned: PythonObject) raises -> PythonObject:
    return PythonObject(_mitered_elbow(Float64(py=angle), vaned=Bool(py=vaned)))


@export
def PyInit_fittings_ext() -> PythonObject:
    try:
        var m = PythonModuleBuilder("fittings_ext")
        m.def_function[reducer_round]("reducer_round")
        m.def_function[expander_round]("expander_round")
        m.def_function[junction_tee_branch]("junction_tee_branch")
        m.def_function[junction_tee_combine]("junction_tee_combine")
        m.def_function[damper_butterfly]("damper_butterfly")
        m.def_function[diffuser_ceiling]("diffuser_ceiling")
        m.def_function[grille_return]("grille_return")
        m.def_function[rectangular_elbow]("rectangular_elbow")
        m.def_function[mitered_elbow]("mitered_elbow")
        return m.finalize()
    except e:
        abort(String("failed to create fittings_ext module: ", e))
        return PythonObject()
