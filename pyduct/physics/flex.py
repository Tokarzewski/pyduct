"""Corrections specific to flexible ducts."""

from __future__ import annotations

from math import exp


def stretch_correction_factor(diameter: float, stretch_percentage: float) -> float:
    """Correction factor for the linear pressure drop of flex duct.

    Parameters
    ----------
    diameter:
        Flex duct diameter [m].
    stretch_percentage:
        100 means fully stretched (no correction). Lower values mean the duct
        is compressed and pressure drop is higher.

    Notes
    -----
    Curve fit (R^2 = 0.995) derived from the chart in *ASHRAE Handbook —
    Fundamentals*.
    """
    return 0.557 * (100 - stretch_percentage) * exp(-4.93 * diameter) + 1
