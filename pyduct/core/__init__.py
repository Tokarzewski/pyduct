"""Core value objects: fluids and cross-section geometry."""

from .fluid import STANDARD_AIR, Fluid, air_at_altitude
from .geometry import CrossSection, Rectangular, Round, equivalent_round_diameter

__all__ = [
    "Fluid",
    "STANDARD_AIR",
    "air_at_altitude",
    "CrossSection",
    "Round",
    "Rectangular",
    "equivalent_round_diameter",
]
