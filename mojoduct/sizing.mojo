"""Duct sizing methods (Mojo port of `pyduct.sizing`).

This file currently ports the round-duct path of ``velocity_method`` — the
most common sizing call. The rectangular path follows the same shape and
will be added as a sibling function once a Mojo discriminated-union is
worth the complexity.

The returned `(Round, Float64)` is a sized-once value: caller can read
``.diameter`` and ``.area`` directly without any further allocation.
"""

from .core.geometry import Round
from .data.standard_sizes import _round_sizes_mm


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
