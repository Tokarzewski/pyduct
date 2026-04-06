"""Tests for results extraction and formatting."""

import pytest

from pyduct2 import (
    Network,
    RigidDuct,
    Round,
    Source,
    Terminal,
    extract_results,
    results_as_csv,
    results_as_dicts,
    results_summary,
    solve,
)


@pytest.fixture
def simple_solved_network() -> Network:
    """A simple solved network for testing results."""
    net = Network("test")
    section = Round(0.2)
    net.add("ahu", Source("ahu"))
    net.add("d1", RigidDuct("d1", section, length=10))
    net.add("term", Terminal("term", flowrate=0.1))
    net.connect("ahu", "d1")
    net.connect("d1", "term")
    solve(net)
    return net


class TestExtractResults:
    def test_one_result_per_component(self, simple_solved_network: Network) -> None:
        results = extract_results(simple_solved_network)
        assert len(results) == 3  # ahu, d1, term

    def test_component_data_populated(self, simple_solved_network: Network) -> None:
        results = extract_results(simple_solved_network)
        # Find the duct result
        duct_result = [r for r in results if r.component_type == "RigidDuct"][0]
        assert duct_result.flowrate_in == pytest.approx(0.1)
        assert duct_result.velocity_in is not None and duct_result.velocity_in > 0
        assert duct_result.pressure_drop > 0

    def test_terminal_has_zero_pressure_drop(self, simple_solved_network: Network) -> None:
        results = extract_results(simple_solved_network)
        term_result = [r for r in results if r.component_type == "Terminal"][0]
        # Terminal device itself has no loss (unless zeta is specified)
        assert term_result.pressure_drop == 0


class TestResultsSummary:
    def test_summary_is_string(self, simple_solved_network: Network) -> None:
        summary = results_summary(simple_solved_network)
        assert isinstance(summary, str)
        assert "Component" in summary or "duct" in summary.lower()

    def test_summary_contains_pressures(self, simple_solved_network: Network) -> None:
        summary = results_summary(simple_solved_network)
        assert "Pa" in summary

    def test_empty_network_summary(self) -> None:
        net = Network("empty")
        summary = results_summary(net)
        assert "(no components)" in summary


class TestResultsAsDicts:
    def test_returns_list_of_dicts(self, simple_solved_network: Network) -> None:
        dicts = results_as_dicts(simple_solved_network)
        assert isinstance(dicts, list)
        assert all(isinstance(d, dict) for d in dicts)

    def test_dict_has_expected_keys(self, simple_solved_network: Network) -> None:
        dicts = results_as_dicts(simple_solved_network)
        assert len(dicts) > 0
        keys = dicts[0].keys()
        assert "component_id" in keys
        assert "pressure_drop" in keys


class TestResultsAsCSV:
    def test_returns_string(self, simple_solved_network: Network) -> None:
        csv = results_as_csv(simple_solved_network)
        assert isinstance(csv, str)

    def test_csv_has_header_and_rows(self, simple_solved_network: Network) -> None:
        csv = results_as_csv(simple_solved_network)
        lines = csv.split("\n")
        assert len(lines) >= 2  # header + at least one row
        # Header should have commas
        assert "," in lines[0]

    def test_custom_delimiter(self, simple_solved_network: Network) -> None:
        csv = results_as_csv(simple_solved_network, delimiter=";")
        # Header should use semicolons
        assert ";" in csv.split("\n")[0]

    def test_empty_network_csv(self) -> None:
        net = Network("empty")
        csv = results_as_csv(net)
        assert csv == ""
