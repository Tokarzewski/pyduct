"""Duct sizing methods (Mojo port of `pyduct.sizing`).

This file currently ports the round-duct path of ``velocity_method`` — the
most common sizing call. The rectangular path follows the same shape and
will be added as a sibling function once a Mojo discriminated-union is
worth the complexity.

The returned `(Round, Float64)` is a sized-once value: caller can read
``.diameter`` and ``.area`` directly without any further allocation.
"""

from .core.fluid import Fluid, standard_air
from .core.geometry import Round
from .data.standard_sizes import _round_sizes_mm
from .physics.friction import friction_factor, relative_roughness, reynolds


def velocity_method_round(
    flowrate: Float64,
    target_velocity: Float64 = 4.0,
) raises -> Tuple[Round, Float64]:
    """Smallest EN-standard round duct whose velocity ≤ ``target_velocity``.

    Returns ``(section, actual_velocity)``. If no standard size meets the
    target, the largest size is returned with its velocity (still ≥ target).
    """
    if flowrate <= 0.0:
        raise Error("flowrate must be positive")
    if target_velocity <= 0.0:
        raise Error("target_velocity must be positive")

    var sizes = _round_sizes_mm()
    var n = len(sizes)
    var last_section = Round(Float64(sizes[n - 1]) * 0.001)
    var last_v = flowrate / last_section.area

    for i in range(n):
        var section = Round(Float64(sizes[i]) * 0.001)
        var v = flowrate / section.area
        if v <= target_velocity:
            return Tuple[Round, Float64](section, v)
        last_section = section
        last_v = v
    # Loop exhausted: fall back to largest size.
    return Tuple[Round, Float64](last_section, last_v)


def _dp_per_m_round(
    section: Round, flowrate: Float64, absolute_roughness: Float64, fluid: Fluid
) -> Float64:
    """Linear pressure drop [Pa/m] for ``section`` carrying ``flowrate``."""
    var v = flowrate / section.area
    var d_h = section.hydraulic_diameter
    var f = friction_factor(
        reynolds(v, d_h, fluid.kinematic_viscosity),
        relative_roughness(absolute_roughness, d_h),
    )
    return f / d_h * (fluid.density * v * v) * 0.5


def equal_friction_method_round(
    flowrate: Float64,
    target_pressure_drop_per_meter: Float64 = 1.0,
    absolute_roughness: Float64 = 0.0001,
    fluid: Optional[Fluid] = None,
) raises -> Tuple[Round, Float64, Float64]:
    """Smallest EN-standard round duct with linear ΔP ≤ target [Pa/m].

    Returns ``(section, velocity_m_s, pressure_drop_per_meter)``. ``fluid``
    defaults to ``standard_air()``.
    """
    if flowrate <= 0.0:
        raise Error("flowrate must be positive")
    if target_pressure_drop_per_meter <= 0.0:
        raise Error("target_pressure_drop_per_meter must be positive")

    var f = fluid.value() if fluid else standard_air()
    var sizes = _round_sizes_mm()
    var n = len(sizes)
    var last_section = Round(Float64(sizes[n - 1]) * 0.001)
    var last_r = _dp_per_m_round(last_section, flowrate, absolute_roughness, f)

    for i in range(n):
        var section = Round(Float64(sizes[i]) * 0.001)
        var r = _dp_per_m_round(section, flowrate, absolute_roughness, f)
        if r <= target_pressure_drop_per_meter:
            var v = flowrate / section.area
            return Tuple[Round, Float64, Float64](section, v, r)
        last_section = section
        last_r = r

    var v_last = flowrate / last_section.area
    return Tuple[Round, Float64, Float64](last_section, v_last, last_r)


def pressure_drop_budget_round(
    flowrate: Float64,
    length: Float64,
    budget_pa: Float64,
    absolute_roughness: Float64 = 0.0001,
    fluid: Optional[Fluid] = None,
) raises -> Tuple[Round, Float64, Float64]:
    """Size a round duct so total pressure drop across ``length`` ≤ ``budget_pa``."""
    if length <= 0.0:
        raise Error("length must be positive")
    if budget_pa <= 0.0:
        raise Error("budget_pa must be positive")
    return equal_friction_method_round(
        flowrate, budget_pa / length, absolute_roughness, fluid
    )
