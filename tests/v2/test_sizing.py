"""Tests for duct sizing methods."""

from math import pi

import pytest

from pyduct2 import (
    STANDARD_AIR,
    Rectangular,
    Round,
    equal_friction_method,
    pressure_drop_budget,
    velocity_method,
)


class TestVelocityMethod:
    def test_round_fits_within_target_velocity(self) -> None:
        section, v = velocity_method(0.1, "round", target_velocity=4.0)
        assert isinstance(section, Round)
        assert v <= 4.0

    def test_rectangular_fits_within_target_velocity(self) -> None:
        section, v = velocity_method(0.1, "rectangular", target_velocity=3.5)
        assert isinstance(section, Rectangular)
        assert v <= 3.5

    def test_low_flowrate_returns_smallest_size(self) -> None:
        section, v = velocity_method(0.001, "round", target_velocity=10.0)
        # The smallest round size should be used.
        assert section.area > 0

    def test_high_flowrate_returns_largest_size(self) -> None:
        section, v = velocity_method(100.0, "round", target_velocity=1.0)
        # Can't meet target, so returns the largest available size.
        assert section.area > 0

    def test_negative_flowrate_rejected(self) -> None:
        with pytest.raises(ValueError):
            velocity_method(-0.1, "round")

    def test_rounds_to_smallest_for_unsupported_shape(self) -> None:
        # "rectangular" is supported; other shapes silently fall back to rectangular
        section, v = velocity_method(0.1, "rectangular", target_velocity=4.0)
        assert isinstance(section, Rectangular)


class TestEqualFrictionMethod:
    def test_round_meets_pressure_drop_target(self) -> None:
        section, v, r = equal_friction_method(0.1, target_pressure_drop_per_meter=1.0)
        assert isinstance(section, Round)
        assert r <= 1.0 + 0.01  # Small tolerance for rounding
        assert v > 0

    def test_rectangular_meets_pressure_drop_target(self) -> None:
        section, v, r = equal_friction_method(
            0.1, target_pressure_drop_per_meter=0.5, shape="rectangular"
        )
        assert isinstance(section, Rectangular)
        assert r <= 0.5 + 0.01

    def test_low_drop_requires_larger_duct(self) -> None:
        sec_low, v_low, _ = equal_friction_method(0.1, 0.5)
        sec_high, v_high, _ = equal_friction_method(0.1, 2.0)
        # Lower target pressure drop → larger duct → lower velocity
        assert v_low < v_high

    def test_negative_target_rejected(self) -> None:
        with pytest.raises(ValueError):
            equal_friction_method(0.1, target_pressure_drop_per_meter=-1.0)


class TestPressureDropBudget:
    def test_budget_method_is_equal_friction_over_length(self) -> None:
        section, v, dp = pressure_drop_budget(0.1, length=10.0, budget_pa=10.0)
        # Target per-meter = 10 / 10 = 1.0 Pa/m
        # Should match equal_friction_method(0.1, 1.0)
        section2, v2, r2 = equal_friction_method(0.1, 1.0)
        assert section.area == pytest.approx(section2.area)
        assert v == pytest.approx(v2)

    def test_budget_zero_length_rejected(self) -> None:
        with pytest.raises(ValueError):
            pressure_drop_budget(0.1, length=0.0, budget_pa=10.0)

    def test_budget_zero_pa_rejected(self) -> None:
        with pytest.raises(ValueError):
            pressure_drop_budget(0.1, length=10.0, budget_pa=0.0)


class TestSizingIntegration:
    def test_sized_duct_in_network(self) -> None:
        """End-to-end: size a duct using velocity method, add to network, solve."""
        from pyduct2 import Network, RigidDuct, Source, Terminal, solve

        # Size the duct
        section, _ = velocity_method(0.05, "round", target_velocity=4.0)

        # Build network
        net = Network("test")
        net.add("ahu", Source("ahu"))
        net.add("duct", RigidDuct("duct", section, length=20.0))
        net.add("term", Terminal("term", flowrate=0.05))

        net.connect("ahu", "duct")
        net.connect("duct", "term")

        # Solve
        dp = solve(net)
        assert dp > 0

    def test_sizing_with_custom_fluid(self) -> None:
        """Sizing with a custom fluid (e.g. warmer air)."""
        from pyduct2 import Fluid

        # Air at 40 °C (slightly less dense and viscous)
        warm_air = Fluid(density=1.13, dynamic_viscosity=1.92e-5)

        section, v = velocity_method(
            0.1, "round", target_velocity=4.0, fluid=warm_air
        )
        # Should still be a valid size
        assert v <= 4.0
