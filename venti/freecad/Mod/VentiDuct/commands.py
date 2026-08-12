# VentiDuct FreeCAD commands.
#
# FreeCAD imports are done lazily inside each command so this module can be
# imported/compiled without FreeCAD (e.g. for testing).

from .venti_core import get_core


def _fc():
    import FreeCAD  # noqa: F401
    return FreeCAD


def _log(msg):
    FreeCAD = _fc()
    try:
        FreeCAD.Console.PrintMessage(str(msg) + "\n")
    except Exception:
        print(str(msg))


def _param(name, default):
    FreeCAD = _fc()
    try:
        p = FreeCAD.ParamGet("User parameter:BaseApp/Preferences/Mod/VentiDuct")
        return p.GetString(name, default)
    except Exception:
        return default


class VentiSize:
    """Size a round duct by the velocity method and report D + velocity."""

    def GetResources(self):
        return {"Pixmap": "", "MenuText": "Size round duct (velocity)",
                "ToolTip": "Size a round duct for a flowrate at a target velocity"}

    def Activated(self):
        flowrate = float(_param("flowrate_m3s", "0.1"))
        target = float(_param("target_velocity", "4.0"))
        with get_core() as core:
            d, v = core.velocity_method_round(flowrate, target)
        _log("venti: sized duct D = {:.0f} mm at v = {:.2f} m/s (Q = {} m3/s)"
             .format(d * 1000, v, flowrate))

    def IsActive(self):
        return True


class VentiSolve:
    """Solve a small Source->Duct->Fitting->Terminal chain, report critical-path ΔP."""

    def GetResources(self):
        return {"Pixmap": "", "MenuText": "Solve example network",
                "ToolTip": "Solve a small duct network and report critical-path static pressure"}

    def Activated(self):
        with get_core() as core:
            q, d, length, rough = 0.1, 0.2, 20.0, 0.0001
            rho, mu = 1.204, 1.825e-5
            area = 3.14159 * (d / 2) ** 2
            v = q / area
            nu = mu / rho
            re = v * d / nu
            f = core.friction_factor(re, rough / d)
            dp_duct = f * (length / d) * rho * v * v * 0.5
            dp_fit = 0.5 * rho * v * v * 0.5
            dp_term = 1.0 * rho * v * v * 0.5
            total = dp_duct + dp_fit + dp_term
        _log("venti: v = {:.2f} m/s  ΔP = {:.2f} Pa (critical path)".format(v, total))

    def IsActive(self):
        return True


class VentiInsulation:
    """Compute insulation thickness to prevent condensation on a cold duct."""

    def GetResources(self):
        return {"Pixmap": "", "MenuText": "Insulation thickness (condensation)",
                "ToolTip": "Required duct insulation thickness to prevent surface condensation"}

    def Activated(self):
        air_c = float(_param("air_c", "8.0"))
        dew_c = float(_param("dew_c", "15.8"))
        amb_c = float(_param("amb_c", "24.0"))
        lam = float(_param("conductivity", "0.035"))
        d_m = float(_param("duct_diameter_m", "0.2"))
        with get_core() as core:
            t = core.insulation_condensation(air_c, dew_c, amb_c, lam, d_m)
        _log("venti: required insulation thickness = {:.0f} mm (λ = {} W/mK)"
             .format((t or 0) * 1000, lam))

    def IsActive(self):
        return True


def install_commands():
    """Register commands with the FreeCAD GUI (called from InitGui)."""
    import FreeCADGui  # noqa: F401
    FreeCADGui.addCommand("VentiSize", VentiSize())
    FreeCADGui.addCommand("VentiSolve", VentiSolve())
    FreeCADGui.addCommand("VentiInsulation", VentiInsulation())