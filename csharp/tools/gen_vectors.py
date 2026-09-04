#!/usr/bin/env python3
"""Generate parity vectors for the C# Wenta.Core port.

On Windows the wenta oracle cannot run (mojo wheels are Linux/macOS-only),
so this generator transcribes the canonical formula source —
`wentamojo` (which is parity-tested against python/wenta) — in pure
Python, anchors spot values to `python/tests` expectations, and uses
scipy for the ElbowRound spline (Python `wenta.components.elbow` itself
uses scipy, so it runs natively here).

Run:  python gen_vectors.py <outdir>
"""
from __future__ import annotations

import math
import os
import sys

from scipy.interpolate import RectBivariateSpline

OUT = sys.argv[1] if len(sys.argv) > 1 else os.path.join(
    os.path.dirname(__file__), "..", "Wenta.Core.Tests", "vectors")


def w(name):
    path = os.path.join(OUT, name)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    return open(path, "w", newline="", encoding="utf-8")


def fmt(x):
    return repr(float(x))


# ----------------------------------------------------------------------------
# wentamojo formula transcriptions
# ----------------------------------------------------------------------------

CFM_TO_M3S = 0.0004719474432
INWC_TO_PA = 249.0889
FT_TO_M = 0.3048
IN_TO_M = 0.0254
FPM_TO_MS = 0.00508


def cfm_to_m3s(v): return v * CFM_TO_M3S
def m3s_to_cfm(v): return v / CFM_TO_M3S
def inwc_to_pa(v): return v * INWC_TO_PA
def pa_to_inwc(v): return v / INWC_TO_PA
def ft_to_m(v): return v * FT_TO_M
def m_to_ft(v): return v / FT_TO_M
def in_to_m(v): return v * IN_TO_M
def m_to_in(v): return v / IN_TO_M
def fpm_to_ms(v): return v * FPM_TO_MS
def ms_to_fpm(v): return v / FPM_TO_MS
def f_to_c(v): return (v - 32.0) * 5.0 / 9.0
def c_to_f(v): return v * 9.0 / 5.0 + 32.0


def ach(flow, volume):
    if volume <= 0.0:
        raise ValueError
    if flow < 0.0:
        raise ValueError
    return flow * 3600.0 / volume


def air_at_altitude(altitude_m, temperature_c=20.0):
    if altitude_m < 0.0:
        raise ValueError
    h = min(altitude_m, 11000.0)
    pressure = 101325.0 * (1.0 - 2.25577e-5 * h) ** 5.2561
    t_k = temperature_c + 273.15
    density = pressure / (287.058 * t_k)
    mu = 1.458e-6 * t_k ** 1.5 / (t_k + 110.4)
    return density, mu


def friction_factor(re, eps):
    if re < 2300.0:
        return 64.0 / re
    arg = (0.234 * eps ** 1.1007
           - 60.525 / re ** 1.1105
           + 56.291 / re ** 1.0712)
    lg = math.log(arg)
    return 1.613 / (lg * lg)


def friction_factor_colebrook(re, eps, tol=1e-12, max_iter=100):
    if re < 2300.0:
        return 64.0 / re
    f = friction_factor(re, eps)
    for _ in range(max_iter):
        rhs = -2.0 * math.log10(eps / 3.71 + 2.51 / (re * math.sqrt(f)))
        f_new = 1.0 / (rhs * rhs)
        diff = f_new - f if f_new >= f else f - f_new
        if diff < tol:
            return f_new
        f = f_new
    return f


def flex_factor(d, stretch):
    return 0.557 * (100.0 - stretch) * math.exp(-4.93 * d) + 1.0


def eq_round_diameter(width, height):
    if width <= 0.0 or height <= 0.0:
        raise ValueError
    return 1.30 * (width * height) ** 0.625 / (width + height) ** 0.25


ROUND_SIZES = [63, 80, 100, 125, 150, 160, 200, 250, 300, 315, 355,
               400, 450, 500, 560, 630, 710, 800, 900, 1000, 1120, 1250]


def nearest_round_size(d_mm, round_up=True):
    n = len(ROUND_SIZES)
    first, last = ROUND_SIZES[0], ROUND_SIZES[-1]
    if d_mm <= first:
        return first
    if d_mm >= last:
        return last
    idx = next(i for i in range(n) if ROUND_SIZES[i] >= d_mm)
    if round_up:
        return ROUND_SIZES[idx]
    prev = ROUND_SIZES[idx - 1]
    return prev if (d_mm - prev) < (ROUND_SIZES[idx] - d_mm) else ROUND_SIZES[idx]


def reducer_round(di, do, angle=45.0):
    if do > di:
        raise ValueError
    if do <= 0.0:
        raise ValueError
    ar = (do / di) ** 2
    z = 0.04 + 0.37 * (1.0 - ar)
    af = 0.8 + 0.004 * (45.0 - angle) if angle < 45.0 else 1.0
    return z * af


def expander_round(di, do, angle=45.0):
    if di > do:
        raise ValueError
    if di <= 0.0:
        raise ValueError
    ar = (di / do) ** 2
    zs = (1.0 - ar) ** 2
    df = 0.5 if angle <= 10.0 else 0.6 if angle <= 20.0 else 0.8 if angle <= 45.0 else 1.0
    return df * zs


def tee_branch(dm, db, fm, fb):
    if fm < 0.0 or fb < 0.0:
        raise ValueError
    total = fm + fb
    if total <= 0.0:
        raise ValueError
    split = fb / total
    area = (db / dm) ** 2 if dm > 0.0 else 0.0
    return 0.08 * split + 0.05 * area, 0.3 + 0.5 * (1.0 - area) + 0.4 * split


def tee_combine(dm, db, fm, fb):
    total = fm + fb
    if total <= 0.0:
        raise ValueError
    split = fb / total
    area = (db / dm) ** 2 if dm > 0.0 else 0.0
    return 0.1 + 0.15 * split + 0.08 * area, 0.4 + 0.6 * (1.0 - area) + 0.3 * split


def damper(open_pct):
    if open_pct < 0.0 or open_pct > 100.0:
        raise ValueError
    if open_pct >= 95.0:
        return 0.1
    cf = 1.0 - open_pct / 100.0
    return 0.1 + cf * cf * 10.0


def diffuser(throw):
    if throw <= 0.0:
        raise ValueError
    return 0.4 / throw


def grille(blockage):
    if blockage < 0.0 or blockage > 1.0:
        raise ValueError
    return 0.25 * (1.0 + blockage)


def rect_elbow(width, height, r, angle):
    smallest = min(width, height, r)
    if smallest <= 0.0:
        raise ValueError
    if angle <= 0.0 or angle > 180.0:
        raise ValueError
    rw = r / width
    floor = rw if rw > 0.1 else 0.1
    z90 = 0.21 / floor ** 0.5
    if z90 > 1.5:
        z90 = 1.5
    return z90 * (height / width) ** 0.25 * (angle / 90.0)


def mitered(angle, vaned):
    if angle <= 0.0 or angle > 180.0:
        raise ValueError
    a = angle / 90.0
    z = 0.55 * a + 0.65 * a * a
    return z * (0.4 if vaned else 1.0)


_RD_GRID = (0.50, 0.75, 1.00, 1.50, 2.00, 2.50)
_ANGLE_GRID = (20, 30, 45, 60, 75, 90, 110, 130, 150, 180)
_ZETA_TABLE = (
    (0.22, 0.32, 0.43, 0.55, 0.64, 0.71, 0.80, 0.85, 0.91, 0.99),
    (0.10, 0.15, 0.20, 0.26, 0.30, 0.33, 0.37, 0.40, 0.42, 0.46),
    (0.07, 0.10, 0.13, 0.17, 0.20, 0.22, 0.25, 0.26, 0.28, 0.31),
    (0.05, 0.07, 0.09, 0.12, 0.14, 0.15, 0.17, 0.18, 0.19, 0.21),
    (0.04, 0.06, 0.08, 0.10, 0.12, 0.13, 0.15, 0.16, 0.17, 0.18),
    (0.04, 0.05, 0.07, 0.09, 0.11, 0.12, 0.14, 0.14, 0.15, 0.17),
)
_SPLINE = RectBivariateSpline(_RD_GRID, _ANGLE_GRID, _ZETA_TABLE)


def elbow_zeta(bend_radius, diameter, angle):
    rd = bend_radius / diameter
    if rd < _RD_GRID[0] or rd > _RD_GRID[-1]:
        raise ValueError
    if angle < _ANGLE_GRID[0] or angle > _ANGLE_GRID[-1]:
        raise ValueError
    return float(_SPLINE(rd, angle).item())


# ----------------------------------------------------------------------------
# sizing + standard sizes (pure-Python mirror of wenta.sizing logic)
# ----------------------------------------------------------------------------

RECT_SIZES = [(100, 200), (150, 200), (200, 200), (100, 250), (150, 250),
              (200, 250), (250, 250), (100, 300), (150, 300), (200, 300),
              (250, 300), (300, 300), (100, 400), (150, 400), (200, 400),
              (250, 400), (300, 400), (400, 400), (150, 500), (200, 500),
              (250, 500), (300, 500), (400, 500), (500, 500), (150, 600),
              (200, 600), (250, 600), (300, 600), (400, 600), (500, 600),
              (600, 600), (200, 800), (250, 800), (300, 800), (400, 800),
              (500, 800), (600, 800), (800, 800), (250, 1000), (300, 1000),
              (400, 1000), (500, 1000), (600, 1000), (800, 1000), (1000, 1000),
              (300, 1200), (400, 1200), (500, 1200), (600, 1200), (800, 1200),
              (1000, 1200), (1200, 1200), (400, 1400), (500, 1400), (600, 1400),
              (800, 1400), (1000, 1400), (1200, 1400), (400, 1600), (500, 1600),
              (600, 1600), (800, 1600), (1000, 1600), (1200, 1600), (500, 1800),
              (600, 1800), (800, 1800), (1000, 1800), (1200, 1800), (500, 2000),
              (600, 2000), (800, 2000), (1000, 2000), (1200, 2000)]

NOISE_LIMITS = {"studio": 2.5, "bedroom": 3.0, "office": 4.0,
                "classroom": 4.5, "retail": 5.0, "industrial": 7.5}
RHO_AIR, MU_AIR = 1.204, 1.825e-5
NU_AIR = MU_AIR / RHO_AIR


def sec_area(shape, i):
    if shape == "round":
        d = ROUND_SIZES[i] / 1000.0
        return math.pi * (d / 2.0) ** 2
    w, h = RECT_SIZES[i]
    return (w / 1000.0) * (h / 1000.0)


def velocity_method(flow, shape, tv):
    n = len(ROUND_SIZES if shape == "round" else RECT_SIZES)
    for i in range(n):
        a = sec_area(shape, i)
        v = flow / a
        if v <= tv:
            return i, v
    return n - 1, flow / sec_area(shape, n - 1)


def equal_friction_method(flow, target, shape, eps=0.0001):
    n = len(ROUND_SIZES if shape == "round" else RECT_SIZES)
    best = None
    for i in range(n):
        a = sec_area(shape, i)
        if shape == "round":
            d_h = ROUND_SIZES[i] / 1000.0
        else:
            w, h = RECT_SIZES[i]
            d_h = 2 * (w / 1000.0) * (h / 1000.0) / ((w + h) / 1000.0)
        v = flow / a
        f = friction_factor(friction_factor_re(v, d_h), eps / d_h)
        dpm = f / d_h * (RHO_AIR * v * v) / 2.0
        if dpm <= target:
            return i, v, dpm
        best = (i, v, dpm)
    return best


def friction_factor_re(v, d_h):
    return v * d_h / NU_AIR


def aspect_method(flow, tv, ar):
    cands = []
    for i, (w, h) in enumerate(RECT_SIZES):
        if max(w, h) / min(w, h) >= ar:
            cands.append((sec_area("rect", i), i))
    cands.sort()
    for a, i in cands:
        v = flow / a
        if v <= tv:
            return i, v
    a, i = cands[-1]
    return i, flow / a


# ----------------------------------------------------------------------------
# pure-Python solver mirror (network semantics from wenta.network.solver)
# ----------------------------------------------------------------------------

class MirrorNet:
    def __init__(self):
        self.comp = {}
        self.succ = {}
        self.pred = {}
        self.flow = {}
        self.dp = {}
        self.port_node = {}

    def node(self, n):
        self.succ.setdefault(n, [])
        self.pred.setdefault(n, [])
        self.flow.setdefault(n, 0.0)
        self.dp.setdefault(n, 0.0)

    def edge(self, a, b):
        self.node(a)
        self.node(b)
        self.succ[a].append(b)
        self.pred[b].append(a)

    def topo(self):
        indeg = {n: len(self.pred[n]) for n in self.succ}
        order, ready = [], [n for n in self.succ if indeg[n] == 0]
        while ready:
            n = ready.pop(0)
            order.append(n)
            for s in self.succ[n]:
                indeg[s] -= 1
                if indeg[s] == 0:
                    ready.append(s)
        assert len(order) == len(self.succ), "cycle"
        return order


def mirror_readme_net():
    """README example: AHU -> 20 m rigid round D200 duct -> terminal 0.1 m3/s."""
    net = MirrorNet()
    # nodes: ahu (comp), duct:inlet/outlet ports, duct comp, term
    # components: ahu(out), duct in->out, term(in flow 0.1)
    for n in ("ahu", "ahu:outlet", "duct", "duct:inlet", "duct:outlet",
              "term", "term:inlet"):
        net.node(n)
    # internal edges
    net.edge("ahu", "ahu:outlet")          # source -> out port
    net.edge("duct:inlet", "duct")         # in port -> component
    net.edge("duct", "duct:outlet")        # component -> out port
    net.edge("term:inlet", "term")         # in port -> component
    # connections
    net.edge("ahu:outlet", "duct:inlet")
    net.edge("duct:outlet", "term:inlet")

    # component params
    d = 0.2
    area = math.pi * (d / 2.0) ** 2
    length, eps = 20.0, 0.0001
    rho, nu = RHO_AIR, NU_AIR

    def compute():
        # propagate
        for n in net.flow:
            net.flow[n] = 0.0
        net.flow["term:inlet"] = 0.1
        for n in reversed(net.topo()):
            f = net.flow[n]
            if f:
                for p in net.pred[n]:
                    net.flow[p] += f
        # compute dp
        for n in net.dp:
            net.dp[n] = 0.0
        v = net.flow["duct:inlet"] / area
        re = v * d / nu
        f = friction_factor(re, eps / d)
        dp = f * (length / d) * rho * v * v / 2.0
        net.dp["duct:inlet"] = dp
        net.dp["duct:outlet"] = 0.0
        net.dp["ahu"] = 0.0
        net.dp["ahu:outlet"] = 0.0
        net.dp["duct"] = 0.0
        net.dp["term:inlet"] = 0.0
        net.dp["term"] = 0.0
        # critical path DP
        dist = {}
        for n in net.topo():
            preds = net.pred[n]
            best_d = 0.0
            if preds:
                best_d = max(dist[p] for p in preds)
            dist[n] = best_d + net.dp[n]
        return max(dist.values()), net.flow["duct:inlet"], dp

    return compute


def mirror_tee_net():
    """ahu -> duct(20m D315) -> tee -> straight duct(5m D200)->term(0.06),
    branch flex(D125, 3m, 2 Pa/m, 100%) -> term(0.04)."""
    net = MirrorNet()
    for n in ("ahu", "ahu:outlet", "duct", "duct:inlet", "duct:outlet",
              "tee", "tee:combined", "tee:straight", "tee:branch",
              "d2", "d2:inlet", "d2:outlet",
              "flex", "flex:inlet", "flex:outlet",
              "t1", "t1:inlet", "t2", "t2:inlet"):
        net.node(n)
    net.edge("ahu", "ahu:outlet")
    net.edge("duct:inlet", "duct")
    net.edge("duct", "duct:outlet")
    net.edge("tee:combined", "tee")
    net.edge("tee", "tee:straight")
    net.edge("tee", "tee:branch")
    net.edge("d2:inlet", "d2")
    net.edge("d2", "d2:outlet")
    net.edge("flex:inlet", "flex")
    net.edge("flex", "flex:outlet")
    net.edge("t1:inlet", "t1")
    net.edge("t2:inlet", "t2")
    net.edge("ahu:outlet", "duct:inlet")
    net.edge("duct:outlet", "tee:combined")
    net.edge("tee:straight", "d2:inlet")
    net.edge("tee:branch", "flex:inlet")
    net.edge("d2:outlet", "t1:inlet")
    net.edge("flex:outlet", "t2:inlet")

    rho, nu = RHO_AIR, NU_AIR

    def compute():
        for n in net.flow:
            net.flow[n] = 0.0
        net.flow["t1:inlet"] = 0.06
        net.flow["t2:inlet"] = 0.04
        for n in reversed(net.topo()):
            f = net.flow[n]
            if f:
                for p in net.pred[n]:
                    net.flow[p] += f
        for n in net.dp:
            net.dp[n] = 0.0

        def dp_straight(node, d, length, eps):
            area = math.pi * (d / 2.0) ** 2
            v = net.flow[node] / area
            re = v * d / nu
            f = friction_factor(re, eps / d)
            return f * (length / d) * rho * v * v / 2.0

        net.dp["duct:inlet"] = dp_straight("duct:inlet", 0.315, 20.0, 0.0001)
        net.dp["d2:inlet"] = dp_straight("d2:inlet", 0.2, 5.0, 0.0001)
        # flex: 2 Pa/m * 3 m * beta(0.125, 100) = 6 Pa
        beta = flex_factor(0.125, 100.0)
        net.dp["flex:inlet"] = 2.0 * 3.0 * beta
        # tee: zeta straight 0.1, branch 0.4 (catalog-ish), combined area D315
        area_tee = math.pi * (0.315 / 2.0) ** 2
        v_s = net.flow["tee:straight"] / area_tee
        v_b = net.flow["tee:branch"] / area_tee
        net.dp["tee:straight"] = 0.1 * rho * v_s * v_s / 2.0
        net.dp["tee:branch"] = 0.4 * rho * v_b * v_b / 2.0
        # terminals: 0 (no cross-section)
        net.dp["t1:inlet"] = 0.0
        net.dp["t2:inlet"] = 0.0

        dist = {}
        for n in net.topo():
            preds = net.pred[n]
            best_d = max((dist[p] for p in preds), default=0.0)
            dist[n] = best_d + net.dp[n]
        return (max(dist.values()),
                net.flow["duct:inlet"],
                net.flow["tee:straight"],
                net.flow["tee:branch"])

    return compute


# ----------------------------------------------------------------------------
# write vector files
# ----------------------------------------------------------------------------

def main():
    # --- units ---
    with w("units.csv") as f:
        f.write("id;op;arg;expected;error\n")
        for i, (op, fn) in enumerate([
                ("cfm_to_m3s", cfm_to_m3s), ("m3s_to_cfm", m3s_to_cfm),
                ("inwc_to_pa", inwc_to_pa), ("pa_to_inwc", pa_to_inwc),
                ("ft_to_m", ft_to_m), ("m_to_ft", m_to_ft),
                ("in_to_m", in_to_m), ("m_to_in", m_to_in),
                ("fpm_to_ms", fpm_to_ms), ("ms_to_fpm", ms_to_fpm),
                ("f_to_c", f_to_c), ("c_to_f", c_to_f)]):
            for x in (0.0, 1.0, 2.5, 100.0, 1234.5678):
                f.write(f"{op}{i};{op};{fmt(x)};{fmt(fn(x))};\n")
        for x, v in ((0.05, 100.0), (0.5, 60.0)):
            f.write(f"ach1;ach;{fmt(x)},{fmt(v)};{fmt(ach(x, v))};\n")
        f.write("ach_bad1;ach;0.05,0.0;;error\n")
        f.write("ach_bad2;ach;-0.05,10.0;;error\n")

    # --- fluid ---
    with w("fluid.csv") as f:
        f.write("id;op;arg;density;viscosity;error\n")
        for alt in (0.0, 100.0, 500.0, 1500.0, 3000.0, 11000.0, 20000.0):
            for t in (10.0, 20.0, 40.0):
                d, mu = air_at_altitude(alt, t)
                f.write(f"alt{alt:.0f}t{t:.0f};air_at_altitude;"
                        f"{fmt(alt)},{fmt(t)};{fmt(d)};{fmt(mu)};\n")
        f.write("alt_bad;air_at_altitude;-100.0,20.0;;;error\n")

    # --- geometry ---
    with w("geometry.csv") as f:
        f.write("id;op;arg;a;b;error\n")
        for d in (0.05, 0.1, 0.2, 0.315, 1.25):
            f.write(f"round{d};round;{fmt(d)};"
                    f"{fmt(math.pi * (d / 2.0) ** 2)};{fmt(d)};\n")
        for x, y in ((0.1, 0.2), (0.3, 0.3), (0.4, 0.2), (1.0, 2.0)):
            f.write(f"rect{x}x{y};rect;{fmt(x)},{fmt(y)};{fmt(x * y)};"
                    f"{fmt(2 * x * y / (x + y))};\n")
        for x, y in ((0.1, 0.2), (0.3, 0.4), (0.5, 1.0)):
            f.write(f"eq{x}x{y};eq_round;{fmt(x)},{fmt(y)};"
                    f"{fmt(eq_round_diameter(x, y))};0;\n")
        f.write("round_bad;round;0.0;;;error\n")
        f.write("rect_bad;rect;0.1,0.0;;;error\n")

    # --- friction / losses / flex ---
    with w("friction.csv") as f:
        f.write("id;op;arg;expected;error\n")
        for v, d_h, nu in ((5.0, 0.2, 1.516e-5), (3.0, 0.315, NU_AIR),
                           (1.0, 0.1, 1.5e-5)):
            f.write(f"re{v}{d_h};reynolds;{fmt(v)},{fmt(d_h)},{fmt(nu)};"
                    f"{fmt(v * d_h / nu)};\n")
        f.write("rr1;rel_rough;0.0001,0.2;" + fmt(0.0005) + ";\n")
        # laminar + turbulent spot values (64/Re + Swamee-Jain)
        for re in (1000.0, 2000.0):
            f.write(f"ff_lam{re:.0f};friction_factor;{fmt(re)},0.01;{fmt(64.0 / re)};\n")
        for re in (5000.0, 25000.0, 100000.0, 1000000.0):
            for eps in (1e-5, 1e-4, 1e-3, 1e-2):
                f.write(f"ff{re:.0f}_{eps:.0e};friction_factor;{fmt(re)},{fmt(eps)};"
                        f"{fmt(friction_factor(re, eps))};\n")
                f.write(f"cb{re:.0f}_{eps:.0e};colebrook;{fmt(re)},{fmt(eps)};"
                        f"{fmt(friction_factor_colebrook(re, eps))};\n")

    with w("losses.csv") as f:
        f.write("id;op;arg;expected;error\n")
        f.write("sp1;straight;0.02,20.0,0.2,3.1831,1.204;"
                + fmt(0.02 * (20.0 / 0.2) * 1.204 * 3.1831 * 3.1831 / 2.0) + ";\n")
        f.write("sp2;straight;0.03,5.0,0.315,2.0,1.204;"
                + fmt(0.03 * (5.0 / 0.315) * 1.204 * 2.0 * 2.0 / 2.0) + ";\n")
        f.write("lp1;local;0.5,4.0,1.204;" + fmt(0.5 * 1.204 * 16.0 / 2.0) + ";\n")
        f.write("lp2;local;1.2,3.0,1.204;" + fmt(1.2 * 1.204 * 9.0 / 2.0) + ";\n")

    with w("flex.csv") as f:
        f.write("id;op;arg;expected;error\n")
        for d in (0.125, 0.2, 0.315):
            for s in (100.0, 80.0, 50.0):
                f.write(f"flex{d}_{s:.0f};flex;{fmt(d)},{fmt(s)};"
                        f"{fmt(flex_factor(d, s))};\n")

    # --- standard sizes ---
    with w("sizes.csv") as f:
        f.write("id;op;arg;expected;error\n")
        for d in (0.0, 50.0, 63.0, 70.0, 100.0, 155.0, 355.0, 900.0, 1250.0, 2000.0):
            f.write(f"up{d:.0f};nearest;{fmt(d)},True;{nearest_round_size(d, True)};\n")
            f.write(f"cl{d:.0f};nearest;{fmt(d)},False;{nearest_round_size(d, False)};\n")

    # --- fittings ---
    with w("fittings.csv") as f:
        f.write("id;op;arg;e1;e2;error\n")
        for di, do in ((0.315, 0.2), (0.2, 0.125), (0.25, 0.25)):
            f.write(f"red{di}_{do};reducer;{fmt(di)},{fmt(do)},45.0;"
                    f"{fmt(reducer_round(di, do))};;\n")
        f.write("red30;reducer;0.315,0.2,30.0;"
                + fmt(reducer_round(0.315, 0.2, 30.0)) + ";;\n")
        f.write("red60;reducer;0.315,0.2,60.0;"
                + fmt(reducer_round(0.315, 0.2, 60.0)) + ";;\n")
        f.write("red_bad;reducer;0.2,0.315,45.0;;;error\n")
        for di, do in ((0.2, 0.315), (0.2, 0.2)):
            f.write(f"exp{di}_{do};expander;{fmt(di)},{fmt(do)},45.0;"
                    f"{fmt(expander_round(di, do))};;\n")
        f.write("exp10;expander;0.2,0.315,10.0;" + fmt(expander_round(0.2, 0.315, 10.0)) + ";;\n")
        f.write("exp20;expander;0.2,0.315,20.0;" + fmt(expander_round(0.2, 0.315, 20.0)) + ";;\n")
        f.write("exp60;expander;0.2,0.315,60.0;" + fmt(expander_round(0.2, 0.315, 60.0)) + ";;\n")
        f.write("exp_bad;expander;0.315,0.2,45.0;;;error\n")
        for fm, fb in ((0.1, 0.05), (0.2, 0.1), (0.1, 0.0)):
            z1, z2 = tee_branch(0.315, 0.2, fm, fb)
            f.write(f"tb{fm}_{fb};tee_branch;0.315,0.2,{fmt(fm)},{fmt(fb)};"
                    f"{fmt(z1)};{fmt(z2)};\n")
            z1, z2 = tee_combine(0.315, 0.2, fm, fb)
            f.write(f"tc{fm}_{fb};tee_combine;0.315,0.2,{fmt(fm)},{fmt(fb)};"
                    f"{fmt(z1)};{fmt(z2)};\n")
        for o in (100.0, 96.0, 50.0, 25.0, 0.0):
            f.write(f"dmp{o:.0f};damper;{fmt(o)};" + fmt(damper(o)) + ";;\n")
        f.write("dmp_bad;damper;150.0;;;;error\n")
        for t in (0.5, 1.0, 2.0):
            f.write(f"dif{t};diffuser;{fmt(t)};" + fmt(diffuser(t)) + ";;\n")
        for b in (0.0, 0.15, 0.5, 1.0):
            f.write(f"grl{b};grille;{fmt(b)};" + fmt(grille(b)) + ";;\n")
        f.write("grl_bad;grille;1.5;;;;error\n")
        for wd, ht, r in ((0.4, 0.2, 0.4), (0.3, 0.3, 0.15), (0.6, 0.3, 0.1)):
            f.write(f"relbow{wd}x{ht};rect_elbow;{fmt(wd)},{fmt(ht)},{fmt(r)},90.0;"
                    f"{fmt(rect_elbow(wd, ht, r, 90.0))};;\n")
        f.write("relbow45;rect_elbow;0.4,0.2,0.4,45.0;"
                + fmt(rect_elbow(0.4, 0.2, 0.4, 45.0)) + ";;\n")
        for ang in (30.0, 45.0, 90.0, 120.0):
            f.write(f"mit{ang:.0f};mitered;{fmt(ang)},False;"
                    f"{fmt(mitered(ang, False))};;\n")
            f.write(f"mit{ang:.0f}v;mitered;{fmt(ang)},True;"
                    f"{fmt(mitered(ang, True))};;\n")
        f.write("mit_bad;mitered;0.0,False;;;;error\n")

    # --- elbow (scipy ground truth) ---
    with w("elbow.csv") as f:
        f.write("id;op;R;D;angle;expected;error\n")
        d = 0.25
        # exact grid points
        for rd in _RD_GRID:
            for ang in _ANGLE_GRID:
                f.write(f"elb_grid_{rd}_{ang};elbow;{fmt(rd * d)},{fmt(d)},{fmt(ang)};"
                        f"{fmt(elbow_zeta(rd * d, d, ang))};\n")
        # intermediate points (1e-4 tolerance territory)
        for rd in (0.55, 0.9, 1.2, 1.8, 2.3):
            for ang in (25.0, 37.5, 52.0, 100.0, 137.0, 165.0):
                f.write(f"elb_mid_{rd}_{ang};elbow;{fmt(rd * d)},{fmt(d)},{fmt(ang)};"
                        f"{fmt(elbow_zeta(rd * d, d, ang))};\n")
        f.write(f"elb_bad_rd;elbow;{fmt(0.05)},{fmt(d)},90.0;;error\n")
        f.write(f"elb_bad_ang;elbow;{fmt(0.25)},{fmt(d)},10.0;;error\n")

    # --- sizing ---
    with w("sizing.csv") as f:
        f.write("id;op;arg;idx;velocity;dpm;error\n")
        for flow in (0.02, 0.05, 0.1, 0.3, 0.5, 1.2):
            for tv in (2.0, 4.0, 6.0):
                i, v = velocity_method(flow, "round", tv)
                f.write(f"vm_r_{flow}_{tv};velocity_round;{fmt(flow)},{fmt(tv)};"
                        f"{i};{fmt(v)};0;\n")
                i, v = velocity_method(flow, "rect", tv)
                f.write(f"vm_q_{flow}_{tv};velocity_rect;{fmt(flow)},{fmt(tv)};"
                        f"{i};{fmt(v)};0;\n")
            i, v, dpm = equal_friction_method(flow, 1.0, "round")
            f.write(f"ef_r_{flow};equal_friction;{fmt(flow)},1.0;"
                    f"{i};{fmt(v)};{fmt(dpm)};\n")
            i, v, dpm = equal_friction_method(flow, 0.8, "rect")
            f.write(f"ef_q_{flow};equal_friction_rect;{fmt(flow)},0.8;"
                    f"{i};{fmt(v)};{fmt(dpm)};\n")
        for space in NOISE_LIMITS:
            i, v = velocity_method(0.1, "round", NOISE_LIMITS[space])
            f.write(f"nl_{space};noise_limit;0.1,{space};{i};{fmt(v)};0;\n")
        for ar in (1.5, 2.0, 3.0, 4.0):
            i, v = aspect_method(0.1, 4.0, ar)
            f.write(f"as_{ar};aspect;0.1,4.0,{fmt(ar)};{i};{fmt(v)};0;\n")
        f.write("vm_bad;velocity_round;0.0,4.0;;;;error\n")
        f.write("nl_bad;noise_limit;0.1,dungeon;;;;error\n")

    # --- solver (two networks) ---
    with w("solver.csv") as f:
        f.write("id;op;expected_dp;expected_flow;aux1;aux2;error\n")
        dp, flow, dpduct = mirror_readme_net()()
        f.write(f"net_readme;net_readme;;{fmt(dp)};{fmt(flow)};{fmt(dpduct)};\n")
        dp, flow, qs, qb = mirror_tee_net()()
        f.write(f"net_tee;net_tee;;{fmt(dp)};{fmt(flow)};{fmt(qs)};{fmt(qb)}\n")

    print("vectors written to", OUT)


if __name__ == "__main__":
    main()
