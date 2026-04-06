"""End-to-end tests for the Network model and solver."""

import pytest

from pyduct2 import (
    Network,
    RigidDuct,
    Round,
    Source,
    Tee,
    Terminal,
    TwoPortFitting,
    critical_path,
    critical_path_pressure_drop,
    propagate_flowrates,
    solve,
)


def _simple_linear_net() -> Network:
    section = Round(0.2)
    net = Network("simple")
    net.add("ahu", Source("ahu"))
    net.add("d1", RigidDuct("d1", section, length=10))
    net.add("term", Terminal("t", flowrate=0.1))
    net.connect("ahu", "d1")
    net.connect("d1", "term")
    return net


def _two_branch_net() -> Network:
    section = Round(1.0)
    net = Network("two-branch")
    net.add("ahu", Source("ahu"))
    net.add("d1", RigidDuct("d1", section, length=10))
    net.add("tee", Tee("tee", section))
    net.add("term1", Terminal("t1", flowrate=5.0))
    net.add("term2", Terminal("t2", flowrate=7.0))
    net.connect("ahu", "d1")
    net.connect("d1", "tee.combined")
    net.connect("tee.straight", "term1")
    net.connect("tee.branch", "term2")
    return net


class TestBuilding:
    def test_duplicate_id_rejected(self) -> None:
        net = _simple_linear_net()
        with pytest.raises(ValueError):
            net.add("d1", RigidDuct("again", Round(0.1), length=1))

    def test_unknown_component_rejected(self) -> None:
        net = _simple_linear_net()
        with pytest.raises(KeyError):
            net.connect("nope", "d1")

    def test_tee_requires_explicit_port_when_ambiguous(self) -> None:
        net = Network()
        net.add("ahu", Source("ahu"))
        net.add("tee", Tee("tee", Round(0.5)))
        net.add("t1", Terminal("t1", flowrate=1))
        net.connect("ahu", "tee.combined")
        # Tee has 2 out-ports → must disambiguate
        with pytest.raises(ValueError):
            net.connect("tee", "t1")


class TestPropagation:
    def test_simple_linear(self) -> None:
        net = _simple_linear_net()
        propagate_flowrates(net)
        assert net.components["d1"].port("inlet").flowrate == pytest.approx(0.1)
        assert net.components["d1"].port("outlet").flowrate == pytest.approx(0.1)
        assert net.components["ahu"].port("outlet").flowrate == pytest.approx(0.1)

    def test_branching(self) -> None:
        net = _two_branch_net()
        propagate_flowrates(net)
        tee = net.components["tee"]
        assert tee.port("straight").flowrate == pytest.approx(5.0)
        assert tee.port("branch").flowrate == pytest.approx(7.0)
        assert tee.port("combined").flowrate == pytest.approx(12.0)
        assert net.components["d1"].port("inlet").flowrate == pytest.approx(12.0)
        assert net.components["ahu"].port("outlet").flowrate == pytest.approx(12.0)


class TestSolve:
    def test_simple_returns_positive_dp(self) -> None:
        net = _simple_linear_net()
        dp = solve(net)
        assert dp > 0

    def test_critical_path_starts_at_source(self) -> None:
        net = _two_branch_net()
        solve(net)
        path = critical_path(net)
        assert path[0] == "ahu:outlet" or path[0] == "ahu"

    def test_critical_path_dp_is_consistent(self) -> None:
        net = _two_branch_net()
        dp = solve(net)
        # solver result must equal recomputation from the path
        assert dp == pytest.approx(critical_path_pressure_drop(net))

    def test_higher_loss_branch_dominates_critical_path(self) -> None:
        # The branch leg has zeta_branch=0.5 (vs 0 on straight), so the
        # critical path must traverse the branch leg of the tee.
        net = _two_branch_net()
        solve(net)
        path = critical_path(net)
        assert "tee:branch" in path
        assert "tee:straight" not in path
