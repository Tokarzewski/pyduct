"""Core value objects: fluids and cross-section geometry."""

from .fluid import Fluid, STANDARD_AIR
from .geometry import CrossSection, Round, Rectangular

__all__ = ["Fluid", "STANDARD_AIR", "CrossSection", "Round", "Rectangular"]
