from dataclasses import dataclass
from typing import Optional, List
from .connectors import Connector
from . import friction

@dataclass
class OneWayFitting:
    name: str
    # type
    connectors: Connector
    number_of_connectors: int = 1

    def __post_init__(self) -> None:
        self.connectors.id = "1"

@dataclass
class TwoWayFitting:
    name: str
    type: str
    connectors: Optional[List[Connector]] = None
    number_of_connectors: int = 2

    def __post_init__(self) -> None:
        self.connectors = [Connector("1"), Connector("2")]

    def calculate(self) -> None:
        # arguments width, length, diameter, area, flowrate, velocity
        c2 = self.connector[2]
        if c2.flowrate > 0:
            c2.area = 1
            v2 = c2.flowrate / c2.area
            c2.dzeta = self.type.dzeta()
            c2.pressure_drop = friction.local_pressure_drop(c2.dzeta, v2)


@dataclass
class ThreeWayFitting:
    name: str
    #type: str
    connectors: Optional[List[Connector]] = None
    number_of_connectors: int = 3

    def __post_init__(self) -> None:
        self.connectors = [Connector("1"), Connector("2"), Connector("3")]

    def calculate(self) -> None:
        # arguments width, length, diameter, area, flowrate, velocity
        c2 = self.connector[2]
        c3 = self.connector[3]

        if self.connectors[2].flowrate > 0:
            c2.area = 1
            v2 = c2.flowrate / c2.area
            c2.dzeta = 0.5
            c2.pressure_drop = friction.local_pressure_drop(c2.dzeta, v2)

        if c3.flowrate > 0:
            c3.area = 1
            v3 = c3.flowrate / c3.area
            c3.dzeta = 0.3
            c3.pressure_drop = friction.local_pressure_drop(c3.dzeta, v3)


@dataclass
class FourWayFitting:
    name: str
    #type: str
    connectors: Optional[List[Connector]]
    number_of_connectors: int = 4

    def __post_init__(self) -> None:
        self.connectors = [Connector("1"), Connector("2"), Connector("3"), Connector("4")]
