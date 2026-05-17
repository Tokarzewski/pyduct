"""Fluid properties — density, dynamic & kinematic viscosity (Mojo port).

Mirrors `pyduct.core.fluid`:

* `Fluid(density, dynamic_viscosity)` — frozen value object
* `standard_air()` — dry air at 20 °C, 101 325 Pa (matches CoolProp to 4 sf)
* `air_at_altitude(altitude_m, temperature_c)` — ISA atmosphere density
  + Sutherland viscosity for high-elevation projects.
"""


struct Fluid(Copyable, ImplicitlyCopyable, Movable):
    """Working fluid (typically air) with cached kinematic viscosity."""

    var density: Float64                # rho [kg/m^3]
    var dynamic_viscosity: Float64      # mu [Pa.s]
    var kinematic_viscosity: Float64    # nu = mu / rho [m^2/s]

    def __init__(out self, density: Float64, dynamic_viscosity: Float64) raises:
        if density <= 0.0:
            raise Error("density must be positive")
        if dynamic_viscosity <= 0.0:
            raise Error("dynamic_viscosity must be positive")
        self.density = density
        self.dynamic_viscosity = dynamic_viscosity
        self.kinematic_viscosity = dynamic_viscosity / density


def standard_air() raises -> Fluid:
    """Dry air at 20 °C, 101 325 Pa (matches CoolProp.PropsSI to 4 sf)."""
    return Fluid(density=1.204, dynamic_viscosity=1.825e-5)


def air_at_altitude(altitude_m: Float64, temperature_c: Float64 = 20.0) raises -> Fluid:
    """Dry-air properties at altitude (ISA) and temperature (°C)."""
    if altitude_m < 0.0:
        raise Error("altitude_m must be non-negative")
    var h = altitude_m if altitude_m < 11000.0 else 11000.0
    # ISA pressure up to the tropopause.
    var pressure = 101325.0 * (1.0 - 2.25577e-5 * h) ** 5.2561
    var t_k = temperature_c + 273.15
    var r_specific = 287.058     # J/(kg·K) for dry air
    var density = pressure / (r_specific * t_k)
    # Sutherland: mu(T) = 1.458e-6 * T^1.5 / (T + 110.4)
    var mu = 1.458e-6 * t_k ** 1.5 / (t_k + 110.4)
    return Fluid(density=density, dynamic_viscosity=mu)
