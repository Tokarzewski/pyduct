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

from ..core.fluid import STANDARD_AIR, Fluid
from .network import Network


def propagate_flowrates(network: Network) -> None:
    """Walk the graph and assign a flowrate to every port.

    Terminal demands are propagated upstream so each duct/fitting/source sees
    the total volumetric flow it must carry.

    Graph invariants let us push flow to *every* predecessor without filtering:
      * in-port preds are always out-ports (upstream connection)
      * out-port preds are always the owning component
      * component preds are always the in-ports (≤ 1 for every component
        supported today — Source has 0, everything else has exactly 1)
    """
    nodes = network.graph._node
    preds = network.predecessors_map()

    # Reset all node flowrates.
    for attrs in nodes.values():
        attrs["flowrate"] = 0.0

    # Seed terminal demands onto their in-port nodes. The terminal list is
    # cached on the Network so we don't isinstance-scan every solve.
    for term in network.terminals():
        nodes[term.ports[0].node_id]["flowrate"] = term.flowrate

    # Walk downstream-first; each node forwards its accumulated flow to every
    # predecessor (which is the correct upstream node by construction).
    for node in reversed(network.topo_order()):
        flow = nodes[node]["flowrate"]
        if flow:
            for pred in preds[node]:
                nodes[pred]["flowrate"] += flow

    # Copy graph flowrates back onto the Port objects so component.compute()
    # can use them directly.
    for comp in network.components.values():
        for p in comp.ports:
            p.flowrate = nodes[p.node_id]["flowrate"]


def compute_pressure_drops(
    network: Network, fluid: Fluid = STANDARD_AIR
) -> None:
    """Call ``compute()`` on every component and copy results to graph nodes."""
    nodes = network.graph._node
    for comp in network.components.values():
        comp.compute(fluid)
        for p in comp.ports:
            nodes[p.node_id]["pressure_drop"] = p.pressure_drop
    # Component nodes' pressure_drop is initialised to 0.0 in Network.add(),
    # so no setdefault is needed here.


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
        if not preds:
            best_p, best_d = None, 0.0
        elif len(preds) == 1:
            # Hot case: ports and most internal edges have exactly one
            # predecessor — skip the max() call.
            best_p = preds[0]
            best_d = dist[best_p]
        else:
            best_p = max(preds, key=dist.__getitem__)
            best_d = dist[best_p]
        prev[n] = best_p
        dist[n] = best_d + nodes[n]["pressure_drop"]
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
    """Return the total pressure drop along the critical path [Pa].

    The DP runs in native Mojo via ``wenta.ext.solver_ext``. The Python
    side projects the network's NetworkX graph into three flat lists once
    (topo / preds / dp) and crosses the Mojo boundary exactly once per
    solve, so the boundary cost is amortised over the whole walk.
    """
    from wenta.ext.solver_ext import critical_path_sum

    _, int_topo, int_preds = network.int_topo_view()
    nodes = network.graph._node
    topo_str = network.topo_order()
    dp = [nodes[n]["pressure_drop"] for n in topo_str]
    return critical_path_sum(int_topo, int_preds, dp)


def solve(network: Network, fluid: Fluid = STANDARD_AIR) -> float:
    """Run the full solver pipeline and return the critical-path pressure drop."""
    propagate_flowrates(network)
    compute_pressure_drops(network, fluid)
    return critical_path_pressure_drop(network)
