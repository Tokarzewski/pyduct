"""Pure-function solver for a :class:`Network`.

The solver does three things:

1. :func:`propagate_flowrates` walks the graph in reverse topological order,
   summing the demanded flow at every :class:`Terminal` upstream toward the
   :class:`Source`. Every port ends up with a flowrate.
2. :func:`compute_pressure_drops` calls each component's ``compute()`` once a
   :class:`Fluid` is supplied, and copies the resulting per-port pressure drop
   onto the corresponding graph node.
3. :func:`critical_path` returns the longest path through the network weighted
   by per-port pressure drop, i.e. the worst-case static pressure required at
   the source.

:func:`solve` is a convenience that runs all three.
"""

from __future__ import annotations

import networkx as nx

from ..components.base import Port
from ..components.fitting import Terminal
from ..core.fluid import STANDARD_AIR, Fluid
from .network import Network, port_node_id


def propagate_flowrates(network: Network) -> None:
    """Walk the graph and assign a flowrate to every port.

    Terminal demands are propagated upstream so each duct/fitting/source sees
    the total volumetric flow it must carry.
    """
    G = network.graph

    # Reset all node flowrates.
    for node in G.nodes:
        G.nodes[node]["flowrate"] = 0.0

    # Seed terminal demands onto their in-port nodes.
    for cid, comp in network.components.items():
        if isinstance(comp, Terminal):
            (port,) = comp.ports
            G.nodes[port_node_id(cid, port.name)]["flowrate"] = comp.flowrate

    # Reverse topological order: downstream nodes first.
    #
    # Edge directions:
    #   in-port    -> component
    #   component  -> out-port
    #   out-port   -> downstream in-port (connection)
    #
    # In reverse, we visit a downstream in-port BEFORE its upstream out-port,
    # an out-port BEFORE its owning component, and a component BEFORE its
    # in-ports. That's exactly the order we need to push flowrates upstream.
    for node in reversed(list(nx.topological_sort(G))):
        attrs = G.nodes[node]
        flow = attrs["flowrate"]

        if attrs["kind"] == "port":
            port_obj: Port = attrs["port"]
            if port_obj.direction == "in":
                # Push flow back across the connection edge to the upstream
                # out-port (if any).
                for pred in G.predecessors(node):
                    if G.nodes[pred]["kind"] == "port":
                        G.nodes[pred]["flowrate"] += flow
            else:  # out-port
                # Push flow to the owning component.
                for pred in G.predecessors(node):
                    if G.nodes[pred]["kind"] == "component":
                        G.nodes[pred]["flowrate"] += flow
        else:  # component
            # Distribute the component's accumulated flow to its in-ports.
            in_port_nodes = [
                pred
                for pred in G.predecessors(node)
                if G.nodes[pred]["kind"] == "port"
            ]
            if len(in_port_nodes) == 1:
                G.nodes[in_port_nodes[0]]["flowrate"] += flow
            # 0 in-ports → Source: nothing to do.
            # >1 in-ports would need a split rule; not supported yet.

    # Copy graph flowrates back onto the Port objects so component.compute()
    # can use them directly.
    for cid, comp in network.components.items():
        for p in comp.ports:
            p.flowrate = G.nodes[port_node_id(cid, p.name)]["flowrate"]


def compute_pressure_drops(
    network: Network, fluid: Fluid = STANDARD_AIR
) -> None:
    """Call ``compute()`` on every component and copy results to graph nodes.

    The per-node pressure drops are also encoded as edge weights so that
    :func:`networkx.dag_longest_path` can be used directly: each edge gets a
    weight equal to the pressure drop of its target node.
    """
    G = network.graph
    for cid, comp in network.components.items():
        comp.compute(fluid)
        for p in comp.ports:
            G.nodes[port_node_id(cid, p.name)]["pressure_drop"] = p.pressure_drop
    # Component nodes carry no pressure drop themselves.
    for cid in network.components:
        G.nodes[cid].setdefault("pressure_drop", 0.0)
    # Encode node weights onto edges (target-side) for dag_longest_path.
    for u, v in G.edges:
        G[u][v]["pressure_drop"] = G.nodes[v].get("pressure_drop", 0.0)


def critical_path(network: Network) -> list[str]:
    """Return the list of graph node ids on the critical path.

    The critical path is the longest path (by total ``pressure_drop``) from any
    :class:`Source` outlet to any :class:`Terminal` inlet.
    """
    return nx.dag_longest_path(
        network.graph, weight="pressure_drop", default_weight=0
    )


def critical_path_pressure_drop(network: Network) -> float:
    """Return the total pressure drop along the critical path [Pa]."""
    G = network.graph
    return sum(G.nodes[n].get("pressure_drop", 0.0) for n in critical_path(network))


def solve(network: Network, fluid: Fluid = STANDARD_AIR) -> float:
    """Run the full solver pipeline and return the critical-path pressure drop."""
    propagate_flowrates(network)
    compute_pressure_drops(network, fluid)
    return critical_path_pressure_drop(network)
