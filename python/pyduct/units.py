"""Unit converters between SI (pyduct's native units) and US customary.

Mojo-backed shim — the math runs in ``mojoduct.ext.units_ext``. The
Python import path stays the same.
"""

from __future__ import annotations

from mojoduct.ext.units_ext import (
    air_changes_per_hour as _air_changes_per_hour,
    c_to_f as _c_to_f,
    cfm_to_m3s as _cfm_to_m3s,
    f_to_c as _f_to_c,
    fpm_to_ms as _fpm_to_ms,
    ft_to_m as _ft_to_m,
    in_to_m as _in_to_m,
    inwc_to_pa as _inwc_to_pa,
    m3s_to_cfm as _m3s_to_cfm,
    m_to_ft as _m_to_ft,
    m_to_in as _m_to_in,
    ms_to_fpm as _ms_to_fpm,
    pa_to_inwc as _pa_to_inwc,
)


def cfm_to_m3s(cfm: float) -> float:
    """Convert volumetric flow from CFM (ft³/min) to m³/s."""
    return _cfm_to_m3s(cfm)


def m3s_to_cfm(m3s: float) -> float:
    """Convert volumetric flow from m³/s to CFM (ft³/min)."""
    return _m3s_to_cfm(m3s)


def inwc_to_pa(inwc: float) -> float:
    """Convert pressure from inches of water column to pascals."""
    return _inwc_to_pa(inwc)


def pa_to_inwc(pa: float) -> float:
    """Convert pressure from pascals to inches of water column."""
    return _pa_to_inwc(pa)


def ft_to_m(ft: float) -> float:
    """Feet → metres."""
    return _ft_to_m(ft)


def m_to_ft(m: float) -> float:
    """Metres → feet."""
    return _m_to_ft(m)


def in_to_m(inches: float) -> float:
    """Inches → metres."""
    return _in_to_m(inches)


def m_to_in(m: float) -> float:
    """Metres → inches."""
    return _m_to_in(m)


def fpm_to_ms(fpm: float) -> float:
    """Feet-per-minute → metres-per-second."""
    return _fpm_to_ms(fpm)


def ms_to_fpm(ms: float) -> float:
    """Metres-per-second → feet-per-minute."""
    return _ms_to_fpm(ms)


def f_to_c(fahrenheit: float) -> float:
    """Fahrenheit → Celsius."""
    return _f_to_c(fahrenheit)


def c_to_f(celsius: float) -> float:
    """Celsius → Fahrenheit."""
    return _c_to_f(celsius)


def air_changes_per_hour(flowrate_m3s: float, volume_m3: float) -> float:
    """ACH = flowrate × 3600 / room_volume.

    Raises :class:`ValueError` when ``volume_m3`` is non-positive or
    ``flowrate_m3s`` is negative.
    """
    try:
        return _air_changes_per_hour(flowrate_m3s, volume_m3)
    except Exception as e:  # Mojo raises generic Error; translate to ValueError.
        raise ValueError(str(e)) from e
