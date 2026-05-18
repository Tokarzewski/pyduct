"""Mojo tests for the native core / physics / units modules.

Run with:

    uv run mojo run wentamojo/tests/test_core.mojo
"""

from std.math import pi, isclose
from std.testing import TestSuite, assert_true, assert_equal, assert_raises

from wentamojo.core.geometry import Round, Rectangular, equivalent_round_diameter
from wentamojo.core.fluid import Fluid, standard_air, air_at_altitude
from wentamojo.physics.friction import (
    reynolds,
    relative_roughness,
    friction_factor,
    friction_factor_colebrook,
)
from wentamojo.physics.losses import straight_pressure_drop, local_pressure_drop
from wentamojo.units import (
    cfm_to_m3s,
    m3s_to_cfm,
    inwc_to_pa,
    pa_to_inwc,
    f_to_c,
    c_to_f,
    air_changes_per_hour,
)


# --- helpers ---------------------------------------------------------------


def _close(a: Float64, b: Float64, tol: Float64 = 1e-9) -> Bool:
    var d = a - b if a >= b else b - a
    return d < tol


# --- geometry --------------------------------------------------------------


def test_round_area_and_hydraulic_diameter() raises:
    var r = Round(2.0)
    assert_true(_close(r.area, pi))
    assert_true(_close(r.hydraulic_diameter, 2.0))


def test_round_rejects_nonpositive_diameter() raises:
    with assert_raises():
        _ = Round(0.0)
    with assert_raises():
        _ = Round(-0.5)


def test_rectangular_geometry() raises:
    var s = Rectangular(0.5, 0.4)
    assert_true(_close(s.area, 0.20))
    # 2 * 0.5 * 0.4 / 0.9
    assert_true(_close(s.hydraulic_diameter, 2.0 * 0.5 * 0.4 / 0.9))


def test_rectangular_rejects_nonpositive() raises:
    with assert_raises():
        _ = Rectangular(0.0, 0.3)
    with assert_raises():
        _ = Rectangular(0.3, -0.1)


def test_equivalent_round_diameter_known() raises:
    # 1.30 * (0.3*0.3)^0.625 / 0.6^0.25
    var d = equivalent_round_diameter(0.3, 0.3)
    var expected = 1.30 * (0.3 * 0.3) ** 0.625 / (0.6) ** 0.25
    assert_true(_close(d, expected, tol=1e-12))


# --- fluid -----------------------------------------------------------------


def test_standard_air_matches_constants() raises:
    var f = standard_air()
    assert_true(_close(f.density, 1.204))
    assert_true(_close(f.dynamic_viscosity, 1.825e-5, tol=1e-12))
    assert_true(_close(f.kinematic_viscosity, 1.825e-5 / 1.204, tol=1e-12))


def test_air_at_altitude_density_decreases() raises:
    var low = air_at_altitude(0.0)
    var high = air_at_altitude(2000.0)
    assert_true(high.density < low.density)


def test_air_at_altitude_rejects_negative() raises:
    with assert_raises():
        _ = air_at_altitude(-1.0)


# --- friction --------------------------------------------------------------


def test_reynolds_basic() raises:
    var nu = 1.5e-5
    var re = reynolds(velocity=10.0, hydraulic_diameter=0.2, kinematic_viscosity=nu)
    assert_true(_close(re, 10.0 * 0.2 / nu, tol=1e-9))


def test_laminar_friction_factor() raises:
    var f = friction_factor(reynolds_number=1000.0, rel_roughness=0.0)
    assert_true(_close(f, 64.0 / 1000.0))


def test_turbulent_friction_factor_range() raises:
    # Re = 1e5, eps = 1e-4 — should fall in the standard 0.015–0.025 band.
    var f = friction_factor(reynolds_number=1.0e5, rel_roughness=1.0e-4)
    assert_true(f > 0.015 and f < 0.030)


def test_colebrook_matches_swamee_jain_within_5pct() raises:
    var re = 5.0e4
    var eps = 5.0e-4
    var f_sj = friction_factor(re, eps)
    var f_col = friction_factor_colebrook(re, eps)
    var ratio = f_sj / f_col if f_sj >= f_col else f_col / f_sj
    assert_true(ratio < 1.05)


# --- losses ----------------------------------------------------------------


def test_straight_pressure_drop() raises:
    # dp = f * (L/D_h) * rho * v^2 / 2
    var dp = straight_pressure_drop(
        friction_factor=0.02, length=10.0, hydraulic_diameter=0.2,
        velocity=5.0, density=1.2,
    )
    var expected = 0.02 * (10.0 / 0.2) * 1.2 * 25.0 * 0.5
    assert_true(_close(dp, expected))


def test_local_pressure_drop() raises:
    var dp = local_pressure_drop(zeta=0.5, velocity=4.0, density=1.2)
    var expected = 0.5 * 1.2 * 16.0 * 0.5
    assert_true(_close(dp, expected))


# --- units -----------------------------------------------------------------


def test_cfm_roundtrip() raises:
    assert_true(_close(m3s_to_cfm(cfm_to_m3s(2000.0)), 2000.0, tol=1e-9))


def test_inwc_roundtrip() raises:
    assert_true(_close(pa_to_inwc(inwc_to_pa(1.0)), 1.0, tol=1e-9))


def test_inwc_to_pa_known() raises:
    assert_true(_close(inwc_to_pa(1.0), 249.0889, tol=1e-3))


def test_temperature_roundtrip() raises:
    assert_true(_close(f_to_c(32.0), 0.0))
    assert_true(_close(f_to_c(212.0), 100.0))
    assert_true(_close(c_to_f(0.0), 32.0))


def test_air_changes_per_hour() raises:
    # 0.01 m^3/s into a 36 m^3 room is 1 ACH.
    assert_true(_close(air_changes_per_hour(0.01, 36.0), 1.0, tol=1e-9))


def test_ach_rejects_zero_volume() raises:
    with assert_raises():
        _ = air_changes_per_hour(0.1, 0.0)


def main() raises:
    TestSuite.discover_tests[__functions_in_module()]().run()
