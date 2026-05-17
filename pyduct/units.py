"""Unit converters between SI (pyduct's native units) and US customary.

pyduct works internally in metres, m³/s, Pa, kg/m³, °C, m/s. These helpers
let you accept inputs (and report outputs) in the US-customary units common
in ASHRAE design work without polluting the core API with double-unit
parameters.

All converters are pure functions and free of dependencies.
"""

from __future__ import annotations

# Volumetric flow ----------------------------------------------------------
_CFM_TO_M3S = 0.0004719474432  # ft³/min → m³/s


def cfm_to_m3s(cfm: float) -> float:
    """Convert volumetric flow from CFM (ft³/min) to m³/s."""
    return cfm * _CFM_TO_M3S


def m3s_to_cfm(m3s: float) -> float:
    """Convert volumetric flow from m³/s to CFM (ft³/min)."""
    return m3s / _CFM_TO_M3S


# Pressure -----------------------------------------------------------------
_INWC_TO_PA = 249.0889  # inch of water column at 4 °C → Pa


def inwc_to_pa(inwc: float) -> float:
    """Convert pressure from inches of water column to pascals."""
    return inwc * _INWC_TO_PA


def pa_to_inwc(pa: float) -> float:
    """Convert pressure from pascals to inches of water column."""
    return pa / _INWC_TO_PA


# Length -------------------------------------------------------------------
def ft_to_m(ft: float) -> float:
    """Feet → metres."""
    return ft * 0.3048


def m_to_ft(m: float) -> float:
    """Metres → feet."""
    return m / 0.3048


def in_to_m(inches: float) -> float:
    """Inches → metres."""
    return inches * 0.0254


def m_to_in(m: float) -> float:
    """Metres → inches."""
    return m / 0.0254


# Velocity -----------------------------------------------------------------
def fpm_to_ms(fpm: float) -> float:
    """Feet-per-minute → metres-per-second."""
    return fpm * 0.00508


def ms_to_fpm(ms: float) -> float:
    """Metres-per-second → feet-per-minute."""
    return ms / 0.00508


# Temperature --------------------------------------------------------------
def f_to_c(fahrenheit: float) -> float:
    """Fahrenheit → Celsius."""
    return (fahrenheit - 32.0) * 5.0 / 9.0


def c_to_f(celsius: float) -> float:
    """Celsius → Fahrenheit."""
    return celsius * 9.0 / 5.0 + 32.0


# Room / building helpers --------------------------------------------------
def air_changes_per_hour(flowrate_m3s: float, volume_m3: float) -> float:
    """Air changes per hour (ACH) for a room.

    Parameters
    ----------
    flowrate_m3s:
        Supply (or extract) volumetric flow rate into the room [m³/s].
    volume_m3:
        Internal room volume [m³].
    """
    if volume_m3 <= 0:
        raise ValueError(f"volume_m3 must be positive, got {volume_m3}")
    if flowrate_m3s < 0:
        raise ValueError(f"flowrate_m3s must be non-negative, got {flowrate_m3s}")
    return flowrate_m3s * 3600.0 / volume_m3
