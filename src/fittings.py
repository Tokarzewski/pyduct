from dataclasses import dataclass
from typing import Optional


@dataclass
class OneWayFitting:
    name: str
    connector1: Optional[str] = None
    
    def connect(self, duct, other_fitting):
        self.connector1 = other_fitting.name
        # XYZ record this change in the netowrkx.Graph()
        print(self.name, "connected through", duct.name, "to", self.connector1)


@dataclass
class TwoWayFitting:
    name: str
    connector1: Optional[str] = None
    connector2: Optional[str] = None

@dataclass
class ThreeWayFitting:
    name: str
    connector1: str
    connector2: str
    connector3: str

@dataclass
class FourWayFitting:
    name: str
    connector1: str
    connector2: str
    connector3: str
    connector4: str