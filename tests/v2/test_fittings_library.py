"""Tests for the fittings loss coefficient library."""

import pytest

from pyduct2.components.fittings_library import (
    damper_butterfly,
    diffuser_ceiling,
    expander_round,
    grille_return,
    junction_tee_branch,
    junction_tee_combine,
    reducer_round,
)


class TestReducerRound:
    def test_basic(self) -> None:
        zeta = reducer_round(0.3, 0.2)
        assert 0 < zeta < 1

    def test_zero_area_ratio_gives_high_loss(self) -> None:
        # Very small outlet → high loss
        zeta_small = reducer_round(0.3, 0.01)
        zeta_large = reducer_round(0.3, 0.2)
        assert zeta_small > zeta_large

    def test_outlet_larger_than_inlet_rejected(self) -> None:
        with pytest.raises(ValueError):
            reducer_round(0.2, 0.3)


class TestExpanderRound:
    def test_basic(self) -> None:
        zeta = expander_round(0.2, 0.3)
        assert zeta > 0

    def test_inlet_larger_than_outlet_rejected(self) -> None:
        with pytest.raises(ValueError):
            expander_round(0.3, 0.2)

    def test_angle_affects_loss(self) -> None:
        zeta_small_angle = expander_round(0.2, 0.3, angle_deg=10)
        zeta_large_angle = expander_round(0.2, 0.3, angle_deg=45)
        # Larger angle (more abrupt) → higher loss
        assert zeta_large_angle > zeta_small_angle


class TestJunctionTeeBranch:
    def test_basic(self) -> None:
        z_main, z_branch = junction_tee_branch(0.2, 0.15, 0.05, 0.05)
        assert z_main > 0
        assert z_branch > 0
        # Typically branch loss > main loss
        assert z_branch > z_main

    def test_all_flow_straight_gives_low_loss(self) -> None:
        z_main, z_branch = junction_tee_branch(0.2, 0.15, 0.1, 0.0)
        # All flow straight → minimal main loss
        assert z_main < 0.15

    def test_negative_flowrate_rejected(self) -> None:
        with pytest.raises(ValueError):
            junction_tee_branch(0.2, 0.15, -0.05, 0.05)


class TestJunctionTeeCombine:
    def test_basic(self) -> None:
        z_main, z_branch = junction_tee_combine(0.2, 0.15, 0.05, 0.05)
        assert z_main > 0
        assert z_branch > 0

    def test_combining_has_higher_loss_than_splitting(self) -> None:
        # Same geometry and flow split
        z_split_main, z_split_branch = junction_tee_branch(0.2, 0.15, 0.05, 0.05)
        z_comb_main, z_comb_branch = junction_tee_combine(0.2, 0.15, 0.05, 0.05)
        # Combining should have higher losses
        assert z_comb_main > z_split_main
        assert z_comb_branch > z_split_branch


class TestDamperButterfly:
    def test_fully_open_low_loss(self) -> None:
        zeta = damper_butterfly(100)
        assert zeta < 0.2

    def test_partially_open_higher_loss(self) -> None:
        zeta_100 = damper_butterfly(100)
        zeta_50 = damper_butterfly(50)
        zeta_10 = damper_butterfly(10)
        assert zeta_100 < zeta_50 < zeta_10

    def test_fully_closed_high_loss(self) -> None:
        zeta = damper_butterfly(0)
        assert zeta > 5

    def test_invalid_percentage_rejected(self) -> None:
        with pytest.raises(ValueError):
            damper_butterfly(-10)
        with pytest.raises(ValueError):
            damper_butterfly(150)


class TestDiffuserCeiling:
    def test_basic(self) -> None:
        zeta = diffuser_ceiling(1.0)
        assert 0.2 < zeta < 0.8

    def test_higher_throw_lower_loss(self) -> None:
        zeta_low = diffuser_ceiling(0.5)
        zeta_high = diffuser_ceiling(1.5)
        # Higher throw → lower loss
        assert zeta_low > zeta_high

    def test_zero_area_throw_rejected(self) -> None:
        with pytest.raises(ValueError):
            diffuser_ceiling(0.0)


class TestGrilleReturn:
    def test_basic(self) -> None:
        zeta = grille_return(0.15)
        assert 0.2 < zeta < 0.6

    def test_higher_blockage_higher_loss(self) -> None:
        zeta_low = grille_return(0.05)
        zeta_high = grille_return(0.25)
        assert zeta_low < zeta_high

    def test_blockage_bounds(self) -> None:
        with pytest.raises(ValueError):
            grille_return(-0.1)
        with pytest.raises(ValueError):
            grille_return(1.5)
