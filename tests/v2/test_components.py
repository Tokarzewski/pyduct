"""Tests for individual components in isolation."""

from math import pi

import pytest

from pyduct2 import (
    STANDARD_AIR,
    ElbowRound,
    FlexDuct,
    RigidDuct,
    Round,
    Source,
    Tee,
    Terminal,
    TwoPortFitting,
)
from pyduct2.components.base import Port


def test_port_objects_are_unique_per_instance() -> None:
    """Regression: in the old design, every RigidDuct shared the same port list."""
    section = Round(diameter=0.2)
    d1 = RigidDuct("d1", section, length=5)
    d2 = RigidDuct("d2", section, length=5)
    assert d1.ports is not d2.ports
    assert d1.ports[0] is not d2.ports[0]


def test_source_has_one_outlet_port() -> None:
    s = Source("ahu")
    assert len(s.ports) == 1
    assert s.ports[0].direction == "out"


def test_terminal_seeds_demand_on_inlet_port() -> None:
    t = Terminal("t1", flowrate=0.05)
    assert t.ports[0].direction == "in"
    assert t.ports[0].flowrate == 0.05


def test_terminal_negative_flowrate_rejected() -> None:
    with pytest.raises(ValueError):
        Terminal("t", flowrate=-1.0)


class TestRigidDuctCompute:
    def test_pressure_drop_positive(self) -> None:
        d = RigidDuct("d", Round(0.2), length=10.0)
        d.ports[0].flowrate = 0.2  # m^3/s
        d.compute(STANDARD_AIR)
        assert d.ports[0].pressure_drop > 0
        assert d.ports[1].pressure_drop == 0
        # outlet flowrate is forwarded
        assert d.ports[1].flowrate == pytest.approx(0.2)

    def test_velocity(self) -> None:
        section = Round(0.2)
        d = RigidDuct("d", section, length=1.0)
        d.ports[0].flowrate = 0.1
        d.compute(STANDARD_AIR)
        v = 0.1 / section.area
        assert d.ports[0].velocity == pytest.approx(v)

    def test_unset_flowrate_raises(self) -> None:
        d = RigidDuct("d", Round(0.2), length=1.0)
        with pytest.raises(ValueError):
            d.compute(STANDARD_AIR)

    def test_zero_length_rejected(self) -> None:
        with pytest.raises(ValueError):
            RigidDuct("d", Round(0.2), length=0.0)


class TestFlexDuct:
    def test_compute(self) -> None:
        d = FlexDuct(
            "f",
            diameter=0.2,
            length=5.0,
            pressure_drop_per_meter=2.0,
        )
        d.ports[0].flowrate = 0.1
        d.compute(STANDARD_AIR)
        # fully stretched -> beta = 1 -> dp = 2 * 5 * 1 = 10
        assert d.ports[0].pressure_drop == pytest.approx(10.0)


class TestTwoPortFitting:
    def test_compute(self) -> None:
        f = TwoPortFitting("f", Round(0.2), zeta=0.5)
        f.ports[0].flowrate = 0.1
        f.compute(STANDARD_AIR)
        # outlet carries the loss; inlet is zero
        assert f.ports[1].pressure_drop > 0
        assert f.ports[0].pressure_drop == 0


class TestTee:
    def test_combined_flow_is_sum_of_legs(self) -> None:
        tee = Tee("t", Round(0.3))
        tee.port("straight").flowrate = 0.05
        tee.port("branch").flowrate = 0.07
        tee.compute(STANDARD_AIR)
        assert tee.port("combined").flowrate == pytest.approx(0.12)


class TestElbowRound:
    def test_zeta_in_range(self) -> None:
        e = ElbowRound(bend_radius=1.0, diameter=1.0, angle=90)
        assert 0.1 < e.zeta < 0.4

    def test_out_of_range_rejected(self) -> None:
        with pytest.raises(ValueError):
            ElbowRound(bend_radius=10.0, diameter=1.0, angle=90)
        with pytest.raises(ValueError):
            ElbowRound(bend_radius=1.0, diameter=1.0, angle=10)
