"""Tests for YAML/JSON network I/O."""

import json
import tempfile
from pathlib import Path

import pytest

from pyduct2 import (
    Network,
    RigidDuct,
    Round,
    Source,
    Terminal,
    load_from_json,
    load_network_from_dict,
    save_network_to_dict,
    save_to_json,
)


@pytest.fixture
def simple_network() -> Network:
    """A simple network for I/O testing."""
    net = Network("io_test")
    section = Round(0.2)
    net.add("ahu", Source("ahu"))
    net.add("duct", RigidDuct("duct", section, length=10))
    net.add("term", Terminal("term", flowrate=0.1))
    net.connect("ahu", "duct")
    net.connect("duct", "term")
    return net


class TestNetworkToDict:
    def test_converts_to_dict(self, simple_network: Network) -> None:
        data = save_network_to_dict(simple_network)
        assert isinstance(data, dict)
        assert "name" in data
        assert "components" in data
        assert "connections" in data

    def test_dict_has_all_components(self, simple_network: Network) -> None:
        data = save_network_to_dict(simple_network)
        assert len(data["components"]) == 3
        assert "ahu" in data["components"]
        assert "duct" in data["components"]
        assert "term" in data["components"]

    def test_component_types_preserved(self, simple_network: Network) -> None:
        data = save_network_to_dict(simple_network)
        assert data["components"]["ahu"]["type"] == "Source"
        assert data["components"]["duct"]["type"] == "RigidDuct"
        assert data["components"]["term"]["type"] == "Terminal"


class TestNetworkFromDict:
    def test_loads_from_dict(self, simple_network: Network) -> None:
        data = save_network_to_dict(simple_network)
        net2 = load_network_from_dict(data)
        assert net2.name == simple_network.name
        assert len(net2.components) == len(simple_network.components)

    def test_roundtrip_preserves_structure(self, simple_network: Network) -> None:
        data = save_network_to_dict(simple_network)
        net2 = load_network_from_dict(data)
        data2 = save_network_to_dict(net2)
        # Components and connections should match
        assert len(data2["components"]) == len(data["components"])
        for cid in data["components"]:
            assert cid in data2["components"]
            comp1 = data["components"][cid]
            comp2 = data2["components"][cid]
            assert comp1["type"] == comp2["type"]
            assert comp1["name"] == comp2["name"]


class TestJSONSerialization:
    def test_save_to_json(self, simple_network: Network) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            filepath = Path(tmpdir) / "network.json"
            save_to_json(simple_network, filepath)
            assert filepath.exists()

    def test_load_from_json(self, simple_network: Network) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            filepath = Path(tmpdir) / "network.json"
            save_to_json(simple_network, filepath)
            net2 = load_from_json(filepath)
            assert net2.name == simple_network.name
            assert len(net2.components) == len(simple_network.components)

    def test_json_is_valid(self, simple_network: Network) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            filepath = Path(tmpdir) / "network.json"
            save_to_json(simple_network, filepath)
            # Verify it's valid JSON by loading and parsing
            with open(filepath) as f:
                data = json.load(f)
            assert "name" in data
            assert "components" in data


class TestYAMLSerialization:
    def test_save_to_yaml(self, simple_network: Network) -> None:
        pytest.importorskip("yaml")  # Skip if pyyaml not installed
        from pyduct2 import save_to_yaml

        with tempfile.TemporaryDirectory() as tmpdir:
            filepath = Path(tmpdir) / "network.yaml"
            save_to_yaml(simple_network, filepath)
            assert filepath.exists()

    def test_load_from_yaml(self, simple_network: Network) -> None:
        pytest.importorskip("yaml")
        from pyduct2 import load_from_yaml, save_to_yaml

        with tempfile.TemporaryDirectory() as tmpdir:
            filepath = Path(tmpdir) / "network.yaml"
            save_to_yaml(simple_network, filepath)
            net2 = load_from_yaml(filepath)
            assert net2.name == simple_network.name
            assert len(net2.components) == len(simple_network.components)


class TestIOErrorHandling:
    def test_missing_file(self) -> None:
        with pytest.raises(FileNotFoundError):
            load_from_json("/nonexistent/path.json")

    def test_pyyaml_missing(self, simple_network: Network) -> None:
        """Test that helpful error is raised if pyyaml not installed."""
        import sys

        # Temporarily hide yaml
        yaml_backup = sys.modules.get("yaml")
        sys.modules["yaml"] = None  # type: ignore

        try:
            from pyduct2 import save_to_yaml

            with pytest.raises(ImportError, match="pyyaml"):
                with tempfile.TemporaryDirectory() as tmpdir:
                    filepath = Path(tmpdir) / "network.yaml"
                    save_to_yaml(simple_network, filepath)
        finally:
            # Restore yaml
            if yaml_backup:
                sys.modules["yaml"] = yaml_backup
            else:
                sys.modules.pop("yaml", None)
