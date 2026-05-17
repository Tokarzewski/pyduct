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

:func:`solve` is a convenience that runs all three. All three reuse the
``Network``'s cached topological order, so repeated solves on an unchanged
network skip the topo sort entirely.
"""

from __future__ import annotations

from ..components.base import Port
from ..components.fitting import Terminal
from ..core.fluid import STANDARD_AIR, Fluid
from .network import Network, port_node_id


def propagate_flowrates(network: Network) -> None:
    """Walk the graph and assign a flowrate to every port.

    Terminal demands are propagated upstream so each duct/fitting/source sees
    the total volumetric flow it must carry.
    """
    # Direct dict access into NetworkX's internal node store skips the
    # NodeView wrapper, which dominates the profile for tight solver loops.
    nodes = network.graph._node
    preds = network.predecessors_map()

    # Reset all node flowrates.
    for attrs in nodes.values():
        attrs["flowrate"] = 0.0

    # Seed terminal demands onto their in-port nodes.
    for cid, comp in network.components.items():
        if isinstance(comp, Terminal):
            (port,) = comp.ports
            nodes[port_node_id(cid, port.name)]["flowrate"] = comp.flowrate

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
    for node in reversed(network.topo_order()):
        attrs = nodes[node]
        flow = attrs["flowrate"]

        if attrs["kind"] == "port":
            port_obj: Port = attrs["port"]
            if port_obj.direction == "in":
                # Push flow back across the connection edge to the upstream
                # out-port (if any).
                for pred in preds[node]:
                    pred_attrs = nodes[pred]
                    if pred_attrs["kind"] == "port":
                        pred_attrs["flowrate"] += flow
            else:  # out-port
                # Push flow to the owning component.
                for pred in preds[node]:
                    pred_attrs = nodes[pred]
                    if pred_attrs["kind"] == "component":
                        pred_attrs["flowrate"] += flow
        else:  # component
            # Distribute the component's accumulated flow to its in-ports.
            in_port_nodes = [p for p in preds[node] if nodes[p]["kind"] == "port"]
            if len(in_port_nodes) == 1:
                nodes[in_port_nodes[0]]["flowrate"] += flow
            # 0 in-ports → Source: nothing to do.
            # >1 in-ports would need a split rule; not supported yet.

    # Copy graph flowrates back onto the Port objects so component.compute()
    # can use them directly.
    for cid, comp in network.components.items():
        for p in comp.ports:
            p.flowrate = nodes[port_node_id(cid, p.name)]["flowrate"]


def compute_pressure_drops(
    network: Network, fluid: Fluid = STANDARD_AIR
) -> None:
    """Call ``compute()`` on every component and copy results to graph nodes."""
    nodes = network.graph._node
    for cid, comp in network.components.items():
        comp.compute(fluid)
        for p in comp.ports:
            nodes[port_node_id(cid, p.name)]["pressure_drop"] = p.pressure_drop
    # Component nodes carry no pressure drop themselves.
    for cid in network.components:
        nodes[cid].setdefault("pressure_drop", 0.0)


def critical_path(network: Network) -> list[str]:
    """Return the list of graph node ids on the critical path.

    The critical path is the longest path (by total node ``pressure_drop``)
    from any :class:`Source` to any :class:`Terminal`. Implemented as a
    single-pass DP over the cached topological order — O(V + E), no NetworkX
    longest-path call.
    """
    nodes = network.graph._node
    preds_map = network.predecessors_map()
    dist: dict[str, float] = {}
    prev: dict[str, str | None] = {}
    for n in network.topo_order():
        preds = preds_map[n]
        if preds:
            best_p = max(preds, key=dist.__getitem__)
            best_d = dist[best_p]
            prev[n] = best_p
        else:
            best_d = 0.0
            prev[n] = None
        dist[n] = best_d + nodes[n].get("pressure_drop", 0.0)
    if not dist:
        return []
    end = max(dist, key=dist.__getitem__)
    path: list[str] = []
    cur: str | None = end
    while cur is not None:
        path.append(cur)
        cur = prev[cur]
    path.reverse()
    return path


def critical_path_pressure_drop(network: Network) -> float:
    """Return the total pressure drop along the critical path [Pa]."""
    nodes = network.graph._node
    return sum(nodes[n].get("pressure_drop", 0.0) for n in critical_path(network))


def solve(network: Network, fluid: Fluid = STANDARD_AIR) -> float:
    """Run the full solver pipeline and return the critical-path pressure drop."""
    propagate_flowrates(network)
    compute_pressure_drops(network, fluid)
    return critical_path_pressure_drop(network)
