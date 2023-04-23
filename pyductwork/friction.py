from colebrook import eptFriction


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
    return eptFriction(Re, k / d_h)


def pressure_drop_per_meter(f, d_h, v, rho=1.2):
    """Pressure drop per meter [Pa/m]
    f - friction coefficient,
    d_h - hydraulic diameter,
    v - speed,
    rho - density"""
    return f / d_h * (rho * v**2) / 2


def linear_pressure_drop(R, L, Beta=1):
    """Linear pressure drop. [Pa]
    R - pressure drop per meter,
    L - duct legth,
    Beta - roughness correction factor"""
    return R * L * Beta


def local_pressure_drop(dzeta, v, rho=1.2):
    """Local pressure drop [Pa]:
    dzeta - drag coefficient,
    rho - density,
    v - velocity.
    """
    return dzeta * rho * v**2 / 2


def flex_stretch_correction_factor(diameter, stretch_percentage):
    # Flex is fully stretched, when stretch_percentage = 100
    # Developed based on chart from ASHRAE Fundamentals
    # R**2 = 0.995
    from math import exp

    return 0.557 * (100 - stretch_percentage) * exp(-4.93 * diameter) + 1


def flex_pressure_drop_per_meter(diameter, V):
    # this is manufacturer and flex designtype specific
    # a flex library is needed
    return 1

if __name__ == "__main__":
    print(friction_coefficient(4000, 0.00001, 0.05))