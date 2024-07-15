import networkx as nx
from .small_example_network import sup1, G
import pytest

def test_critical_path():
    # Test critical path nodes
    nodes_on_critical_path = sup1.critical_path_nodes()
    assert isinstance(nodes_on_critical_path, list), "Critical path nodes should be a list"
    assert len(nodes_on_critical_path) > 0, "Critical path should not be empty"
    assert all(node in G.nodes() for node in nodes_on_critical_path), "All nodes in critical path should be in the graph"

    # Test critical path pressure drop
    critical_path_dp = sup1.critical_path_pressure_drop()
    assert isinstance(critical_path_dp, float), "Critical path pressure drop should be a float"
    assert pytest.approx(65.07, abs=0.01) == critical_path_dp, "Critical path pressure drop should be close to 65.07"

    # Test pressure drop attributes
    dp_labels = nx.get_node_attributes(G, name="pressure_drop")
    assert len(dp_labels) == len(G.nodes()), "All nodes should have a pressure_drop attribute"
    assert all(isinstance(value, (int, float)) for value in dp_labels.values()), "Pressure drop values should be numeric"
