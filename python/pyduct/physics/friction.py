"""Friction-related correlations for duct flow — Mojo-backed shim.

Math lives in ``wenta.physics.friction``; this module is a thin Python
wrapper. The Python import path stays the same.
"""

from __future__ import annotations

from wenta.ext.friction_ext import (
    friction_factor as _friction_factor,
)
from wenta.ext.friction_ext import (
    friction_factor_colebrook as _friction_factor_colebrook,
)
from wenta.ext.friction_ext import (
    relative_roughness as _relative_roughness,
)
from wenta.ext.friction_ext import (
    reynolds as _reynolds,
)

LAMINAR_RE_LIMIT = 2300.0


def reynolds(
    velocity: float, hydraulic_diameter: float, kinematic_viscosity: float
) -> float:
    """Reynolds number ``Re = v * D_h / nu``."""
    return _reynolds(velocity, hydraulic_diameter, kinematic_viscosity)


def relative_roughness(absolute_roughness: float, hydraulic_diameter: float) -> float:
    """Relative roughness ``epsilon / D_h``."""
    return _relative_roughness(absolute_roughness, hydraulic_diameter)


def friction_factor(reynolds_number: float, rel_roughness: float) -> float:
    """Darcy friction factor (Swamee–Jain explicit approximation).

    Laminar fallback ``64 / Re`` for Re < 2300; turbulent expression valid
    for ``5e3 < Re < 1e8`` and ``1e-5 < eps/D_h < 5e-1``.
    """
    return _friction_factor(reynolds_number, rel_roughness)


def friction_factor_colebrook(
    reynolds_number: float,
    rel_roughness: float,
    *,
    tol: float = 1e-12,
    max_iter: int = 100,
) -> float:
    """Implicit Colebrook–White friction factor (fixed-point iteration).

    Slower but more precise than :func:`friction_factor`. The Mojo
    implementation uses the same fixed-point algorithm with a Swamee–Jain
    seed. ``tol`` and ``max_iter`` are honoured by the Mojo defaults
    (1e-12 / 100); explicit overrides are not currently forwarded.
    """
    # The Mojo function uses tol=1e-12, max_iter=100 internally — same as
    # the original Python defaults. Custom values are not yet forwarded
    # through the FFI boundary; if you need a different tolerance, file
    # an issue or call the wenta module directly.
    return _friction_factor_colebrook(reynolds_number, rel_roughness)
