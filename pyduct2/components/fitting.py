"""Fitting and terminal components."""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Optional

from ..core.fluid import Fluid
from ..core.geometry import CrossSection
from ..physics.losses import local_pressure_drop
from .base import Component, Port


@dataclass
class Source(Component):
    """A flow source — typically the AHU/fan supplying the network.

    `Source` has a single ``out`` port. Its flowrate is determined by the
    solver as the sum of all downstream terminal demands.
    """

    name: str
    ports: list[Port] = field(init=False)

    def __post_init__(self) -> None:
        self.ports = [Port(name="outlet", direction="out")]

    def compute(self, fluid: Fluid) -> None:
        # A pure source contributes no pressure drop of its own.
        port = self.ports[0]
        port.velocity = 0.0
        port.pressure_drop = 0.0


@dataclass
class Terminal(Component):
    """A one-port terminal: diffuser, grille, register, or cap.

    `flowrate` is the *demanded* volumetric flow [m^3/s] at this terminal —
    use 0 for a cap. If `cross_section` and `zeta` are supplied, the local
    pressure drop of the terminal device is also computed.
    """

    name: str
    flowrate: float
    cross_section: Optional[CrossSection] = None
    zeta: float = 0.0
    ports: list[Port] = field(init=False)

    def __post_init__(self) -> None:
        if self.flowrate < 0:
            raise ValueError(f"flowrate must be >= 0, got {self.flowrate}")
        self.ports = [
            Port(name="inlet", direction="in", flowrate=self.flowrate),
        ]

    def compute(self, fluid: Fluid) -> None:
        port = self.ports[0]
        if (
            port.flowrate is None
            or port.flowrate == 0
            or self.cross_section is None
        ):
            port.velocity = 0.0
            port.pressure_drop = 0.0
            return
        v = port.flowrate / self.cross_section.area
        port.velocity = v
        port.pressure_drop = local_pressure_drop(self.zeta, v, fluid.density)


@dataclass
class TwoPortFitting(Component):
    """A generic in-line fitting (elbow, reducer, transition, damper, ...).

    The local pressure drop is computed from `zeta` referenced to the
    velocity at `cross_section`. The drop is reported on the outlet port so
    it accumulates correctly along the critical path.
    """

    name: str
    cross_section: CrossSection
    zeta: float
    ports: list[Port] = field(init=False)

    def __post_init__(self) -> None:
        self.ports = [
            Port(name="inlet", direction="in"),
            Port(name="outlet", direction="out"),
        ]

    def compute(self, fluid: Fluid) -> None:
        inlet, outlet = self.ports
        if inlet.flowrate is None:
            raise ValueError(
                f"TwoPortFitting {self.name!r}: inlet flowrate not set"
            )
        v = inlet.flowrate / self.cross_section.area
        inlet.velocity = v
        outlet.velocity = v
        outlet.flowrate = inlet.flowrate
        inlet.pressure_drop = 0.0
        outlet.pressure_drop = local_pressure_drop(
            self.zeta, v, fluid.density
        )


@dataclass
class Tee(Component):
    """A three-port branch fitting.

    Ports
    -----
    * ``combined`` — the upstream side (single ``in`` port).
    * ``straight`` — the downstream straight leg (``out``).
    * ``branch``   — the downstream branch leg (``out``).

    Each leg has its own loss coefficient ``zeta_straight`` / ``zeta_branch``.
    The drop is reported on the corresponding leg port.
    """

    name: str
    cross_section: CrossSection
    zeta_straight: float = 0.0
    zeta_branch: float = 0.5
    ports: list[Port] = field(init=False)

    def __post_init__(self) -> None:
        self.ports = [
            Port(name="combined", direction="in"),
            Port(name="straight", direction="out"),
            Port(name="branch", direction="out"),
        ]

    def compute(self, fluid: Fluid) -> None:
        combined, straight, branch = self.ports
        if straight.flowrate is None or branch.flowrate is None:
            raise ValueError(f"Tee {self.name!r}: leg flowrates not set")
        a = self.cross_section.area
        rho = fluid.density
        v_s = straight.flowrate / a
        v_b = branch.flowrate / a
        combined.flowrate = straight.flowrate + branch.flowrate
        combined.velocity = combined.flowrate / a
        combined.pressure_drop = 0.0
        straight.velocity = v_s
        branch.velocity = v_b
        straight.pressure_drop = local_pressure_drop(self.zeta_straight, v_s, rho)
        branch.pressure_drop = local_pressure_drop(self.zeta_branch, v_b, rho)
