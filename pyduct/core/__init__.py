"""Core value objects: fluids and cross-section geometry."""

from .fluid import STANDARD_AIR, Fluid
from .geometry import CrossSection, Rectangular, Round

__all__ = ["Fluid", "STANDARD_AIR", "CrossSection", "Round", "Rectangular"]
