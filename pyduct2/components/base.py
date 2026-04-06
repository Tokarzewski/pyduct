"""Base classes shared by all ductwork components."""

from __future__ import annotations

from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from typing import Literal, Optional

from ..core.fluid import Fluid

PortDirection = Literal["in", "out"]


@dataclass
class Port:
    """A connection point on a :class:`Component`.

    A `Port` carries the local flow state (flowrate, velocity) and the local
    pressure drop attributed to it. Ports are owned by exactly one component
    and identified by their `name` within that component.

    The `direction` field follows the *physical* airflow convention:

    * ``"in"``  — air enters the component through this port (e.g. duct inlet,
      tee combined inlet, terminal inlet);
    * ``"out"`` — air leaves the component through this port (e.g. duct outlet,
      tee straight/branch legs, source/AHU outlet).
    """

    name: str
    direction: PortDirection
    flowrate: Optional[float] = None  # [m^3/s]
    velocity: Optional[float] = None  # [m/s]
    pressure_drop: float = 0.0        # [Pa]


class Component(ABC):
    """A piece of ductwork (duct, fitting, terminal) with one or more ports.

    Subclasses must:

    * populate ``self.ports`` in their ``__post_init__``;
    * implement :meth:`compute` to fill in velocity and pressure_drop on each
      port given the upstream flowrates and a :class:`Fluid`.
    """

    name: str
    ports: list[Port]

    @abstractmethod
    def compute(self, fluid: Fluid) -> None:
        """Populate ``velocity`` and ``pressure_drop`` on each port.

        Called by the solver after flowrates have been propagated through the
        network. The flowrate on each port is already set when this is called.
        """

    # Convenience helpers used by Network --------------------------------------

    def port(self, name: str) -> Port:
        for p in self.ports:
            if p.name == name:
                return p
        raise KeyError(
            f"{type(self).__name__} {self.name!r} has no port named {name!r}"
        )

    def inlets(self) -> list[Port]:
        return [p for p in self.ports if p.direction == "in"]

    def outlets(self) -> list[Port]:
        return [p for p in self.ports if p.direction == "out"]
