"""Numerical parity tests: Mojo port vs. Python reference (`pyduct`).

Follows the Branch-C pattern from the `migration-to-python-mojo` skill —
the Python implementation is the authoritative oracle, and every Mojo
unit must reproduce its outputs within a tight tolerance on a corpus of
representative inputs.

Run with the project's uv venv active so the `pyduct` Python package is
importable:

    uv run mojo run mojoduct/tests/test_parity.mojo
"""

from std.math import abs
from std.python import Python, PythonObject
from std.testing import TestSuite, assert_true

from mojoduct.core.geometry import Round, Rectangular, equivalent_round_diameter
from mojoduct.core.fluid import air_at_altitude
from mojoduct.physics.friction import (
    reynolds,
    relative_roughness,
    friction_factor,
    friction_factor_colebrook,
)
from mojoduct.physics.flex import stretch_correction_factor
from mojoduct.physics.losses import straight_pressure_drop, local_pressure_drop
from mojoduct.data.standard_sizes import nearest_round_size
from mojoduct.sizing import (
    velocity_method_round,
    equal_friction_method_round,
    pressure_drop_budget_round,
)
from mojoduct.components.fittings_library import rectangular_elbow, mitered_elbow
from mojoduct.units import cfm_to_m3s, inwc_to_pa, ft_to_m, air_changes_per_hour


# --- helpers ---------------------------------------------------------------


def _close_rel(a: Float64, b: Float64, rtol: Float64 = 1e-12, atol: Float64 = 1e-15) -> Bool:
    """Numpy-style isclose: |a - b| <= atol + rtol * |b|."""
    var diff = a - b if a >= b else b - a
    var mag = b if b >= 0.0 else -b
    return diff <= atol + rtol * mag


# --- geometry parity --------------------------------------------------------


def test_parity_round_geometry() raises:
    var py_geo = Python.import_module("pyduct.core.geometry")
    var diameters = [0.063, 0.1, 0.2, 0.355, 0.8, 1.25]
    for d in diameters:
        var py_obj = py_geo.Round(d)
        var py_area = Float64(py=py_obj.area)
        var py_dh = Float64(py=py_obj.hydraulic_diameter)
        var mojo_obj = Round(d)
        assert_true(_close_rel(mojo_obj.area, py_area))
        assert_true(_close_rel(mojo_obj.hydraulic_diameter, py_dh))


def test_parity_rectangular_geometry() raises:
    var py_geo = Python.import_module("pyduct.core.geometry")
    var pairs = [(0.2, 0.2), (0.3, 0.5), (0.4, 1.0), (1.2, 0.5)]
    for pair in pairs:
        var w = pair[0]
        var h = pair[1]
        var py_obj = py_geo.Rectangular(w, h)
        var py_area = Float64(py=py_obj.area)
        var py_dh = Float64(py=py_obj.hydraulic_diameter)
        var mojo_obj = Rectangular(w, h)
        assert_true(_close_rel(mojo_obj.area, py_area))
        assert_true(_close_rel(mojo_obj.hydraulic_diameter, py_dh))


def test_parity_equivalent_round_diameter() raises:
    var py_geo = Python.import_module("pyduct.core.geometry")
    var pairs = [(0.2, 0.2), (0.3, 0.5), (0.4, 1.0), (1.2, 0.5)]
    for pair in pairs:
        var w = pair[0]
        var h = pair[1]
        var py_val = Float64(py=py_geo.equivalent_round_diameter(w, h))
        var mojo_val = equivalent_round_diameter(w, h)
        # Transcendentals: allow libm-implementation rounding noise.
        assert_true(_close_rel(mojo_val, py_val, rtol=1e-9))


# --- fluid parity -----------------------------------------------------------


def test_parity_air_at_altitude() raises:
    var py_fl = Python.import_module("pyduct.core.fluid")
    var cases = [
        (0.0, 20.0), (500.0, 20.0), (1500.0, 15.0), (3000.0, 0.0), (5000.0, -10.0),
    ]
    for c in cases:
        var alt = c[0]
        var t = c[1]
        var py_f = py_fl.air_at_altitude(alt, t)
        var mojo_f = air_at_altitude(alt, t)
        # Pressure/viscosity formulas use **5.2561 and T^1.5; libm-rounding slack.
        assert_true(_close_rel(mojo_f.density, Float64(py=py_f.density), rtol=1e-9))
        assert_true(
            _close_rel(
                mojo_f.dynamic_viscosity,
                Float64(py=py_f.dynamic_viscosity),
                rtol=1e-9,
            )
        )
        assert_true(
            _close_rel(
                mojo_f.kinematic_viscosity,
                Float64(py=py_f.kinematic_viscosity),
                rtol=1e-9,
            )
        )


# --- friction parity --------------------------------------------------------


def test_parity_reynolds() raises:
    var py_fr = Python.import_module("pyduct.physics.friction")
    var nu = 1.5e-5
    var cases = [
        (0.5, 0.1), (3.0, 0.2), (10.0, 0.4), (25.0, 0.8),
    ]
    for c in cases:
        var v = c[0]
        var dh = c[1]
        var py_re = Float64(py=py_fr.reynolds(v, dh, nu))
        var mj_re = reynolds(v, dh, nu)
        assert_true(_close_rel(mj_re, py_re))


def test_parity_relative_roughness() raises:
    var py_fr = Python.import_module("pyduct.physics.friction")
    var cases = [
        (0.0001, 0.1), (0.0001, 0.5), (0.0005, 0.2), (0.001, 0.4),
    ]
    for c in cases:
        var eps = c[0]
        var dh = c[1]
        var py_eps = Float64(py=py_fr.relative_roughness(eps, dh))
        var mj_eps = relative_roughness(eps, dh)
        assert_true(_close_rel(mj_eps, py_eps))


def test_parity_friction_factor_swamee_jain() raises:
    var py_fr = Python.import_module("pyduct.physics.friction")
    # Sweep laminar + turbulent regimes; eps from smooth to very rough.
    var res = [500.0, 1000.0, 5000.0, 5.0e4, 1.0e5, 1.0e6, 1.0e7]
    var epses = [0.0, 1.0e-5, 1.0e-4, 5.0e-4, 1.0e-3, 5.0e-2]
    for re in res:
        for eps in epses:
            var py_f = Float64(py=py_fr.friction_factor(re, eps))
            var mj_f = friction_factor(re, eps)
            # Closed-form Swamee-Jain — libm log/** rounding accounts for any
            # last-bit difference. 1e-9 is still 5+ orders tighter than HVAC use.
            assert_true(_close_rel(mj_f, py_f, rtol=1e-9))


def test_parity_friction_factor_colebrook() raises:
    var py_fr = Python.import_module("pyduct.physics.friction")
    # The fixed-point iteration may take a slightly different number of
    # steps in either side; allow a hair more slack than for the closed
    # form, but still much tighter than test tolerance.
    var cases = [
        (5.0e3, 1.0e-4), (5.0e4, 5.0e-4), (1.0e5, 1.0e-3), (1.0e6, 5.0e-2),
    ]
    for c in cases:
        var re = c[0]
        var eps = c[1]
        var py_f = Float64(py=py_fr.friction_factor_colebrook(re, eps))
        var mj_f = friction_factor_colebrook(re, eps)
        assert_true(_close_rel(mj_f, py_f, rtol=1e-9))


# --- losses parity ----------------------------------------------------------


def test_parity_straight_pressure_drop() raises:
    var py_ls = Python.import_module("pyduct.physics.losses")
    var cases = [
        (0.02, 10.0, 0.2, 5.0, 1.2),
        (0.018, 25.0, 0.3, 4.0, 1.18),
        (0.025, 50.0, 0.15, 7.0, 1.225),
    ]
    for c in cases:
        var f = c[0]
        var L = c[1]
        var dh = c[2]
        var v = c[3]
        var rho = c[4]
        var py_dp = Float64(py=py_ls.straight_pressure_drop(f, L, dh, v, rho))
        var mj_dp = straight_pressure_drop(f, L, dh, v, rho)
        assert_true(_close_rel(mj_dp, py_dp))


def test_parity_local_pressure_drop() raises:
    var py_ls = Python.import_module("pyduct.physics.losses")
    var cases = [
        (0.5, 4.0, 1.2), (0.8, 6.0, 1.18), (1.5, 3.5, 1.225),
    ]
    for c in cases:
        var zeta = c[0]
        var v = c[1]
        var rho = c[2]
        var py_dp = Float64(py=py_ls.local_pressure_drop(zeta, v, rho))
        var mj_dp = local_pressure_drop(zeta, v, rho)
        assert_true(_close_rel(mj_dp, py_dp))


# --- units parity -----------------------------------------------------------


def test_parity_units() raises:
    var py_u = Python.import_module("pyduct.units")
    var samples = [0.0, 1.0, 250.0, 10_000.0]
    for v in samples:
        assert_true(_close_rel(cfm_to_m3s(v), Float64(py=py_u.cfm_to_m3s(v))))
        assert_true(_close_rel(inwc_to_pa(v), Float64(py=py_u.inwc_to_pa(v))))
        assert_true(_close_rel(ft_to_m(v), Float64(py=py_u.ft_to_m(v))))
    # ACH parity over a few rooms.
    var room_cases = [(0.01, 36.0), (0.05, 120.0), (0.2, 800.0)]
    for c in room_cases:
        var flow = c[0]
        var vol = c[1]
        var py_ach = Float64(py=py_u.air_changes_per_hour(flow, vol))
        var mj_ach = air_changes_per_hour(flow, vol)
        assert_true(_close_rel(mj_ach, py_ach))


def test_parity_stretch_correction_factor() raises:
    var py_flex = Python.import_module("pyduct.physics.flex")
    var cases = [
        (0.1, 100.0), (0.15, 80.0), (0.2, 50.0), (0.315, 70.0), (0.4, 100.0),
    ]
    for c in cases:
        var d = c[0]
        var s = c[1]
        var py_v = Float64(py=py_flex.stretch_correction_factor(d, s))
        var mj_v = stretch_correction_factor(d, s)
        # exp() — same libm-rounding slack as the friction test.
        assert_true(_close_rel(mj_v, py_v, rtol=1e-9))


def test_parity_velocity_method_round() raises:
    var py_sizing = Python.import_module("pyduct.sizing")
    var cases = [
        (0.05, 4.0), (0.10, 5.0), (0.25, 3.5), (0.50, 4.0), (1.0, 3.0), (5.0, 4.0),
    ]
    for c in cases:
        var flow = c[0]
        var target_v = c[1]
        var py_result = py_sizing.velocity_method(flow, "round", target_v)
        var py_section = py_result[0]
        var py_v = Float64(py=py_result[1])
        var py_d = Float64(py=py_section.diameter)

        var mj_pair = velocity_method_round(flow, target_v)
        # Identical EN-1506 size and bit-identical velocity.
        assert_true(_close_rel(mj_pair[0].diameter, py_d))
        assert_true(_close_rel(mj_pair[1], py_v))


def test_parity_equal_friction_method_round() raises:
    var py_sizing = Python.import_module("pyduct.sizing")
    var cases = [
        (0.05, 1.0), (0.10, 1.0), (0.10, 0.5), (0.25, 1.5), (0.50, 1.0), (1.0, 0.8),
    ]
    for c in cases:
        var flow = c[0]
        var target = c[1]
        var py_r = py_sizing.equal_friction_method(flow, target, "round")
        var py_d = Float64(py=py_r[0].diameter)
        var py_v = Float64(py=py_r[1])
        var py_r_per_m = Float64(py=py_r[2])
        var mj = equal_friction_method_round(flow, target)
        assert_true(_close_rel(mj[0].diameter, py_d))
        assert_true(_close_rel(mj[1], py_v))
        # r_per_m goes through log/**: libm slack.
        assert_true(_close_rel(mj[2], py_r_per_m, rtol=1e-9))


def test_parity_pressure_drop_budget_round() raises:
    var py_sizing = Python.import_module("pyduct.sizing")
    var cases = [(0.05, 10.0, 10.0), (0.10, 20.0, 30.0), (0.25, 50.0, 75.0)]
    for c in cases:
        var flow = c[0]
        var length = c[1]
        var budget = c[2]
        var py_r = py_sizing.pressure_drop_budget(flow, length, budget, "round")
        var py_d = Float64(py=py_r[0].diameter)
        var py_v = Float64(py=py_r[1])
        var py_r_per_m = Float64(py=py_r[2])
        var mj = pressure_drop_budget_round(flow, length, budget)
        assert_true(_close_rel(mj[0].diameter, py_d))
        assert_true(_close_rel(mj[1], py_v))
        assert_true(_close_rel(mj[2], py_r_per_m, rtol=1e-9))


def test_parity_rectangular_elbow() raises:
    var py_fits = Python.import_module("pyduct.components.fittings_library")
    var cases = [
        (0.4, 0.3, 0.2, 90.0),
        (0.4, 0.3, 0.6, 90.0),
        (0.6, 0.2, 0.3, 90.0),
        (0.4, 0.3, 0.3, 45.0),
    ]
    for c in cases:
        var w = c[0]
        var h = c[1]
        var r = c[2]
        var ang = c[3]
        var py_v = Float64(py=py_fits.rectangular_elbow(w, h, r, ang))
        var mj_v = rectangular_elbow(w, h, r, ang)
        assert_true(_close_rel(mj_v, py_v, rtol=1e-9))


def test_parity_mitered_elbow() raises:
    var py_fits = Python.import_module("pyduct.components.fittings_library")
    var angles = [45.0, 60.0, 90.0, 120.0]
    for a in angles:
        for vaned in [False, True]:
            var py_v = Float64(py=py_fits.mitered_elbow(a, vaned=vaned))
            var mj_v = mitered_elbow(a, vaned=vaned)
            assert_true(_close_rel(mj_v, py_v))


def test_parity_nearest_round_size() raises:
    var py_data = Python.import_module("pyduct.data.standard_sizes")
    var queries = [50.0, 100.0, 247.5, 247.6, 500.0, 700.0, 1300.0]
    # round_up = True
    for q in queries:
        var py_v = Int(py=py_data.nearest_round_size(q, round_up=True))
        var mj_v = nearest_round_size(q, round_up=True)
        assert_true(mj_v == py_v)
    # round_up = False
    for q in queries:
        var py_v = Int(py=py_data.nearest_round_size(q, round_up=False))
        var mj_v = nearest_round_size(q, round_up=False)
        assert_true(mj_v == py_v)


def main() raises:
    TestSuite.discover_tests[__functions_in_module()]().run()
