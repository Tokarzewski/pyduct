"""Native-Mojo batch ``compute_pressure_drops`` kernel.

Walks an N-component flat-array view of the network and computes every
port's velocity and pressure drop in one boundary crossing.

Component type tags (Int):
    0 Source         — no math; v=0, dp=0 on the single out-port
    1 Terminal       — if has area: dp = ζ·ρ·v²/2 on the in-port
    2 RigidDuct      — Darcy v + dp on the in-port
    3 FlexDuct       — pdpm·L·β(stretch) on the in-port
    4 TwoPortFitting — dp = ζ·ρ·v²/2 on the out-port
    5 Tee            — dp = ζ·ρ·v²/2 on each of straight/branch out-ports

Per-component params layout (6 Float64s, zero-padded):
    Source         [_, _, _, _, _, _]
    Terminal       [area_or_0, zeta, _, _, _, _]
    RigidDuct      [area, d_h, length, abs_rough, _, _]
    FlexDuct       [area, diameter, length, pdpm, stretch%, _]
    TwoPortFitting [area, zeta, _, _, _, _]
    Tee            [area, zeta_straight, zeta_branch, _, _, _]

Per-component port indices (3 Ints, -1 if unused):
    Source         [out, -1, -1]
    Terminal       [in, -1, -1]
    RigidDuct      [in, out, -1]
    FlexDuct       [in, out, -1]
    TwoPortFitting [in, out, -1]
    Tee            [combined_in, straight_out, branch_out]

The caller pre-allocates `flows` (already populated by propagate_flowrates),
`velocities` (zeroed), `dps` (zeroed). The kernel mutates the latter two
in place — but they're Mojo-native Lists; the Python wrapper reads them
back as PyList for scattering.
"""

from std.math import exp

from ..physics.friction import friction_factor, relative_roughness, reynolds


comptime TAG_SOURCE = 0
comptime TAG_TERMINAL = 1
comptime TAG_RIGID = 2
comptime TAG_FLEX = 3
comptime TAG_FITTING = 4
comptime TAG_TEE = 5


def batch_compute(
    types: List[Int],                # length N
    params: List[Float64],           # length 6*N (row-major)
    port_idx: List[Int],             # length 3*N (row-major)
    flows: List[Float64],            # length P (per-port flow, in)
    density: Float64,
    kinematic_viscosity: Float64,
) raises -> Tuple[List[Float64], List[Float64]]:
    """Run the full pressure-drop pass; return ``(velocities, dps)`` lists of
    length P (per-port)."""
    var p = len(flows)
    var velocities = List[Float64](length=p, fill=0.0)
    var dps = List[Float64](length=p, fill=0.0)
    var n = len(types)

    for i in range(n):
        var tag = types[i]
        var p0 = params[i * 6 + 0]
        var p1 = params[i * 6 + 1]
        var p2 = params[i * 6 + 2]
        var p3 = params[i * 6 + 3]
        var p4 = params[i * 6 + 4]
        var ix0 = port_idx[i * 3 + 0]
        var ix1 = port_idx[i * 3 + 1]
        var ix2 = port_idx[i * 3 + 2]

        if tag == TAG_SOURCE:
            # No drop. Out-port v = 0 (the solver doesn't track source v).
            pass
        elif tag == TAG_TERMINAL:
            # Terminal: in-port idx = ix0. Optional cross_section → p0=area, p1=zeta.
            if p0 > 0.0:
                var v = flows[ix0] / p0
                velocities[ix0] = v
                dps[ix0] = p1 * density * v * v * 0.5
        elif tag == TAG_RIGID:
            # params: area, d_h, length, abs_rough
            var v = flows[ix0] / p0
            var re = reynolds(v, p1, kinematic_viscosity)
            var eps = relative_roughness(p3, p1)
            var f = friction_factor(re, eps)
            velocities[ix0] = v
            velocities[ix1] = v
            dps[ix0] = f * (p2 / p1) * density * v * v * 0.5
            # outlet dp stays 0 by initialisation
        elif tag == TAG_FLEX:
            # params: area, diameter, length, pdpm, stretch%
            var v = flows[ix0] / p0
            var beta = 0.557 * (100.0 - p4) * exp(-4.93 * p1) + 1.0
            velocities[ix0] = v
            velocities[ix1] = v
            dps[ix0] = p3 * p2 * beta
        elif tag == TAG_FITTING:
            # params: area, zeta. dp on out-port.
            var v = flows[ix0] / p0
            velocities[ix0] = v
            velocities[ix1] = v
            dps[ix1] = p1 * density * v * v * 0.5
        elif tag == TAG_TEE:
            # params: area, zeta_straight, zeta_branch.
            # ix0=combined_in, ix1=straight_out, ix2=branch_out.
            var inv_a = 1.0 / p0
            var v_s = flows[ix1] * inv_a
            var v_b = flows[ix2] * inv_a
            var v_c = flows[ix0] * inv_a
            velocities[ix0] = v_c
            velocities[ix1] = v_s
            velocities[ix2] = v_b
            dps[ix1] = p1 * density * v_s * v_s * 0.5
            dps[ix2] = p2 * density * v_b * v_b * 0.5

    return Tuple[List[Float64], List[Float64]](velocities^, dps^)
