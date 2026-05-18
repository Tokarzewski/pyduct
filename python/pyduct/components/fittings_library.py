"""Loss coefficient library for common HVAC fittings.

Mojo-backed shim — every correlation runs in ``wenta.ext.fittings_ext``.
Coefficients are from ASHRAE Fundamentals and ductwork design guides
(Hendiger, Idelchik). The Python import path stays the same.
"""

from __future__ import annotations

from wenta.ext.fittings_ext import (
    damper_butterfly as _damper_butterfly,
    diffuser_ceiling as _diffuser_ceiling,
    expander_round as _expander_round,
    grille_return as _grille_return,
    junction_tee_branch as _junction_tee_branch,
    junction_tee_combine as _junction_tee_combine,
    mitered_elbow as _mitered_elbow,
    rectangular_elbow as _rectangular_elbow,
    reducer_round as _reducer_round,
)

from .._mojo_shim import translate_error as _v


def reducer_round(d_inlet: float, d_outlet: float, angle_deg: float = 45) -> float:
    """Loss coefficient for a round reducer (ASHRAE correlation)."""
    return _v(_reducer_round, d_inlet, d_outlet, angle_deg)


def expander_round(d_inlet: float, d_outlet: float, angle_deg: float = 45) -> float:
    """Loss coefficient for a round expander / diffuser (Borda–Carnot baseline)."""
    return _v(_expander_round, d_inlet, d_outlet, angle_deg)


def junction_tee_branch(
    d_main: float, d_branch: float, flowrate_main: float, flowrate_branch: float
) -> tuple[float, float]:
    """``(zeta_main, zeta_branch)`` for a splitting tee."""
    return _v(_junction_tee_branch, d_main, d_branch, flowrate_main, flowrate_branch)


def junction_tee_combine(
    d_main: float, d_branch: float, flowrate_main: float, flowrate_branch: float
) -> tuple[float, float]:
    """``(zeta_main, zeta_branch)`` for a combining tee."""
    return _v(_junction_tee_combine, d_main, d_branch, flowrate_main, flowrate_branch)


def damper_butterfly(open_percentage: float = 100.0) -> float:
    """Butterfly-damper loss coefficient (0–100 % open)."""
    return _v(_damper_butterfly, open_percentage)


def diffuser_ceiling(area_throw: float = 1.0) -> float:
    """Ceiling-diffuser face-velocity loss coefficient."""
    return _v(_diffuser_ceiling, area_throw)


def grille_return(blockage_factor: float = 0.15) -> float:
    """Return-grille face-velocity loss coefficient."""
    return _v(_grille_return, blockage_factor)


def rectangular_elbow(
    width: float, height: float, bend_radius: float, angle_deg: float = 90.0
) -> float:
    """Smooth-radius rectangular elbow (Idelchik §6) with aspect correction."""
    return _v(_rectangular_elbow, width, height, bend_radius, angle_deg)


def mitered_elbow(angle_deg: float = 90.0, *, vaned: bool = False) -> float:
    """Sharp-corner mitered elbow; ``vaned=True`` cuts the loss to ~40 %."""
    return _v(_mitered_elbow, angle_deg, vaned)
