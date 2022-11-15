def reynolds(v, d_h, vi=15e-6):
    """Equation for calculating Reynolds number, where
    v - speed,
    d_h - hydraulic diameter,
    vi - viscosity"""
    return v * d_h / vi


def friction_coefficient(Re, k, d_h):
    """Friction coefficient [-].
    Re - Reynolds number [-],
    k - absolute roughness [m],
    d_h - hydraulic diameter [m]"""
    import colebrook

    return colebrook.eptFriction(Re, k / d_h)


def pressure_drop_per_meter(f, d_h, v, rho=1.2):
    """Pressure drop per meter [Pa/m]
    f - friction coefficient,
    d_h - hydraulic diameter,
    v - speed,
    rho - density"""
    return f / d_h * (rho * v**2) / 2


def roughness_correction_factor(absolute_roughness, diameter, velocity):
    # XYZ HVAC Systems Duct Design, SMACNA
    print("SMACNA function placeholder")
    return 1


def linear_pressure_drop(R, L, Beta):
    """Linear pressure drop. [Pa]
    R - pressure drop per meter,
    L - duct legth,
    Beta - roughness correction factor"""
    return R * L * Beta


def local_pressure_drop(dzeta, v, rho=1.2):
    """Local pressure drop [Pa].
    dzeta - drag coefficient
    rho - density
    v - velocity
    """
    return dzeta * rho * v**2 / 2
