"""Pressure-drop primitives (Mojo port of `wenta.physics.losses`)."""


def straight_pressure_drop(
    friction_factor: Float64,
    length: Float64,
    hydraulic_diameter: Float64,
    velocity: Float64,
    density: Float64,
) -> Float64:
    """Darcy–Weisbach straight-duct pressure drop [Pa].

        dp = f * (L / D_h) * (rho * v^2 / 2)
    """
    return friction_factor * (length / hydraulic_diameter) * (
        density * velocity * velocity * 0.5
    )


def local_pressure_drop(zeta: Float64, velocity: Float64, density: Float64) -> Float64:
    """Local-fitting pressure drop dp = zeta * (rho * v^2 / 2) [Pa]."""
    return zeta * density * velocity * velocity * 0.5
