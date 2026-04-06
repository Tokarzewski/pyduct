"""Tests for friction-factor correlations."""

from math import isclose

import pytest

from pyduct2.physics.friction import (
    friction_factor,
    friction_factor_colebrook,
    relative_roughness,
    reynolds,
)


def test_reynolds_basic() -> None:
    # v = 5 m/s, D = 0.2 m, nu = 1.516e-5 m^2/s -> Re ~ 65 962
    re = reynolds(5.0, 0.2, 1.516e-5)
    assert isclose(re, 5.0 * 0.2 / 1.516e-5)


def test_relative_roughness() -> None:
    assert relative_roughness(0.0001, 0.2) == pytest.approx(0.0005)


def test_friction_factor_laminar() -> None:
    # Re < 2300 -> 64/Re regardless of roughness.
    assert friction_factor(1000, 0.01) == pytest.approx(0.064)
    assert friction_factor_colebrook(1000, 0.01) == pytest.approx(0.064)


@pytest.mark.parametrize("re", [5_000, 25_000, 100_000, 1_000_000])
@pytest.mark.parametrize("eps", [1e-5, 1e-4, 1e-3, 1e-2])
def test_swamee_jain_matches_colebrook(re: float, eps: float) -> None:
    f_sj = friction_factor(re, eps)
    f_cb = friction_factor_colebrook(re, eps)
    # Swamee–Jain is within ~1% of Colebrook in this range.
    assert abs(f_sj - f_cb) / f_cb < 0.02


def test_colebrook_converges() -> None:
    f = friction_factor_colebrook(50_000, 0.001)
    # Spot value cross-checked against the explicit formula.
    assert 0.02 < f < 0.03
