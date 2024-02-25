from dataclasses import dataclass
from typing import Optional, Literal
from . import friction  # Ensure this import is correct based on your project structure

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

    def calculate_pressure_drop(self, area: float, dzeta: float) -> None:
        """
        Calculate the pressure drop at the connector interface.
        """
        if self.flowrate > 0:
            self.area = area
            velocity = self.flowrate / self.area
            self.dzeta = dzeta
            self.pressure_drop = friction.local_pressure_drop(self.dzeta, velocity)
