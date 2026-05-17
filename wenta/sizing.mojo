"""Duct sizing methods (Mojo port of `pyduct.sizing`).

This file currently ports the round-duct path of ``velocity_method`` — the
most common sizing call. The rectangular path follows the same shape and
will be added as a sibling function once a Mojo discriminated-union is
worth the complexity.

The returned `(Round, Float64)` is a sized-once value: caller can read
``.diameter`` and ``.area`` directly without any further allocation.
"""

from .core.fluid import Fluid, standard_air
from .core.geometry import Rectangular, Round
from .data.standard_sizes import _rect_sizes_mm, _round_sizes_mm
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


# --- Rectangular paths ---------------------------------------------------


def velocity_method_rectangular(
    flowrate: Float64,
    target_velocity: Float64 = 4.0,
) raises -> Tuple[Rectangular, Float64]:
    """Smallest EN-standard rectangular duct whose velocity ≤ ``target_velocity``."""
    if flowrate <= 0.0:
        raise Error("flowrate must be positive")
    if target_velocity <= 0.0:
        raise Error("target_velocity must be positive")

    var sizes = _rect_sizes_mm()
    var n = len(sizes)
    var w_last = Float64(sizes[n - 1][0]) * 0.001
    var h_last = Float64(sizes[n - 1][1]) * 0.001
    var last_section = Rectangular(w_last, h_last)
    var last_v = flowrate / last_section.area

    for i in range(n):
        var w = Float64(sizes[i][0]) * 0.001
        var h = Float64(sizes[i][1]) * 0.001
        var section = Rectangular(w, h)
        var v = flowrate / section.area
        if v <= target_velocity:
            return Tuple[Rectangular, Float64](section, v)
        last_section = section
        last_v = v
    return Tuple[Rectangular, Float64](last_section, last_v)


def _dp_per_m_rect(
    section: Rectangular, flowrate: Float64, absolute_roughness: Float64, fluid: Fluid
) -> Float64:
    var v = flowrate / section.area
    var d_h = section.hydraulic_diameter
    var f = friction_factor(
        reynolds(v, d_h, fluid.kinematic_viscosity),
        relative_roughness(absolute_roughness, d_h),
    )
    return f / d_h * (fluid.density * v * v) * 0.5


def equal_friction_method_rectangular(
    flowrate: Float64,
    target_pressure_drop_per_meter: Float64 = 1.0,
    absolute_roughness: Float64 = 0.0001,
    fluid: Optional[Fluid] = None,
) raises -> Tuple[Rectangular, Float64, Float64]:
    """Smallest EN-standard rectangular duct with linear ΔP ≤ target."""
    if flowrate <= 0.0:
        raise Error("flowrate must be positive")
    if target_pressure_drop_per_meter <= 0.0:
        raise Error("target_pressure_drop_per_meter must be positive")
    var f = fluid.value() if fluid else standard_air()
    var sizes = _rect_sizes_mm()
    var n = len(sizes)
    var w_last = Float64(sizes[n - 1][0]) * 0.001
    var h_last = Float64(sizes[n - 1][1]) * 0.001
    var last_section = Rectangular(w_last, h_last)
    var last_r = _dp_per_m_rect(last_section, flowrate, absolute_roughness, f)

    for i in range(n):
        var w = Float64(sizes[i][0]) * 0.001
        var h = Float64(sizes[i][1]) * 0.001
        var section = Rectangular(w, h)
        var r = _dp_per_m_rect(section, flowrate, absolute_roughness, f)
        if r <= target_pressure_drop_per_meter:
            var v = flowrate / section.area
            return Tuple[Rectangular, Float64, Float64](section, v, r)
        last_section = section
        last_r = r
    var v_last = flowrate / last_section.area
    return Tuple[Rectangular, Float64, Float64](last_section, v_last, last_r)


def pressure_drop_budget_rectangular(
    flowrate: Float64,
    length: Float64,
    budget_pa: Float64,
    absolute_roughness: Float64 = 0.0001,
    fluid: Optional[Fluid] = None,
) raises -> Tuple[Rectangular, Float64, Float64]:
    """Size a rectangular duct so total ΔP across ``length`` ≤ ``budget_pa``."""
    if length <= 0.0:
        raise Error("length must be positive")
    if budget_pa <= 0.0:
        raise Error("budget_pa must be positive")
    return equal_friction_method_rectangular(
        flowrate, budget_pa / length, absolute_roughness, fluid
    )


# --- Noise-limit and aspect-ratio sizing ---------------------------------


def noise_limit_velocity(space_type: String) raises -> Float64:
    """Maximum air velocity [m/s] under typical NC targets for ``space_type``.

    Mirrors :data:`pyduct.sizing.NOISE_LIMITS_M_S` — supports
    "studio", "bedroom", "office", "classroom", "retail", "industrial".
    """
    if space_type == "studio":     return 2.5
    if space_type == "bedroom":    return 3.0
    if space_type == "office":     return 4.0
    if space_type == "classroom":  return 4.5
    if space_type == "retail":     return 5.0
    if space_type == "industrial": return 7.5
    raise Error("unknown space_type; expected studio|bedroom|office|classroom|retail|industrial")


def noise_limit_method_round(
    flowrate: Float64, space_type: String = String("office")
) raises -> Tuple[Round, Float64]:
    """Round-duct sizing under the noise-limit velocity for ``space_type``."""
    return velocity_method_round(flowrate, noise_limit_velocity(space_type))


def noise_limit_method_rectangular(
    flowrate: Float64, space_type: String = String("office")
) raises -> Tuple[Rectangular, Float64]:
    """Rectangular-duct sizing under the noise-limit velocity for ``space_type``."""
    return velocity_method_rectangular(flowrate, noise_limit_velocity(space_type))


def aspect_ratio_method(
    flowrate: Float64,
    target_velocity: Float64 = 4.0,
    aspect_ratio: Float64 = 2.0,
) raises -> Tuple[Rectangular, Float64]:
    """Size a rectangular duct at a target velocity and minimum aspect ratio.

    Iterates EN-standard sizes whose long/short dimension ratio is at
    least ``aspect_ratio`` — useful for low-rise ceiling voids — and
    returns the smallest one whose velocity ≤ ``target_velocity``. Falls
    back to the largest qualifying size if none meet the velocity target.
    """
    if flowrate <= 0.0:
        raise Error("flowrate must be positive")
    if target_velocity <= 0.0:
        raise Error("target_velocity must be positive")
    if aspect_ratio < 1.0:
        raise Error("aspect_ratio must be >= 1")

    var sizes = _rect_sizes_mm()
    # Filter to qualifying (w, h) pairs, sorted by area ascending.
    # EN 1505 is already roughly sorted by width then height; a stable
    # area-sort runs in 22*log22 ops — negligible.
    var qualifying = List[Rectangular]()
    for i in range(len(sizes)):
        var w = Float64(sizes[i][0]) * 0.001
        var h = Float64(sizes[i][1]) * 0.001
        var long = w if w >= h else h
        var short = h if w >= h else w
        if long / short >= aspect_ratio:
            qualifying.append(Rectangular(w, h))
    if len(qualifying) == 0:
        raise Error("no standard rectangular size meets the aspect_ratio")

    # Insertion-sort by area (tiny lists).
    for i in range(1, len(qualifying)):
        var j = i
        while j > 0 and qualifying[j].area < qualifying[j - 1].area:
            var tmp = qualifying[j]
            qualifying[j] = qualifying[j - 1]
            qualifying[j - 1] = tmp
            j -= 1

    var last_section = qualifying[len(qualifying) - 1]
    var last_v = flowrate / last_section.area
    for i in range(len(qualifying)):
        var s = qualifying[i]
        var v = flowrate / s.area
        if v <= target_velocity:
            return Tuple[Rectangular, Float64](s, v)
        last_section = s
        last_v = v
    return Tuple[Rectangular, Float64](last_section, last_v)
