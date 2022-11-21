from dataclasses import dataclass, field
from math import pi
from typing import Literal, Optional
import friction


@dataclass
class RigidDuctType:
    name: str
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
        # print("Start calculating RigidDuctType:", self.name)
        self.area = self.calc_cross_sectional_area()
        self.hydraulic_diameter = self.calc_hydraulic_diameter()
        self.surface_per_meter = self.calc_surface_area_per_meter()


@dataclass
class RigidDuct:
    name: str
    duct_type: RigidDuctType
    length: float
    flowrate: float
    roughness_correction_factor: float = 1
    velocity: float = field(init=False)
    pressure_drop_per_meter: float = field(init=False)
    linear_pressure_drop: float = field(init=False)
    surface_area: float = field(init=False)

    def calc_surface_area(self):
        return self.duct_type.surface_per_meter * self.length

    def calc_velocity(self):
        return self.flowrate / self.duct_type.area

    def calc_pressure_drop_per_meter(self):
        d_h = self.duct_type.hydraulic_diameter
        v = self.velocity
        k = self.duct_type.absolute_roughness
        Re = friction.reynolds(v, d_h)
        f = friction.friction_coefficient(Re, k, d_h)
        return friction.pressure_drop_per_meter(f, d_h, v)

    def calc_linear_pressure_drop(self):
        R = self.pressure_drop_per_meter
        L = self.length
        Beta = self.roughness_correction_factor
        return friction.linear_pressure_drop(R, L, Beta)

    def calculate(self) -> None:
        # print("Start calculating Duct:", self.name)
        self.surface_area = self.calc_surface_area()
        self.velocity = self.calc_velocity()
        self.pressure_drop_per_meter = self.calc_pressure_drop_per_meter()
        self.linear_pressure_drop = self.calc_linear_pressure_drop()


@dataclass
class FlexDuctType:
    name: str
    absolute_roughness: float
    diameter: float
    area: float = field(init=False)
    hydraulic_diameter: float = field(init=False)

    def calc_cross_sectional_area(self):
        return pi * (self.diameter / 2) ** 2

    def calc_hydraulic_diameter(self):
        return self.diameter

    def calculate(self):
        # print("Start calculating FlexDuctType:", self.name)
        self.area = self.calc_cross_sectional_area()
        self.hydraulic_diameter = self.calc_hydraulic_diameter()


@dataclass
class FlexDuct:
    name: str
    duct_type: FlexDuctType
    length: float
    flowrate: float
    stretch_percentage: float
    velocity: float = field(init=False)
    stretch_correction_factor: float = field(init=False)
    pressure_drop_per_meter: float = field(init=False)
    linear_pressure_drop: float = field(init=False)

    def calc_velocity(self):
        return self.flowrate / self.duct_type.area

    def calc_stretch_correction_factor(self):
        stretch_percentage = self.stretch_percentage
        if stretch_percentage == 1:
            return 1
        else:
            diameter = self.duct_type.hydraulic_diameter
            return friction.flex_stretch_correction_factor(diameter, stretch_percentage)

    def calc_pressure_drop_per_meter(self):
        diameter = self.duct_type.diameter
        V = self.flowrate
        return friction.flex_pressure_drop_per_meter(diameter, V)

    def calc_linear_pressure_drop(self):
        R = self.pressure_drop_per_meter
        L = self.length
        return friction.linear_pressure_drop(R, L, 1) * self.stretch_correction_factor

    def calculate(self) -> None:
        # print("Start calculating Duct:", self.name)
        self.velocity = self.calc_velocity()
        self.stretch_correction_factor = self.calc_stretch_correction_factor()
        self.pressure_drop_per_meter = self.calc_pressure_drop_per_meter()
        self.linear_pressure_drop = self.calc_linear_pressure_drop()