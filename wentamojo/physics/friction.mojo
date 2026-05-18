"""Friction-related correlations for duct flow (Mojo port).

Mirrors `wenta.physics.friction`:

* `reynolds(velocity, hydraulic_diameter, kinematic_viscosity)`
* `relative_roughness(absolute_roughness, hydraulic_diameter)`
* `friction_factor(reynolds_number, rel_roughness)` — Swamee–Jain explicit
* `friction_factor_colebrook(...)` — implicit Colebrook–White via fixed-point
"""

from std.math import log, log10, sqrt


comptime LAMINAR_RE_LIMIT: Float64 = 2300.0


def reynolds(
    velocity: Float64, hydraulic_diameter: Float64, kinematic_viscosity: Float64
) -> Float64:
    """Reynolds number Re = v * D_h / nu."""
    return velocity * hydraulic_diameter / kinematic_viscosity


def relative_roughness(absolute_roughness: Float64, hydraulic_diameter: Float64) -> Float64:
    """Relative roughness epsilon / D_h."""
    return absolute_roughness / hydraulic_diameter


def friction_factor(reynolds_number: Float64, rel_roughness: Float64) -> Float64:
    """Darcy friction factor (Swamee–Jain explicit approximation).

    Falls back to laminar `64 / Re` for Re < 2300.
    """
    if reynolds_number < LAMINAR_RE_LIMIT:
        return 64.0 / reynolds_number
    var arg = (
        0.234 * rel_roughness ** 1.1007
        - 60.525 / reynolds_number ** 1.1105
        + 56.291 / reynolds_number ** 1.0712
    )
    var l = log(arg)
    return 1.613 / (l * l)


def friction_factor_colebrook(
    reynolds_number: Float64,
    rel_roughness: Float64,
    tol: Float64 = 1e-12,
    max_iter: Int = 100,
) -> Float64:
    """Darcy friction factor from the implicit Colebrook–White equation.

    Fixed-point iteration seeded from the Swamee–Jain estimate.
    """
    if reynolds_number < LAMINAR_RE_LIMIT:
        return 64.0 / reynolds_number
    var f = friction_factor(reynolds_number, rel_roughness)
    for _ in range(max_iter):
        var rhs = -2.0 * log10(
            rel_roughness / 3.71 + 2.51 / (reynolds_number * sqrt(f))
        )
        var f_new = 1.0 / (rhs * rhs)
        var diff = f_new - f if f_new >= f else f - f_new
        if diff < tol:
            return f_new
        f = f_new
    return f
