"""Loss-coefficient lookup for round elbows.

Returns a `zeta` value that can be plugged into a :class:`TwoPortFitting`.
"""

from __future__ import annotations

from dataclasses import dataclass

from scipy.interpolate import RectBivariateSpline

# Source: Wentylacja i Klimatyzacja - Materiały pomocniczne do projektowania,
# Jacek Hendiger, Piotr Ziętek, Marta Chludzińska.

_RD_GRID = (0.50, 0.75, 1.00, 1.50, 2.00, 2.50)
_ANGLE_GRID = (20, 30, 45, 60, 75, 90, 110, 130, 150, 180)
_ZETA_TABLE = (
    (0.22, 0.32, 0.43, 0.55, 0.64, 0.71, 0.80, 0.85, 0.91, 0.99),
    (0.10, 0.15, 0.20, 0.26, 0.30, 0.33, 0.37, 0.40, 0.42, 0.46),
    (0.07, 0.10, 0.13, 0.17, 0.20, 0.22, 0.25, 0.26, 0.28, 0.31),
    (0.05, 0.07, 0.09, 0.12, 0.14, 0.15, 0.17, 0.18, 0.19, 0.21),
    (0.04, 0.06, 0.08, 0.10, 0.12, 0.13, 0.15, 0.16, 0.17, 0.18),
    (0.04, 0.05, 0.07, 0.09, 0.11, 0.12, 0.14, 0.14, 0.15, 0.17),
)

# Build the spline once at import time.
_SPLINE = RectBivariateSpline(_RD_GRID, _ANGLE_GRID, _ZETA_TABLE)


@dataclass(frozen=True)
class ElbowRound:
    """Round elbow loss coefficient.

    Parameters
    ----------
    bend_radius:
        Centreline bend radius R [m].
    diameter:
        Duct diameter D [m].
    angle:
        Bend angle [deg], in the range [20, 180].

    Attributes
    ----------
    zeta:
        Interpolated loss coefficient (dimensionless), looked up by R/D and
        angle from the source table.
    """

    bend_radius: float
    diameter: float
    angle: float

    def __post_init__(self) -> None:
        if self.diameter <= 0:
            raise ValueError(f"diameter must be positive, got {self.diameter}")
        if self.bend_radius <= 0:
            raise ValueError(f"bend_radius must be positive, got {self.bend_radius}")
        rd = self.bend_radius / self.diameter
        if not (_RD_GRID[0] <= rd <= _RD_GRID[-1]):
            raise ValueError(
                f"R/D = {rd:.3f} is outside the tabulated range "
                f"[{_RD_GRID[0]}, {_RD_GRID[-1]}]"
            )
        if not (_ANGLE_GRID[0] <= self.angle <= _ANGLE_GRID[-1]):
            raise ValueError(
                f"angle = {self.angle} is outside the tabulated range "
                f"[{_ANGLE_GRID[0]}, {_ANGLE_GRID[-1]}]"
            )

    @property
    def zeta(self) -> float:
        rd = self.bend_radius / self.diameter
        # RectBivariateSpline returns a 2-D array; pull out the scalar.
        return float(_SPLINE(rd, self.angle).item())
