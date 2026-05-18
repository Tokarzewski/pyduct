"""Python extension exposing `wentamojo.units` to Python."""

from std.os import abort
from std.python import PythonObject
from std.python.bindings import PythonModuleBuilder

from wentamojo.units import (
    cfm_to_m3s as _cfm_to_m3s,
    m3s_to_cfm as _m3s_to_cfm,
    inwc_to_pa as _inwc_to_pa,
    pa_to_inwc as _pa_to_inwc,
    ft_to_m as _ft_to_m,
    m_to_ft as _m_to_ft,
    in_to_m as _in_to_m,
    m_to_in as _m_to_in,
    fpm_to_ms as _fpm_to_ms,
    ms_to_fpm as _ms_to_fpm,
    f_to_c as _f_to_c,
    c_to_f as _c_to_f,
    air_changes_per_hour as _ach,
)


def cfm_to_m3s(v: PythonObject) raises -> PythonObject:
    return PythonObject(_cfm_to_m3s(Float64(py=v)))


def m3s_to_cfm(v: PythonObject) raises -> PythonObject:
    return PythonObject(_m3s_to_cfm(Float64(py=v)))


def inwc_to_pa(v: PythonObject) raises -> PythonObject:
    return PythonObject(_inwc_to_pa(Float64(py=v)))


def pa_to_inwc(v: PythonObject) raises -> PythonObject:
    return PythonObject(_pa_to_inwc(Float64(py=v)))


def ft_to_m(v: PythonObject) raises -> PythonObject:
    return PythonObject(_ft_to_m(Float64(py=v)))


def m_to_ft(v: PythonObject) raises -> PythonObject:
    return PythonObject(_m_to_ft(Float64(py=v)))


def in_to_m(v: PythonObject) raises -> PythonObject:
    return PythonObject(_in_to_m(Float64(py=v)))


def m_to_in(v: PythonObject) raises -> PythonObject:
    return PythonObject(_m_to_in(Float64(py=v)))


def fpm_to_ms(v: PythonObject) raises -> PythonObject:
    return PythonObject(_fpm_to_ms(Float64(py=v)))


def ms_to_fpm(v: PythonObject) raises -> PythonObject:
    return PythonObject(_ms_to_fpm(Float64(py=v)))


def f_to_c(v: PythonObject) raises -> PythonObject:
    return PythonObject(_f_to_c(Float64(py=v)))


def c_to_f(v: PythonObject) raises -> PythonObject:
    return PythonObject(_c_to_f(Float64(py=v)))


def air_changes_per_hour(
    flow: PythonObject, volume: PythonObject
) raises -> PythonObject:
    return PythonObject(_ach(Float64(py=flow), Float64(py=volume)))


@export
def PyInit_units_ext() -> PythonObject:
    try:
        var m = PythonModuleBuilder("units_ext")
        m.def_function[cfm_to_m3s]("cfm_to_m3s")
        m.def_function[m3s_to_cfm]("m3s_to_cfm")
        m.def_function[inwc_to_pa]("inwc_to_pa")
        m.def_function[pa_to_inwc]("pa_to_inwc")
        m.def_function[ft_to_m]("ft_to_m")
        m.def_function[m_to_ft]("m_to_ft")
        m.def_function[in_to_m]("in_to_m")
        m.def_function[m_to_in]("m_to_in")
        m.def_function[fpm_to_ms]("fpm_to_ms")
        m.def_function[ms_to_fpm]("ms_to_fpm")
        m.def_function[f_to_c]("f_to_c")
        m.def_function[c_to_f]("c_to_f")
        m.def_function[air_changes_per_hour]("air_changes_per_hour")
        return m.finalize()
    except e:
        abort(String("failed to create units_ext module: ", e))
        return PythonObject()
