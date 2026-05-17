"""Duct sizing methods: velocity, equal-friction, and pressure-drop budget.

Each method picks the smallest EN-standard duct size that meets the design
criterion. All three share a single iterator helper, ``_smallest_meeting``,
so adding new sizing strategies is a one-function exercise.
"""

from __future__ import annotations

from collections.abc import Callable, Iterable
from typing import Literal

from .core.fluid import STANDARD_AIR, Fluid
from .core.geometry import CrossSection
from .data.standard_sizes import (
    STANDARD_RECTANGULAR_SECTIONS,
    STANDARD_ROUND_SECTIONS,
)
from .physics.friction import friction_factor, relative_roughness, reynolds

Shape = Literal["round", "rectangular"]


def _sections_for(shape: Shape) -> Iterable[CrossSection]:
    return STANDARD_ROUND_SECTIONS if shape == "round" else STANDARD_RECTANGULAR_SECTIONS


def _smallest_meeting(
    sections: Iterable[CrossSection],
    evaluator: Callable[[CrossSection], float],
    target: float,
) -> tuple[CrossSection, float]:
    """Return ``(section, value)`` for the smallest section whose evaluated
    value is ≤ ``target``; fall back to the largest section if none match."""
    last_section: CrossSection | None = None
    last_value = 0.0
    for section in sections:
        last_value = evaluator(section)
        last_section = section
        if last_value <= target:
            return section, last_value
    if last_section is None:
        raise ValueError("no standard sections available")
    return last_section, last_value


def velocity_method(
    flowrate: float,
    shape: Shape = "round",
    target_velocity: float = 4.0,
    *,
    absolute_roughness: float = 0.0001,
    fluid: Fluid = STANDARD_AIR,
) -> tuple[CrossSection, float]:
    """Size a duct so velocity ≤ ``target_velocity``.

    Returns ``(cross_section, actual_velocity)``. Typical targets: main ducts
    3–5 m/s, branches 2–4 m/s, returns 1.5–3 m/s.
    """
    if flowrate <= 0:
        raise ValueError(f"flowrate must be positive, got {flowrate}")
    if target_velocity <= 0:
        raise ValueError(f"target_velocity must be positive, got {target_velocity}")
    return _smallest_meeting(
        _sections_for(shape),
        lambda s: flowrate / s.area,
        target_velocity,
    )


def equal_friction_method(
    flowrate: float,
    target_pressure_drop_per_meter: float = 1.0,
    shape: Shape = "round",
    *,
    absolute_roughness: float = 0.0001,
    fluid: Fluid = STANDARD_AIR,
) -> tuple[CrossSection, float, float]:
    """Size a duct so linear pressure drop ≤ ``target_pressure_drop_per_meter``.

    Returns ``(cross_section, velocity, pressure_drop_per_meter)``. Typical
    HVAC range: 0.5–1.5 Pa/m; low-velocity systems 0.3–0.5 Pa/m.
    """
    if flowrate <= 0:
        raise ValueError(f"flowrate must be positive, got {flowrate}")
    if target_pressure_drop_per_meter <= 0:
        raise ValueError(
            "target_pressure_drop_per_meter must be positive, "
            f"got {target_pressure_drop_per_meter}"
        )
    nu = fluid.kinematic_viscosity
    rho = fluid.density

    def dp_per_m(s: CrossSection) -> float:
        v = flowrate / s.area
        d_h = s.hydraulic_diameter
        f = friction_factor(reynolds(v, d_h, nu), relative_roughness(absolute_roughness, d_h))
        return f / d_h * (rho * v**2) / 2

    section, r = _smallest_meeting(_sections_for(shape), dp_per_m, target_pressure_drop_per_meter)
    return section, flowrate / section.area, r


def pressure_drop_budget(
    flowrate: float,
    length: float,
    budget_pa: float,
    shape: Shape = "round",
    *,
    absolute_roughness: float = 0.0001,
    fluid: Fluid = STANDARD_AIR,
) -> tuple[CrossSection, float, float]:
    """Size a duct so total pressure drop across ``length`` ≤ ``budget_pa``.

    Returns ``(cross_section, velocity, pressure_drop_per_meter)``.
    """
    if length <= 0:
        raise ValueError(f"length must be positive, got {length}")
    if budget_pa <= 0:
        raise ValueError(f"budget_pa must be positive, got {budget_pa}")
    return equal_friction_method(
        flowrate,
        budget_pa / length,
        shape,
        absolute_roughness=absolute_roughness,
        fluid=fluid,
    )
