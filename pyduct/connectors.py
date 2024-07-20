from dataclasses import dataclass
from typing import Optional, Literal
from .physics.friction import local_pressure_drop  # Ensure this import is correct based on your project structure
from .physics.general import calc_velocity

@dataclass
class Connector:
    id: str = None
    shape: Literal["round", "rectangular"] = "round"
    diameter: float = None
    width: float = None
    height: float = None
    flowrate: Optional[float] = None
    area: Optional[float] = None
    velocity: Optional[float] = None
    dzeta: Optional[float] = None
    pressure_drop: Optional[float] = None

    def calculate_velocity(self) -> float:
        self.velocity = calc_velocity(self.flowrate, self.area)

    def calculate_pressure_drop(self, area: float, dzeta: float) -> None:
        """
        Calculate the pressure drop at the connector interface.
        """
        self.area = area
        self.dzeta = dzeta
        self.calculate_velocity()
        self.pressure_drop = local_pressure_drop(self.dzeta, self.velocity)
