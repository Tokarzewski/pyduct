"""Pressure-loss primitives — Mojo-backed shim.

Math lives in ``wentamojo.physics.losses``; this module is a thin Python
wrapper.
"""

from __future__ import annotations

from wentamojo.ext.losses_ext import (
    local_pressure_drop as _local_pressure_drop,
)
from wentamojo.ext.losses_ext import (
    straight_pressure_drop as _straight_pressure_drop,
)


def straight_pressure_drop(
    friction_factor: float,
    length: float,
    hydraulic_diameter: float,
    velocity: float,
    density: float,
) -> float:
    """Darcy–Weisbach pressure drop [Pa] for a straight duct.

    ``dp = f * (L / D_h) * rho * v^2 / 2``.
    """
    return _straight_pressure_drop(
        friction_factor, length, hydraulic_diameter, velocity, density
    )


def local_pressure_drop(zeta: float, velocity: float, density: float) -> float:
    """Local (minor) fitting pressure drop ``dp = zeta * rho * v^2 / 2`` [Pa]."""
    return _local_pressure_drop(zeta, velocity, density)
