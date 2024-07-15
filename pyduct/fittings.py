from dataclasses import dataclass, field
from typing import List
from .connectors import Connector


@dataclass
class OneWayFitting:
    name: str
    flowrate: float
    connectors: Connector = field(init=False)
    # type: str

    def __post_init__(self) -> None:
        self.connectors = [Connector(flowrate=self.flowrate)]

    def calculate(self) -> None:
        c1 = self.connectors[0]
        c1.calculate_pressure_drop(area=1, dzeta=0.5)


@dataclass
class TwoWayFitting:
    name: str
    fitting_type: str
    connectors: List[Connector] = field(init=False)

    def __post_init__(self):
        self.connectors = [Connector(), Connector()]

    def calculate(self) -> None:
        c1 = self.connectors[0]
        c2 = self.connectors[1]

        c1.pressure_drop = 0
        c2.calculate_pressure_drop(area=1, dzeta=self.fitting_type.dzeta)


@dataclass
class ThreeWayFitting:
    """c1 source, c2 straight, c3 branch"""

    name: str
    connectors: List[Connector] = field(init=False)
    #fitting_type: str

    def __post_init__(self):
        self.connectors = [Connector(), Connector(), Connector()]

    def calculate(self) -> None:
        # c1 = self.connectors[0]
        c2 = self.connectors[1]
        c3 = self.connectors[2]

        c2.calculate_pressure_drop(area=1, dzeta=0.5)
        c3.calculate_pressure_drop(area=1, dzeta=0.5)
