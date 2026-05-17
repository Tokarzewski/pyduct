"""Flex-duct corrections (Mojo port of `pyduct.physics.flex`)."""

from std.math import exp


def stretch_correction_factor(diameter: Float64, stretch_percentage: Float64) -> Float64:
    """Pressure-drop multiplier for a partially-stretched flex duct.

    100 % stretched → factor 1.0 (no correction); lower stretch → higher
    multiplier. Curve fit (R² = 0.995) from ASHRAE Fundamentals.
    """
    return 0.557 * (100.0 - stretch_percentage) * exp(-4.93 * diameter) + 1.0
