from dataclasses import dataclass
from typing import Optional


@dataclass
class Connector:
    id: str = None
    flowrate: Optional[float] = None
    area: Optional[float] = None
    dzeta: Optional[float] = None
    pressure_drop: Optional[float] = None