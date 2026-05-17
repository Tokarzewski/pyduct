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


def air_at_altitude(altitude_m: float, temperature_c: float = 20.0) -> Fluid:
    """Dry-air properties at a given altitude and temperature.

    Pressure follows the ISA standard atmosphere up to the tropopause
    (≈ 11 000 m); density uses the ideal-gas law. Dynamic viscosity uses
    Sutherland's formula (essentially temperature-only — pressure dependence
    is negligible at HVAC scales).

    Parameters
    ----------
    altitude_m:
        Elevation above sea level [m]; clamped to [0, 11_000].
    temperature_c:
        Dry-bulb air temperature [°C]; defaults to 20 °C.

    Returns
    -------
    Fluid
        Density and dynamic viscosity at the requested conditions.
    """
    if altitude_m < 0:
        raise ValueError(f"altitude_m must be non-negative, got {altitude_m}")
    h = min(altitude_m, 11_000.0)
    # ISA: T_isa(h) = 288.15 - 0.0065 h; P(h) = 101325 * (T_isa/288.15)**5.2561
    pressure = 101_325.0 * (1.0 - 2.25577e-5 * h) ** 5.2561
    T_k = temperature_c + 273.15
    R_specific = 287.058  # J/(kg·K) for dry air
    density = pressure / (R_specific * T_k)
    # Sutherland for air: mu(T) = 1.458e-6 * T^1.5 / (T + 110.4)
    dynamic_viscosity = 1.458e-6 * T_k ** 1.5 / (T_k + 110.4)
    return Fluid(density=density, dynamic_viscosity=dynamic_viscosity)
