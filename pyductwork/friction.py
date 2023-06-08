from math import log, log10, sqrt, exp


def reynolds(v, d_h, vi=15e-6):
    """Equation for calculating Reynolds number, where
    v - speed,
    d_h - hydraulic diameter,
    vi - viscosity"""
    return v * d_h / vi


def relative_roughness(e, D):
    """Relative Roughness, 
    e - absolute roughness [m],
    D - hydraulic diameter [m].
    """
    return e / D


def friction_coefficient(Re, E):
    """Friction coefficient [-].
    Re - Reynolds number [-],
    E - Relative roughness.
    Model: Swamee, Jain
    Year: 1976
    Paper: https://cedb.asce.org/CEDBsearch/record.jsp?dockey=0006693
    Suitable Range:
        5000 < Reyonolds < 10^8
        0.00001 < E < 0.5
    Copied from https://pypi.org/project/colebrook/
    """
    if Re < 2300:
        return 64 / Re
    return 1.613 * (log(0.234 * E ** 1.1007 - 60.525 / Re**1.1105 + 56.291 / Re**1.0712))**-2


def friction_coefficient2(Re, E):
    """Friction coefficient [-].
    Re - Reynolds number [-],
    E - Relative roughness.
    """
    from scipy.optimize import root
    if Re < 2300:
        return 64 / Re
    def f(x):
        return (-2*log10((2.51/(Re*sqrt(x))) + (E/3.71))) - 1.0/sqrt(x)
    return root(f, 0.04).x


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
    return 0.557 * (100 - stretch_percentage) * exp(-4.93 * diameter) + 1


def flex_pressure_drop_per_meter(diameter, V):
    # this is manufacturer and flex designtype specific
    # a flex library is needed
    return 1

if __name__ == "__main__":
    Re = 4000
    E = 0.0002
    x = float(friction_coefficient(Re, E))
    print(f"{x:f}")
    x = float(friction_coefficient2(Re, E))
    print(f"{x:f}")