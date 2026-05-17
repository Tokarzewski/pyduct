"""Tests for cross-section geometry."""

import dataclasses
from math import isclose, pi

import pytest

from pyduct import Rectangular, Round, equivalent_round_diameter


class TestRound:
    def test_area(self) -> None:
        s = Round(diameter=2.0)
        assert isclose(s.area, pi)

    def test_hydraulic_diameter_equals_diameter(self) -> None:
        s = Round(diameter=0.4)
        assert s.hydraulic_diameter == 0.4

    def test_zero_diameter_rejected(self) -> None:
        with pytest.raises(ValueError):
            Round(diameter=0.0)

    def test_negative_diameter_rejected(self) -> None:
        with pytest.raises(ValueError):
            Round(diameter=-1.0)

    def test_immutable(self) -> None:
        s = Round(diameter=0.5)
        with pytest.raises(dataclasses.FrozenInstanceError):
            s.diameter = 0.6  # type: ignore[misc]


class TestRectangular:
    def test_area(self) -> None:
        s = Rectangular(width=0.5, height=0.4)
        assert isclose(s.area, 0.2)

    def test_hydraulic_diameter_square(self) -> None:
        # For a square, D_h = side length.
        s = Rectangular(width=0.3, height=0.3)
        assert isclose(s.hydraulic_diameter, 0.3)

    def test_hydraulic_diameter_general(self) -> None:
        # 2 * 0.4 * 0.2 / (0.6) = 0.1333...
        s = Rectangular(width=0.4, height=0.2)
        assert isclose(s.hydraulic_diameter, 2 * 0.4 * 0.2 / 0.6)

    def test_zero_dimension_rejected(self) -> None:
        with pytest.raises(ValueError):
            Rectangular(width=0.0, height=0.3)
        with pytest.raises(ValueError):
            Rectangular(width=0.3, height=-0.1)


class TestEquivalentRoundDiameter:
    def test_square_known_value(self) -> None:
        # ASHRAE: D_eq = 1.30 · (a·b)^0.625 / (a+b)^0.25
        a = b = 0.3
        expected = 1.30 * (a * b) ** 0.625 / (a + b) ** 0.25
        assert isclose(equivalent_round_diameter(a, b), expected)

    def test_rejects_nonpositive(self) -> None:
        with pytest.raises(ValueError):
            equivalent_round_diameter(0.0, 0.3)
        with pytest.raises(ValueError):
            equivalent_round_diameter(0.3, -0.1)
