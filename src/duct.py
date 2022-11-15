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
    surface_per_meter: float = field(init=False)

    def calc_cross_sectional_area(self):
        if self.shape == "round":
            return pi * (self.diameter / 2) ** 2
        else:
            return self.height * self.width

    def calc_hydraulic_diameter(self):
        if self.shape == "round":
            return self.diameter
        else:
            return 2 * self.height * self.width / (self.height + self.width)

    def calc_surface_area_per_meter(self):
        if self.shape == "round":
            return pi * self.diameter
        else:
            return 2 * (self.height + self.width)

    def calculate(self):
        # print("Start calculating ducttype:", self.name)
        self.area = self.calc_cross_sectional_area()
        self.hydraulic_diameter = self.calc_hydraulic_diameter()
        self.surface_per_meter = self.calc_surface_area_per_meter()


@dataclass
class duct:
    name: str
    ducttype: ducttype
    length: float
    flowrate: float
    roughness_correction_factor: float = 1
    velocity: float = field(init=False)
    pressure_drop_per_meter: float = field(init=False)
    linear_pressure_drop: float = field(init=False)
    surface_area: float = field(init=False)

    def calc_surface_area(self):
        return self.ducttype.surface_per_meter * self.length

    def calc_velocity(self):
        return self.flowrate / self.ducttype.area

    def calc_pressure_drop_per_meter(self):
        k = self.ducttype.absolute_roughness
        d_h = self.ducttype.hydraulic_diameter
        v = self.velocity
        Re = friction.reynolds(v, d_h)
        f = friction.friction_coefficient(Re, k, d_h)
        return friction.pressure_drop_per_meter(f, d_h, v)

    def calc_roughness_correction_factor(self):
        absolute_roughness = self.ducttype.absolute_roughness
        velocity = self.velocity
        diameter = self.ducttype.hydraulic_diameter
        return friction.roughness_correction_factor(
            absolute_roughness, diameter, velocity
        )

    def calc_linear_pressure_drop(self):
        R = self.pressure_drop_per_meter
        L = self.length
        Beta = self.roughness_correction_factor
        return friction.linear_pressure_drop(R, L, Beta)

    def calculate(self) -> None:
        # print("Start calculating duct:", self.name)
        self.surface_area = self.calc_surface_area()
        self.velocity = self.calc_velocity()
        self.pressure_drop_per_meter = self.calc_pressure_drop_per_meter()
        if self.roughness_correction_factor == "autosize":
            self.roughness_correction_factor = self.calc_roughness_correction_factor()
        self.linear_pressure_drop = self.calc_linear_pressure_drop()

