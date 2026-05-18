"""Flex-duct correction — Mojo-backed shim.

Math lives in ``wentamojo.physics.flex``; this module is a thin Python
wrapper.
"""

from __future__ import annotations

from wentamojo.ext.flex_ext import (
    stretch_correction_factor as _stretch_correction_factor,
)


def stretch_correction_factor(diameter: float, stretch_percentage: float) -> float:
    """Pressure-drop multiplier for a partially stretched flex duct.

    100 % stretched → factor 1.0 (no correction); lower stretch → higher
    multiplier. Curve fit (R² = 0.995) from ASHRAE Fundamentals.
    """
    return _stretch_correction_factor(diameter, stretch_percentage)
