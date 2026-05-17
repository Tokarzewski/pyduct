"""Cross-section geometry primitives.

A `CrossSection` is an immutable value object that knows its area and
hydraulic diameter. It replaces the previous `RigidDuctType` whose 8 optional
fields encoded round and rectangular shapes in one mutable bag.
"""

from __future__ import annotations

from abc import ABC, abstractmethod
from dataclasses import dataclass
from math import pi


class CrossSection(ABC):
    """A duct cross-section."""

    @property
    @abstractmethod
    def area(self) -> float:
        """Cross-sectional area [m^2]."""

    @property
    @abstractmethod
    def hydraulic_diameter(self) -> float:
        """Hydraulic diameter D_h = 4 A / P_wetted [m]."""


@dataclass(frozen=True)
class Round(CrossSection):
    """Circular cross-section."""

    diameter: float  # [m]

    def __post_init__(self) -> None:
        if self.diameter <= 0:
            raise ValueError(f"diameter must be positive, got {self.diameter}")

    @property
    def area(self) -> float:
        return pi * (self.diameter / 2) ** 2

    @property
    def hydraulic_diameter(self) -> float:
        return self.diameter


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

    @property
    def area(self) -> float:
        return self.width * self.height

    @property
    def hydraulic_diameter(self) -> float:
        # D_h = 4 A / P = 4 (W H) / (2 (W + H)) = 2 W H / (W + H)
        return 2 * self.width * self.height / (self.width + self.height)
