"""Duct sizing methods: equal friction, velocity method, and pressure drop budget.

These functions take a desired flowrate and return the smallest standard duct
size that meets the design criterion.
"""

from __future__ import annotations

from typing import Literal

from .core.fluid import Fluid, STANDARD_AIR
from .core.geometry import CrossSection, Rectangular, Round
from .data.standard_sizes import (
    STANDARD_RECTANGULAR_DUCT_SIZES,
    STANDARD_ROUND_DUCT_SIZES,
    nearest_round_size,
)
from .physics.friction import (
    friction_factor,
    relative_roughness,
    reynolds,
)
from .physics.losses import straight_pressure_drop


def velocity_method(
    flowrate: float,
    shape: Literal["round", "rectangular"] = "round",
    target_velocity: float = 4.0,
    *,
    absolute_roughness: float = 0.0001,
    fluid: Fluid = STANDARD_AIR,
) -> tuple[CrossSection, float]:
    """Size a duct by velocity method.

    Parameters
    ----------
    flowrate:
        Volumetric flow rate [m^3/s].
    shape:
        ``"round"`` or ``"rectangular"``.
    target_velocity:
        Target velocity [m/s]. Typical ranges: main ducts 3–5, branches 2–4,
        returns 1.5–3.
    absolute_roughness:
        Duct surface roughness [m]. Default is typical galvanized steel.
    fluid:
        Working fluid (default: standard air 20 °C, 101 325 Pa).

    Returns
    -------
    (cross_section, actual_velocity)
        The next-larger standard duct size and the resulting velocity.
    """
    if flowrate <= 0:
        raise ValueError(f"flowrate must be positive, got {flowrate}")
    if target_velocity <= 0:
        raise ValueError(f"target_velocity must be positive, got {target_velocity}")

    if shape == "round":
        # Find the smallest round size with v <= target_velocity.
        for d_mm in STANDARD_ROUND_DUCT_SIZES:
            d_m = d_mm / 1000
            section = Round(d_m)
            v = flowrate / section.area
            if v <= target_velocity:
                return section, v
        # If no size works, return the largest and warn.
        d_mm = STANDARD_ROUND_DUCT_SIZES[-1]
        d_m = d_mm / 1000
        section = Round(d_m)
        return section, flowrate / section.area
    else:  # rectangular
        # Find the smallest rectangular size with v <= target_velocity.
        for w_mm, h_mm in STANDARD_RECTANGULAR_DUCT_SIZES:
            w_m, h_m = w_mm / 1000, h_mm / 1000
            section = Rectangular(w_m, h_m)
            v = flowrate / section.area
            if v <= target_velocity:
                return section, v
        # Fallback to largest size.
        w_mm, h_mm = STANDARD_RECTANGULAR_DUCT_SIZES[-1]
        w_m, h_m = w_mm / 1000, h_mm / 1000
        section = Rectangular(w_m, h_m)
        return section, flowrate / section.area


def equal_friction_method(
    flowrate: float,
    target_pressure_drop_per_meter: float = 1.0,
    shape: Literal["round", "rectangular"] = "round",
    *,
    absolute_roughness: float = 0.0001,
    fluid: Fluid = STANDARD_AIR,
) -> tuple[CrossSection, float, float]:
    """Size a duct by equal friction method.

    The duct is sized so that the linear pressure drop (Pa/m) matches a
    target value, ensuring consistent performance across parallel branches.

    Parameters
    ----------
    flowrate:
        Volumetric flow rate [m^3/s].
    target_pressure_drop_per_meter:
        Target linear pressure drop [Pa/m]. Typical HVAC range is 0.5–1.5;
        low-velocity systems may use 0.3–0.5.
    shape:
        ``"round"`` or ``"rectangular"``.
    absolute_roughness:
        Duct surface roughness [m]. Default is typical galvanized steel.
    fluid:
        Working fluid (default: standard air 20 °C, 101 325 Pa).

    Returns
    -------
    (cross_section, velocity, pressure_drop_per_meter)
        The duct size, resulting velocity, and actual linear pressure drop.
    """
    if flowrate <= 0:
        raise ValueError(f"flowrate must be positive, got {flowrate}")
    if target_pressure_drop_per_meter <= 0:
        raise ValueError(
            f"target_pressure_drop_per_meter must be positive, "
            f"got {target_pressure_drop_per_meter}"
        )

    if shape == "round":
        # Iterate from smallest to largest, find one that meets the target.
        for d_mm in STANDARD_ROUND_DUCT_SIZES:
            d_m = d_mm / 1000
            section = Round(d_m)
            v = flowrate / section.area
            d_h = section.hydraulic_diameter
            re = reynolds(v, d_h, fluid.kinematic_viscosity)
            eps = relative_roughness(absolute_roughness, d_h)
            f = friction_factor(re, eps)
            # Pressure drop per meter: f * (rho * v^2 / 2) / d_h
            r = f / d_h * (fluid.density * v**2) / 2
            if r <= target_pressure_drop_per_meter:
                return section, v, r
        # Fallback: largest size.
        d_mm = STANDARD_ROUND_DUCT_SIZES[-1]
        d_m = d_mm / 1000
        section = Round(d_m)
        v = flowrate / section.area
        d_h = section.hydraulic_diameter
        re = reynolds(v, d_h, fluid.kinematic_viscosity)
        eps = relative_roughness(absolute_roughness, d_h)
        f = friction_factor(re, eps)
        r = f / d_h * (fluid.density * v**2) / 2
        return section, v, r
    else:  # rectangular
        for w_mm, h_mm in STANDARD_RECTANGULAR_DUCT_SIZES:
            w_m, h_m = w_mm / 1000, h_mm / 1000
            section = Rectangular(w_m, h_m)
            v = flowrate / section.area
            d_h = section.hydraulic_diameter
            re = reynolds(v, d_h, fluid.kinematic_viscosity)
            eps = relative_roughness(absolute_roughness, d_h)
            f = friction_factor(re, eps)
            r = f / d_h * (fluid.density * v**2) / 2
            if r <= target_pressure_drop_per_meter:
                return section, v, r
        # Fallback: largest size.
        w_mm, h_mm = STANDARD_RECTANGULAR_DUCT_SIZES[-1]
        w_m, h_m = w_mm / 1000, h_mm / 1000
        section = Rectangular(w_m, h_m)
        v = flowrate / section.area
        d_h = section.hydraulic_diameter
        re = reynolds(v, d_h, fluid.kinematic_viscosity)
        eps = relative_roughness(absolute_roughness, d_h)
        f = friction_factor(re, eps)
        r = f / d_h * (fluid.density * v**2) / 2
        return section, v, r


def pressure_drop_budget(
    flowrate: float,
    length: float,
    budget_pa: float,
    shape: Literal["round", "rectangular"] = "round",
    *,
    absolute_roughness: float = 0.0001,
    fluid: Fluid = STANDARD_AIR,
) -> tuple[CrossSection, float, float]:
    """Size a duct to fit within a total pressure drop budget.

    Parameters
    ----------
    flowrate:
        Volumetric flow rate [m^3/s].
    length:
        Duct length [m].
    budget_pa:
        Total allowable pressure drop [Pa] across this duct.
    shape:
        ``"round"`` or ``"rectangular"``.
    absolute_roughness:
        Duct surface roughness [m].
    fluid:
        Working fluid.

    Returns
    -------
    (cross_section, velocity, total_pressure_drop)
    """
    if length <= 0:
        raise ValueError(f"length must be positive, got {length}")
    if budget_pa <= 0:
        raise ValueError(f"budget_pa must be positive, got {budget_pa}")

    target_per_meter = budget_pa / length
    return equal_friction_method(
        flowrate,
        target_per_meter,
        shape,
        absolute_roughness=absolute_roughness,
        fluid=fluid,
    )
