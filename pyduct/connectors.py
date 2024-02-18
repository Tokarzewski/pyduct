from dataclasses import dataclass
from typing import Optional, Literal


@dataclass
class Connector:
    id: str = None
    shape: Literal["round", "rectangular"] = "round"
    diameter: float = None
    width: float = None
    height: float = None
    flowrate: Optional[float] = None
    area: Optional[float] = None
    dzeta: Optional[float] = None
    pressure_drop: Optional[float] = None
