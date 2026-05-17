"""Cross-section geometry primitives.

A `CrossSection` is an immutable value object that knows its area and
hydraulic diameter. Both are computed once in ``__post_init__`` and cached
as instance attributes, so per-call sizing loops avoid repeated `math`.
"""

from __future__ import annotations

from dataclasses import dataclass
from math import pi


class CrossSection:
    """Base class for duct cross-sections."""

    area: float
    hydraulic_diameter: float


@dataclass(frozen=True)
class Round(CrossSection):
    """Circular cross-section."""

    diameter: float  # [m]

    def __post_init__(self) -> None:
        if self.diameter <= 0:
            raise ValueError(f"diameter must be positive, got {self.diameter}")
        object.__setattr__(self, "area", pi * (self.diameter / 2) ** 2)
        object.__setattr__(self, "hydraulic_diameter", self.diameter)


@dataclass(frozen=True)
class Rectangular(CrossSection):
    """Rectangular cross-section."""

    width: float   # [m]
    height: float  # [m]

    def __post_init__(self) -> None:
        if self.width <= 0 or self.height <= 0:
            raise ValueError(
                f"width and height must be positive, got "
                f"width={self.width}, height={self.height}"
            )
        object.__setattr__(self, "area", self.width * self.height)
        # D_h = 4 A / P = 2 W H / (W + H)
        object.__setattr__(
            self,
            "hydraulic_diameter",
            2 * self.width * self.height / (self.width + self.height),
        )


def equivalent_round_diameter(width: float, height: float) -> float:
    """ASHRAE equivalent round diameter for a rectangular duct.

    Returns the round duct diameter that yields the same friction loss and
    air flow for the same pressure drop:

        D_eq = 1.30 · (a·b)^0.625 / (a + b)^0.25      [m]

    Reference: ASHRAE Fundamentals Handbook, Chapter 21.
    """
    if width <= 0 or height <= 0:
        raise ValueError(
            f"width and height must be positive, got width={width}, height={height}"
        )
    return 1.30 * (width * height) ** 0.625 / (width + height) ** 0.25
