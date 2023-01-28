from dataclasses import dataclass, field
from typing import Optional
import friction


@dataclass
class Connector:
    flowrate: Optional[float] = 0
    pressure_drop: Optional[float] = None
    area: Optional[float] = None
    dzeta: Optional[float] = None


@dataclass
class OneWayFitting:
    name: str
    # type
    connector1: Connector


@dataclass
class TwoWayFitting:
    name: str
    type: str
    connector1: Connector = field(init=False)
    connector2: Connector = field(init=False)

    def __post_init__(self) -> None:
        self.connector1 = Connector(0)
        self.connector2 = Connector(0)

    def calculate(self) -> None:
        # arguments width, length, diameter, area, flowrate, velocity
        if self.connector2.flowrate > 0:
            self.connector2.area = 1
            v2 = self.connector2.flowrate / self.connector2.area
            self.connector2.dzeta = self.type.dzeta()
            self.connector2.pressure_drop = friction.local_pressure_drop(
                self.connector2.dzeta, v2)


@dataclass
class ThreeWayFitting:
    name: str
    # type
    connector1: Connector = field(init=False)
    connector2: Connector = field(init=False)
    connector3: Connector = field(init=False)

    def __post_init__(self) -> None:
        self.connector1 = Connector(0)
        self.connector2 = Connector(0)
        self.connector3 = Connector(0)

    def calculate(self) -> None:
        # arguments width, length, diameter, area, flowrate, velocity
        if self.connector2.flowrate > 0:
            self.connector2.area = 1
            v2 = self.connector2.flowrate / self.connector2.area
            self.connector2.dzeta = 0.5
            self.connector2.pressure_drop = friction.local_pressure_drop(
                self.connector2.dzeta, v2)

        if self.connector3.flowrate > 0:
            self.connector3.area = 1
            v3 = self.connector3.flowrate / self.connector3.area
            self.connector3.dzeta = 0.3
            self.connector3.pressure_drop = friction.local_pressure_drop(
                self.connector3.dzeta, v3)


@dataclass
class FourWayFitting:
    name: str
    # type
    connector1: Connector = field(init=False)
    connector2: Connector = field(init=False)
    connector3: Connector = field(init=False)
    connector4: Connector = field(init=False)

    def __post_init__(self) -> None:
        self.connector1 = Connector(0)
        self.connector2 = Connector(0)
        self.connector3 = Connector(0)
        self.connector4 = Connector(0)
