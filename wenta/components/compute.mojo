"""Fused-math kernels for the component compute() hot path.

NOTE: These are kept as reference kernels for a future batch
``compute_pressure_drops`` port — they're not currently called from the
Python solver. Measurement showed that one Mojo boundary crossing per
component is more expensive than the original pure-Python math chain
(boundary cost ~600 ns dominates ~500 ns of native math). The path to
real solver speedup is one boundary crossing for ALL components, not
one per component; that requires a batch dispatcher running entirely
in Mojo. These kernels will plug into that dispatcher when it lands.
"""

from std.math import pi

from .fittings_library import (
    mitered_elbow as _mitered_elbow,
    rectangular_elbow as _rectangular_elbow,
)
from ..physics.friction import friction_factor, relative_roughness, reynolds


def duct_pressure_drop(
    flowrate: Float64,
    area: Float64,
    hydraulic_diameter: Float64,
    length: Float64,
    absolute_roughness: Float64,
    kinematic_viscosity: Float64,
    density: Float64,
) -> Tuple[Float64, Float64]:
    """Velocity + Darcy ΔP for a straight duct. Returns ``(v, dp)``.

    Fuses reynolds + relative_roughness + friction_factor +
    straight_pressure_drop — one boundary crossing per duct.
    """
    var v = flowrate / area
    var re = reynolds(v, hydraulic_diameter, kinematic_viscosity)
    var eps = relative_roughness(absolute_roughness, hydraulic_diameter)
    var f = friction_factor(re, eps)
    var dp = f * (length / hydraulic_diameter) * (density * v * v) * 0.5
    return Tuple[Float64, Float64](v, dp)


def fitting_pressure_drop(
    flowrate: Float64, area: Float64, zeta: Float64, density: Float64
) -> Tuple[Float64, Float64]:
    """Velocity + local ΔP for a generic ζ-based fitting. Returns ``(v, dp)``."""
    var v = flowrate / area
    var dp = zeta * density * v * v * 0.5
    return Tuple[Float64, Float64](v, dp)


def flex_duct_pressure_drop(
    flowrate: Float64,
    diameter: Float64,
    length: Float64,
    pressure_drop_per_meter: Float64,
    stretch_percentage: Float64,
) -> Tuple[Float64, Float64]:
    """Velocity + ΔP for a flexible duct. Returns ``(v, dp)``.

    Fuses area = π·(d/2)² + stretch_correction + dp = pdpm·L·β.
    """
    var r = diameter * 0.5
    var area = pi * r * r
    var v = flowrate / area
    # stretch_correction_factor body (inlined to keep one boundary crossing).
    from std.math import exp
    var beta = 0.557 * (100.0 - stretch_percentage) * exp(-4.93 * diameter) + 1.0
    var dp = pressure_drop_per_meter * length * beta
    return Tuple[Float64, Float64](v, dp)
