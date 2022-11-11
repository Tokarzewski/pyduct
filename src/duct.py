from dataclasses import dataclass, field
from math import pi
from typing import Literal, Optional
import friction


@dataclass
class ducttype:
    name: str
    type: Literal["rigid", "flex"]
    shape: Literal["round", "rectangular"]
    absolute_roughness: float
    diameter: Optional[float] = None
    height: Optional[float] = None
    width: Optional[float] = None
    area: float = field(init=False)
    hydraulic_diameter: float = field(init=False)

    def calc_area(self):
        if self.shape == "round":
            return pi * (self.diameter / 2) ** 2
        else:
            return self.height * self.width

    def calc_hydraulic_diameter(self):
        if self.shape == "round":
            return self.diameter
        else:
            return 2 * self.height * self.width / (self.height + self.width)

    def __post_init__(self):
        self.area = self.calc_area()
        self.hydraulic_diameter = self.calc_hydraulic_diameter()


@dataclass
class duct:
    name: str
    ducttype: ducttype
    length: float
    flowrate: float
    velocity: float = field(init=False)
    pressure_drop_per_meter: float = field(init=False)
    linear_pressure_drop: float = field(init=False)

    def calc_velocity(self):
        return self.flowrate / self.ducttype.area

    def calc_pressure_drop_per_meter(self):
        k = self.ducttype.absolute_roughness
        d_h = self.ducttype.hydraulic_diameter
        v = self.velocity
        Re = friction.reynolds(v, d_h)
        f = friction.friction_coefficient(Re, k, d_h)
        return friction.pressure_drop_per_meter(f, d_h, v)

    def calc_linear_pressure_drop(self):
        R = self.pressure_drop_per_meter
        L = self.length
        return friction.linear_pressure_drop(R, L)

    def __post_init__(self) -> None:
        self.velocity = self.calc_velocity()
        self.pressure_drop_per_meter = self.calc_pressure_drop_per_meter()
        self.linear_pressure_drop = self.calc_linear_pressure_drop()