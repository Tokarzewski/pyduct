"""Pressure-loss formulas."""

from __future__ import annotations


def straight_pressure_drop(
    friction_factor: float,
    length: float,
    hydraulic_diameter: float,
    velocity: float,
    density: float,
) -> float:
    """Darcy–Weisbach pressure drop [Pa] for a straight duct of given length.

    ``dp = f * (L / D_h) * (rho * v^2 / 2)``
    """
    return (
        friction_factor
        * length
        / hydraulic_diameter
        * (density * velocity ** 2)
        / 2
    )


def local_pressure_drop(zeta: float, velocity: float, density: float) -> float:
    """Local (minor) pressure drop [Pa]: ``dp = zeta * rho * v^2 / 2``."""
    return zeta * density * velocity ** 2 / 2
