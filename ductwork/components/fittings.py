from dataclasses import dataclass, field
from typing import Optional, List
import friction
from components.connectors import Connector

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
        if self.connector[2].flowrate > 0:
            self.connector[2].area = 1
            v2 = self.connector[2].flowrate / self.connector[2].area
            self.connector[2].dzeta = self.type.dzeta()
            self.connector[2].pressure_drop = friction.local_pressure_drop(
                self.connector[2].dzeta, v2)


@dataclass
class ThreeWayFitting:
    name: str
    # type
    connectors: Optional[List[Connector]] = None
    number_of_connectors: int = 3

    def __post_init__(self) -> None:
        self.connectors = [Connector("1"), Connector("2"), Connector("3")]

    def calculate(self) -> None:
        # arguments width, length, diameter, area, flowrate, velocity
        if self.connectors[2].flowrate > 0:
            self.connector[2].area = 1
            v2 = self.connector[2].flowrate / self.connector[2].area
            self.connector[2].dzeta = 0.5
            self.connector[2].pressure_drop = friction.local_pressure_drop(
                self.dzeta, v2)

        if self.connector[3].flowrate > 0:
            self.connector[3].area = 1
            v3 = self.connector[3].flowrate / self.connector[3].area
            self.connector[3].dzeta = 0.3
            self.connector[3].pressure_drop = friction.local_pressure_drop(
                self.connector[3].dzeta, v3)


@dataclass
class FourWayFitting:
    name: str
    # type
    connectors: Optional[List[Connector]]
    number_of_connectors: int = 4

    def __post_init__(self) -> None:
        self.connectors = [Connector("1"), Connector("2"), Connector("3"), Connector("4")]
