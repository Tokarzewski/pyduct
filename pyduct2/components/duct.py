"""Straight duct components."""

from __future__ import annotations

from dataclasses import dataclass, field
from math import pi

from ..core.fluid import Fluid
from ..core.geometry import CrossSection
from ..physics.flex import stretch_correction_factor
from ..physics.friction import (
    friction_factor,
    relative_roughness,
    reynolds,
)
from ..physics.losses import straight_pressure_drop
from .base import Component, Port


@dataclass
class RigidDuct(Component):
    """A rigid (sheet-metal) straight duct.

    The full Darcy–Weisbach pressure drop is reported on the inlet port; the
    outlet port carries 0 so no double-counting occurs along a critical path.
    """

    name: str
    cross_section: CrossSection
    length: float
    absolute_roughness: float = 0.0001  # [m] - typical galvanized steel
    ports: list[Port] = field(init=False)

    def __post_init__(self) -> None:
        if self.length <= 0:
            raise ValueError(f"length must be positive, got {self.length}")
        self.ports = [
            Port(name="inlet", direction="in"),
            Port(name="outlet", direction="out"),
        ]

    def compute(self, fluid: Fluid) -> None:
        inlet, outlet = self.ports
        if inlet.flowrate is None:
            raise ValueError(
                f"RigidDuct {self.name!r}: inlet flowrate not set"
            )

        d_h = self.cross_section.hydraulic_diameter
        v = inlet.flowrate / self.cross_section.area
        re = reynolds(v, d_h, fluid.kinematic_viscosity)
        eps = relative_roughness(self.absolute_roughness, d_h)
        f = friction_factor(re, eps)

        inlet.velocity = v
        outlet.velocity = v
        outlet.flowrate = inlet.flowrate
        inlet.pressure_drop = straight_pressure_drop(
            f, self.length, d_h, v, fluid.density
        )
        outlet.pressure_drop = 0.0


@dataclass
class FlexDuct(Component):
    """A flexible round duct with manufacturer-supplied per-meter pressure drop.

    Because flex pressure-drop curves are highly product-specific, this
    component takes the per-meter drop as an explicit parameter rather than
    trying to derive it from a friction factor.
    """

    name: str
    diameter: float                  # [m]
    length: float                    # [m]
    pressure_drop_per_meter: float   # [Pa/m] from manufacturer chart
    stretch_percentage: float = 100.0
    ports: list[Port] = field(init=False)

    def __post_init__(self) -> None:
        if self.diameter <= 0 or self.length <= 0:
            raise ValueError("diameter and length must be positive")
        if not 0 < self.stretch_percentage <= 100:
            raise ValueError(
                f"stretch_percentage must be in (0, 100], got {self.stretch_percentage}"
            )
        self._area = pi * (self.diameter / 2) ** 2
        self.ports = [
            Port(name="inlet", direction="in"),
            Port(name="outlet", direction="out"),
        ]

    def compute(self, fluid: Fluid) -> None:
        inlet, outlet = self.ports
        if inlet.flowrate is None:
            raise ValueError(
                f"FlexDuct {self.name!r}: inlet flowrate not set"
            )
        v = inlet.flowrate / self._area
        beta = stretch_correction_factor(self.diameter, self.stretch_percentage)
        inlet.velocity = v
        outlet.velocity = v
        outlet.flowrate = inlet.flowrate
        inlet.pressure_drop = self.pressure_drop_per_meter * self.length * beta
        outlet.pressure_drop = 0.0
