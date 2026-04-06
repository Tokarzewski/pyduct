"""Tests for the standard-sizes data module."""

import pytest

from pyduct2 import STANDARD_ROUND_DUCT_SIZES, nearest_round_size


def test_round_sizes_sorted() -> None:
    sizes = STANDARD_ROUND_DUCT_SIZES
    assert list(sizes) == sorted(sizes)


def test_nearest_round_size_round_up() -> None:
    assert nearest_round_size(140) == 150
    assert nearest_round_size(150) == 150
    assert nearest_round_size(151) == 160


def test_nearest_round_size_closest() -> None:
    # 140 is closer to 150 (delta 10) than 125 (delta 15)
    assert nearest_round_size(140, round_up=False) == 150
    # 130 is closer to 125 (delta 5) than 150 (delta 20)
    assert nearest_round_size(130, round_up=False) == 125


def test_clamps_to_extremes() -> None:
    assert nearest_round_size(1) == STANDARD_ROUND_DUCT_SIZES[0]
    assert nearest_round_size(99999) == STANDARD_ROUND_DUCT_SIZES[-1]
