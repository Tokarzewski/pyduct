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


# ---- VentiTrace: sketch -> duct network ------------------------------------
# Skeleton: reads the current selection's edges, flattens them into (x, y)
# polylines in metres, traces them into a venti network (venti_topology_trace)
# and reports the component count + critical-path ΔP. Everything is defensive:
# no FreeCAD geometry knowledge beyond Shape/Edges + discretize().


def _shape_edges(obj):
    """Return the list of edges of an object (Shape.Edges or Sketch.Geometry)."""
    shape = getattr(obj, "Shape", None)
    if shape is not None:
        edges = getattr(shape, "Edges", None)
        if edges:
            return list(edges)
    return list(getattr(obj, "Geometry", None) or [])


def _edge_to_polyline(edge, samples=16):
    """Convert one CAD edge to a polyline of (x, y) points [m].

    Part edges are discretized (lines -> 2 points, curves/BSplines -> samples);
    sketch geometry objects fall back to StartPoint/EndPoint.
    """
    pts = []
    try:
        if hasattr(edge, "discretize"):
            pts = [(p.x, p.y) for p in edge.discretize(samples)]
        elif hasattr(edge, "StartPoint") and hasattr(edge, "EndPoint"):
            s, e = edge.StartPoint, edge.EndPoint
            pts = [(s.x, s.y), (e.x, e.y)]
        else:
            for vert in (getattr(edge, "Vertexes", None) or []):
                p = vert.Point
                pts.append((p.x, p.y))
    except Exception:
        return []
    return pts


def _selected_polylines():
    """Collect (x, y) polylines [m] from the current FreeCAD selection."""
    import FreeCADGui  # noqa: F401
    polylines = []
    for obj in FreeCADGui.Selection.getSelection():
        for edge in _shape_edges(obj):
            poly = _edge_to_polyline(edge)
            if len(poly) >= 2:
                polylines.append(poly)
    return polylines


class VentiTrace:
    """Trace the selected sketch/edges into a venti duct network and solve it.

    Skeleton: converts each selected edge (Line/BSpline approx) into a
    polyline of (x, y) points, calls core.trace_network, and prints the
    component count and critical-path ΔP to the console.
    """

    def GetResources(self):
        return {"Pixmap": "", "MenuText": "Trace sketch to duct network",
                "ToolTip": "Trace selected sketch edges into a duct network and solve the critical path"}

    def Activated(self):
        try:
            segments = _selected_polylines()
            if not segments:
                _log("venti: trace: select a sketch or object with edges first")
                return
            with get_core() as core:
                res = core.trace_network(segments)
                n = res.component_count()
                dp = res.solve()
                res.free()
            _log(f"venti: traced {len(segments)} polyline(s) -> {n} components, critical-path ΔP = {dp:.2f} Pa"
                 )
        except Exception as exc:
            _log(f"venti: trace failed: {exc}")

    def IsActive(self):
        return True


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
        _log(f"venti: sized duct D = {d * 1000:.0f} mm at v = {v:.2f} m/s (Q = {flowrate} m3/s)"
             )

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
        _log(f"venti: v = {v:.2f} m/s  ΔP = {total:.2f} Pa (critical path)")

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
        _log(f"venti: required insulation thickness = {(t or 0) * 1000:.0f} mm (λ = {lam} W/mK)"
             )

    def IsActive(self):
        return True


def install_commands():
    """Register commands with the FreeCAD GUI (called from InitGui)."""
    import FreeCADGui  # noqa: F401
    FreeCADGui.addCommand("VentiSize", VentiSize())
    FreeCADGui.addCommand("VentiSolve", VentiSolve())
    FreeCADGui.addCommand("VentiInsulation", VentiInsulation())
    FreeCADGui.addCommand("VentiTrace", VentiTrace())