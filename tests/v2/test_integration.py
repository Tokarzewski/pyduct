"""End-to-end integration tests: realistic use cases."""

import pytest

from pyduct2 import (
    ElbowRound,
    Network,
    RigidDuct,
    Round,
    Source,
    Tee,
    Terminal,
    TwoPortFitting,
    critical_path_pressure_drop,
    equal_friction_method,
    extract_results,
    results_summary,
    solve,
    velocity_method,
)
from pyduct2.components.fittings_library import (
    damper_butterfly,
    junction_tee_branch,
)


class TestSupplyNetworkWithSizing:
    """Design a supply network using the sizing methods."""

    def test_design_main_duct_and_branches(self) -> None:
        # Main trunk: 0.15 m^3/s, sized by velocity method
        main_section, main_v = velocity_method(0.15, "round", target_velocity=5.0)
        assert main_v <= 5.0

        # Branch 1: 0.09 m^3/s
        br1_section, _ = velocity_method(0.09, "round", target_velocity=4.0)

        # Branch 2: 0.06 m^3/s
        br2_section, _ = velocity_method(0.06, "round", target_velocity=4.0)

        # Build network
        net = Network("supply")
        net.add("ahu", Source("ahu"))
        net.add("main_duct", RigidDuct("main", main_section, length=25))
        net.add("tee", Tee("tee_main", main_section))
        net.add("br1_duct", RigidDuct("br1", br1_section, length=15))
        net.add("br2_duct", RigidDuct("br2", br2_section, length=20))
        net.add("term1", Terminal("term1", flowrate=0.09))
        net.add("term2", Terminal("term2", flowrate=0.06))

        net.connect("ahu", "main_duct")
        net.connect("main_duct", "tee.combined")
        net.connect("tee.straight", "br1_duct")
        net.connect("br1_duct", "term1")
        net.connect("tee.branch", "br2_duct")
        net.connect("br2_duct", "term2")

        dp = solve(net)
        assert dp > 0
        # Both terminals should get adequate pressure
        assert dp < 500  # Reasonable limit


class TestNetworkWithFittings:
    """Build a network with various fittings."""

    def test_elbow_in_main_ductwork(self) -> None:
        section = Round(0.25)

        net = Network("with_elbow")
        net.add("ahu", Source("ahu"))

        # Add an elbow (90 °, R/D = 1.0)
        elbow = ElbowRound(bend_radius=0.25, diameter=0.25, angle=90)
        net.add("elbow", TwoPortFitting("elbow_90", section, zeta=elbow.zeta))

        net.add("duct", RigidDuct("duct", section, length=15))
        net.add("term", Terminal("term", flowrate=0.1))

        net.connect("ahu", "elbow")
        net.connect("elbow", "duct")
        net.connect("duct", "term")

        dp = solve(net)
        # Pressure drop should include the elbow loss
        assert dp > 0

    def test_damper_in_branch(self) -> None:
        section = Round(0.2)

        net = Network("with_damper")
        net.add("ahu", Source("ahu"))
        net.add("d1", RigidDuct("d1", section, length=10))
        net.add("tee", Tee("tee", section))

        # Branch with damper
        damper_zeta = damper_butterfly(75)  # 75% open
        net.add("damper", TwoPortFitting("damper", section, zeta=damper_zeta))

        net.add("br_duct", RigidDuct("br", section, length=8))
        net.add("term1", Terminal("term1", flowrate=0.05))
        net.add("term2", Terminal("term2", flowrate=0.05))

        net.connect("ahu", "d1")
        net.connect("d1", "tee.combined")
        net.connect("tee.straight", "term1")
        net.connect("tee.branch", "damper")
        net.connect("damper", "br_duct")
        net.connect("br_duct", "term2")

        dp = solve(net)
        assert dp > 0

    def test_multiple_elbows_in_series(self) -> None:
        section = Round(0.15)

        net = Network("multi_elbow")
        net.add("ahu", Source("ahu"))

        # Series of elbows
        elbow = ElbowRound(bend_radius=0.15, diameter=0.15, angle=90)
        for i in range(3):
            net.add(
                f"elbow_{i}",
                TwoPortFitting(f"elbow_{i}", section, zeta=elbow.zeta),
            )

        net.add("duct", RigidDuct("duct", section, length=5))
        net.add("term", Terminal("term", flowrate=0.05))

        # Chain them
        net.connect("ahu", "elbow_0")
        net.connect("elbow_0", "elbow_1")
        net.connect("elbow_1", "elbow_2")
        net.connect("elbow_2", "duct")
        net.connect("duct", "term")

        dp = solve(net)
        # Three elbows plus duct should give measurable drop
        assert dp > 5


class TestResultsExtraction:
    def test_results_table_from_complex_network(self) -> None:
        net = Network("complex")
        section = Round(0.2)

        net.add("ahu", Source("ahu"))
        net.add("d1", RigidDuct("d1", section, length=15))
        net.add("tee", Tee("tee", section))
        net.add("d2", RigidDuct("d2", section, length=10))
        net.add("d3", RigidDuct("d3", section, length=12))
        net.add("t1", Terminal("term1", flowrate=0.08))
        net.add("t2", Terminal("term2", flowrate=0.12))

        net.connect("ahu", "d1")
        net.connect("d1", "tee.combined")
        net.connect("tee.straight", "d2")
        net.connect("d2", "t1")
        net.connect("tee.branch", "d3")
        net.connect("d3", "t2")

        solve(net)

        results = extract_results(net)
        assert len(results) == 7  # ahu, d1, tee, d2, d3, t1, t2

        # Verify each result has expected data
        for res in results:
            assert res.component_id
            assert res.component_type
            assert res.pressure_drop >= 0

    def test_results_summary_readable(self) -> None:
        net = Network("test")
        section = Round(0.2)
        net.add("ahu", Source("ahu"))
        net.add("d1", RigidDuct("d1", section, length=10))
        net.add("term", Terminal("term", flowrate=0.1))
        net.connect("ahu", "d1")
        net.connect("d1", "term")
        solve(net)

        summary = results_summary(net)
        # Should contain component names and pressures
        assert "ahu" in summary
        assert "Pa" in summary


class TestPressureDropVerification:
    """Verify pressure drop calculations are physically sensible."""

    def test_longer_duct_higher_pressure_drop(self) -> None:
        section = Round(0.2)

        # Short duct
        net1 = Network("short")
        net1.add("ahu", Source("ahu"))
        net1.add("d", RigidDuct("d", section, length=5))
        net1.add("term", Terminal("term", flowrate=0.1))
        net1.connect("ahu", "d")
        net1.connect("d", "term")
        dp1 = solve(net1)

        # Long duct
        net2 = Network("long")
        net2.add("ahu", Source("ahu"))
        net2.add("d", RigidDuct("d", section, length=20))
        net2.add("term", Terminal("term", flowrate=0.1))
        net2.connect("ahu", "d")
        net2.connect("d", "term")
        dp2 = solve(net2)

        # Longer duct should have higher pressure drop
        assert dp2 > dp1

    def test_higher_flowrate_higher_pressure_drop(self) -> None:
        section = Round(0.2)

        # Low flow
        net1 = Network("low_flow")
        net1.add("ahu", Source("ahu"))
        net1.add("d", RigidDuct("d", section, length=10))
        net1.add("term", Terminal("term", flowrate=0.05))
        net1.connect("ahu", "d")
        net1.connect("d", "term")
        dp1 = solve(net1)

        # High flow
        net2 = Network("high_flow")
        net2.add("ahu", Source("ahu"))
        net2.add("d", RigidDuct("d", section, length=10))
        net2.add("term", Terminal("term", flowrate=0.15))
        net2.connect("ahu", "d")
        net2.connect("d", "term")
        dp2 = solve(net2)

        # Higher flowrate → higher pressure drop (dp ∝ v^2)
        assert dp2 > dp1

    def test_larger_duct_lower_pressure_drop(self) -> None:
        # Small duct
        net1 = Network("small")
        net1.add("ahu", Source("ahu"))
        net1.add("d", RigidDuct("d", Round(0.15), length=10))
        net1.add("term", Terminal("term", flowrate=0.1))
        net1.connect("ahu", "d")
        net1.connect("d", "term")
        dp1 = solve(net1)

        # Larger duct
        net2 = Network("large")
        net2.add("ahu", Source("ahu"))
        net2.add("d", RigidDuct("d", Round(0.3), length=10))
        net2.add("term", Terminal("term", flowrate=0.1))
        net2.connect("ahu", "d")
        net2.connect("d", "term")
        dp2 = solve(net2)

        # Larger duct → lower pressure drop
        assert dp2 < dp1
