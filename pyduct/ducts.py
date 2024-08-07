from dataclasses import dataclass, field
from math import pi
from typing import List, Literal, Optional
from .connectors import Connector
from .physics.friction import *

@dataclass
class RigidDuctType:
    name: str
    shape: Literal["round", "rectangular"] = "round"
    absolute_roughness: float = 0.0001
    diameter: Optional[float] = None
    height: Optional[float] = None
    width: Optional[float] = None
    cross_sectional_area: Optional[float] = None
    hydraulic_diameter: Optional[float] = None

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

    def calculate(self):
        self.cross_sectional_area = self.calc_cross_sectional_area()
        self.hydraulic_diameter = self.calc_hydraulic_diameter()


@dataclass
class FlexDuctType:
    name: str
    absolute_roughness: float
    diameter: float
    cross_sectional_area: float = field(init=False)
    hydraulic_diameter: float = field(init=False)

    def calc_cross_sectional_area(self):
        return pi * (self.diameter / 2) ** 2

    def calc_hydraulic_diameter(self):
        return self.diameter

    def calculate(self):
        self.cross_sectional_area = self.calc_cross_sectional_area()
        self.hydraulic_diameter = self.calc_hydraulic_diameter()


@dataclass
class RigidDuct:
    name: str
    duct_type: RigidDuctType
    length: float
    flowrate: Optional[float] = None
    roughness_correction_factor: Optional[float] = None
    connectors = [Connector(), Connector()]
    velocity: Optional[str] = None
    pressure_drop_per_meter: Optional[str] = None
    linear_pressure_drop: Optional[str] = None

    def calc_velocity(self):
        self.flowrate = self.connectors[0].flowrate
        return self.flowrate / self.duct_type.cross_sectional_area

    def calc_pressure_drop_per_meter(self):
        d_h = self.duct_type.hydraulic_diameter
        v = self.velocity
        k = self.duct_type.absolute_roughness
        E = relative_roughness(k, d_h)
        Re = reynolds(v, d_h)
        f = friction_coefficient(Re, E)
        return pressure_drop_per_meter(f, d_h, v)

    def calc_linear_pressure_drop(self):
        R = self.pressure_drop_per_meter
        L = self.length
        # Beta = self.roughness_correction_factor
        Beta = 1
        return linear_pressure_drop(R, L, Beta)

    def calculate(self) -> None:
        self.duct_type.calculate()
        self.velocity = self.calc_velocity()
        self.pressure_drop_per_meter = self.calc_pressure_drop_per_meter()
        self.linear_pressure_drop = self.calc_linear_pressure_drop()
        self.connectors[0].pressure_drop = self.linear_pressure_drop
        self.connectors[1].pressure_drop = 0


@dataclass
class FlexDuct:
    name: str
    duct_type: FlexDuctType
    length: float
    flowrate: float
    stretch_percentage: float
    connectors = [Connector(), Connector()]
    velocity: float = field(init=False)
    stretch_correction_factor: Optional[float] = None
    pressure_drop_per_meter: float = field(init=False)
    linear_pressure_drop: float = field(init=False)

    def calc_velocity(self):
        return self.flowrate / self.duct_type.cross_sectional_area

    def calc_stretch_correction_factor(self):
        stretch_percentage = self.stretch_percentage
        if stretch_percentage == 1:
            return 1
        else:
            diameter = self.duct_type.hydraulic_diameter
            return flex_stretch_correction_factor(diameter, stretch_percentage)

    def calc_pressure_drop_per_meter(self):
        diameter = self.duct_type.diameter
        V = self.flowrate
        return flex_pressure_drop_per_meter(diameter, V)

    def calc_linear_pressure_drop(self):
        R = self.pressure_drop_per_meter
        L = self.length
        return linear_pressure_drop(R, L, 1) * self.stretch_correction_factor

    def calculate(self) -> None:
        self.velocity = self.calc_velocity()
        self.stretch_correction_factor = self.calc_stretch_correction_factor()
        self.pressure_drop_per_meter = self.calc_pressure_drop_per_meter()
        self.linear_pressure_drop = self.calc_linear_pressure_drop()
        self.connectors[0].pressure_drop = self.linear_pressure_drop
        self.connectors[1].pressure_drop = 0
