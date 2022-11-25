from dataclasses import dataclass
from typing import Optional


@dataclass
class OneWayFitting:
    name: str
    flowrate: float
    connector1: Optional[str] = None


@dataclass
class TwoWayFitting:
    name: str
    connector1: Optional[str] = None
    connector2: Optional[str] = None


@dataclass
class ThreeWayFitting:
    name: str
    connector1: Optional[str] = None
    connector2: Optional[str] = None
    connector3: Optional[str] = None


@dataclass
class FourWayFitting:
    name: str
    connector1: Optional[str] = None
    connector2: Optional[str] = None
    connector3: Optional[str] = None
    connector4: Optional[str] = None
