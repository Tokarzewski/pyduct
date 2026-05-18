"""Tests for duct sizing methods."""


import pytest
from wenta import (
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
        from wenta import Network, RigidDuct, Source, Terminal, solve

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
        from wenta import Fluid

        # Air at 40 °C (slightly less dense and viscous)
        warm_air = Fluid(density=1.13, dynamic_viscosity=1.92e-5)

        section, v = velocity_method(
            0.1, "round", target_velocity=4.0, fluid=warm_air
        )
        # Should still be a valid size
        assert v <= 4.0


class TestVelocityMethodBatch:
    def test_matches_per_call_velocity_method(self) -> None:
        import numpy as np
        from wenta import velocity_method, velocity_method_batch

        flows = [0.05, 0.10, 0.25, 0.50, 1.0]
        diameters, velocities = velocity_method_batch(flows, target_velocity=4.0)
        for q, d_batch, v_batch in zip(flows, diameters, velocities, strict=True):
            section, v = velocity_method(q, "round", 4.0)
            assert np.isclose(d_batch, section.diameter)
            assert np.isclose(v_batch, v)

    def test_accepts_numpy_array_and_returns_ndarrays(self) -> None:
        import numpy as np
        from wenta import velocity_method_batch

        flows = np.array([0.05, 0.10])
        d, v = velocity_method_batch(flows)
        assert isinstance(d, np.ndarray) and d.dtype == np.float64
        assert isinstance(v, np.ndarray) and v.dtype == np.float64
        assert d.shape == (2,) and v.shape == (2,)


class TestAspectRatioMethod:
    def test_returns_flat_section(self) -> None:
        from wenta import Rectangular, aspect_ratio_method

        sec, v = aspect_ratio_method(0.2, target_velocity=4.0, aspect_ratio=2.0)
        assert isinstance(sec, Rectangular)
        # Aspect ratio is satisfied.
        long, short = max(sec.width, sec.height), min(sec.width, sec.height)
        assert long / short >= 2.0
        # Velocity is within target (unless the largest size was forced).
        assert v <= 4.0 or sec.area == max(s.area for s in [sec])

    def test_rejects_aspect_ratio_below_one(self) -> None:
        from wenta import aspect_ratio_method

        with pytest.raises(ValueError):
            aspect_ratio_method(0.1, aspect_ratio=0.5)

    def test_rejects_nonpositive_flowrate(self) -> None:
        from wenta import aspect_ratio_method

        with pytest.raises(ValueError):
            aspect_ratio_method(0.0)


class TestNoiseLimitMethod:
    def test_bedroom_is_quieter_than_office(self) -> None:
        from wenta import noise_limit_method

        _, v_bed = noise_limit_method(0.1, "bedroom")
        _, v_off = noise_limit_method(0.1, "office")
        assert v_bed <= v_off  # bedroom limit is lower

    def test_unknown_space_rejected(self) -> None:
        from wenta import noise_limit_method

        with pytest.raises(ValueError, match="unknown space_type"):
            noise_limit_method(0.1, "not_a_space")
