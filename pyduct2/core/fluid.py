"""Fluid properties used in duct calculations."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class Fluid:
    """A working fluid (typically air).

    Parameters
    ----------
    density:
        Mass density rho [kg/m^3].
    dynamic_viscosity:
        Dynamic viscosity mu [Pa.s].
    """

    density: float
    dynamic_viscosity: float

    def __post_init__(self) -> None:
        if self.density <= 0:
            raise ValueError(f"density must be positive, got {self.density}")
        if self.dynamic_viscosity <= 0:
            raise ValueError(
                f"dynamic_viscosity must be positive, got {self.dynamic_viscosity}"
            )

    @property
    def kinematic_viscosity(self) -> float:
        """Kinematic viscosity nu = mu / rho [m^2/s]."""
        return self.dynamic_viscosity / self.density


# Standard dry air at 20 deg C, 101 325 Pa.
# Values match CoolProp.PropsSI("D"/"V", "T", 293.15, "P", 101325, "Air")
# to 4 significant figures, so the library has no runtime dependency on CoolProp.
STANDARD_AIR = Fluid(density=1.204, dynamic_viscosity=1.825e-5)
