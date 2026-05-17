"""Friction-related correlations for duct flow."""

from __future__ import annotations

from math import log, log10, sqrt

LAMINAR_RE_LIMIT = 2300.0


def reynolds(
    velocity: float, hydraulic_diameter: float, kinematic_viscosity: float
) -> float:
    """Reynolds number ``Re = v * D_h / nu``."""
    return velocity * hydraulic_diameter / kinematic_viscosity


def relative_roughness(absolute_roughness: float, hydraulic_diameter: float) -> float:
    """Relative roughness ``epsilon / D_h``."""
    return absolute_roughness / hydraulic_diameter


def friction_factor(reynolds_number: float, rel_roughness: float) -> float:
    """Darcy friction factor (Swamee–Jain explicit approximation).

    Falls back to laminar ``64 / Re`` for Re < 2300. The turbulent expression is
    valid for ``5e3 < Re < 1e8`` and ``1e-5 < eps/D_h < 5e-1``.

    Reference
    ---------
    Swamee, P. K. and Jain, A. K. (1976). *Explicit equations for pipe-flow
    problems.* Journal of the Hydraulics Division, ASCE, 102(HY5), 657-664.
    """
    if reynolds_number < LAMINAR_RE_LIMIT:
        return 64.0 / reynolds_number
    return 1.613 * (
        log(
            0.234 * rel_roughness ** 1.1007
            - 60.525 / reynolds_number ** 1.1105
            + 56.291 / reynolds_number ** 1.0712
        )
    ) ** -2


def friction_factor_colebrook(
    reynolds_number: float,
    rel_roughness: float,
    *,
    tol: float = 1e-12,
    max_iter: int = 100,
) -> float:
    """Darcy friction factor from the implicit Colebrook–White equation.

    Solved by simple fixed-point iteration starting from the Swamee–Jain guess.
    Slower than :func:`friction_factor` but used as a reference / source of
    truth in tests. Has no SciPy dependency.
    """
    if reynolds_number < LAMINAR_RE_LIMIT:
        return 64.0 / reynolds_number

    f = friction_factor(reynolds_number, rel_roughness)
    for _ in range(max_iter):
        rhs = -2 * log10(
            rel_roughness / 3.71 + 2.51 / (reynolds_number * sqrt(f))
        )
        f_new = 1.0 / rhs ** 2
        if abs(f_new - f) < tol:
            return f_new
        f = f_new
    return f
