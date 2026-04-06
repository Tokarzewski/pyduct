"""Visualize a ductwork network graph with critical path highlighted.

Requires matplotlib and networkx. The network is drawn as a directed graph
with component nodes in one colour, critical path in another, and edge labels
showing component names and pressures.
"""

from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    import matplotlib.pyplot as plt

from .network.network import Network
from .network.solver import critical_path


def draw_network(
    network: Network,
    figsize: tuple[float, float] = (14, 10),
    seed: int = 42,
    show: bool = True,
) -> tuple[plt.Figure, plt.Axes]:  # type: ignore[name-defined]
    """Draw the network graph with the critical path highlighted.

    Parameters
    ----------
    network:
        A solved Network (i.e., after calling :func:`solve`).
    figsize:
        Figure size (width, height) in inches.
    seed:
        Random seed for spring layout.
    show:
        If True, call ``plt.show()`` before returning.

    Returns
    -------
    (fig, ax):
        Matplotlib Figure and Axes. Caller can save/modify before showing.

    Requires
    --------
    matplotlib, networkx (imported at call time, not import time).
    """
    import matplotlib.pyplot as plt
    import networkx as nx

    G = network.graph
    path = critical_path(network)

    # Layout
    pos = nx.spring_layout(G, seed=seed, k=2, iterations=50)

    fig, ax = plt.subplots(figsize=figsize)

    # Separate nodes by type and critical path membership.
    component_nodes = [n for n in G.nodes if G.nodes[n]["kind"] == "component"]
    port_nodes = [n for n in G.nodes if G.nodes[n]["kind"] == "port"]
    crit_nodes = [n for n in path if n in G.nodes]
    non_crit_nodes = [n for n in G.nodes if n not in crit_nodes]

    # Draw edges
    nx.draw_networkx_edges(
        G, pos, ax=ax, edge_color="lightgray", arrows=True,
        arrowsize=15, arrowstyle="-|>", width=1.5
    )

    # Draw non-critical nodes
    nx.draw_networkx_nodes(
        G, pos,
        nodelist=non_crit_nodes,
        node_color="lightblue",
        node_size=500,
        ax=ax,
        label="Non-critical",
    )

    # Draw critical-path nodes in orange
    nx.draw_networkx_nodes(
        G, pos,
        nodelist=crit_nodes,
        node_color="orange",
        node_size=700,
        ax=ax,
        label="Critical path",
    )

    # Labels: show short IDs (component_id or port_id) and pressure drops.
    labels = {}
    edge_labels = {}
    for n in G.nodes:
        attrs = G.nodes[n]
        if attrs["kind"] == "component":
            comp = attrs["component"]
            labels[n] = f"{n}\n({comp.name})"
        else:
            # Port node: show ID and pressure drop
            port = attrs["port"]
            dp = port.pressure_drop
            labels[n] = f"{n}\nΔP={dp:.1f}Pa"

    nx.draw_networkx_labels(G, pos, labels, font_size=8, ax=ax)

    ax.set_title(f"Ductwork Network — Critical Path DP: {sum(G.nodes[n]['pressure_drop'] for n in crit_nodes):.1f} Pa")
    ax.legend(loc="upper left")
    ax.axis("off")

    if show:
        plt.show()

    return fig, ax


def summary_text(network: Network) -> str:
    """Return a text summary of the network and critical path."""
    from .results import extract_results
    from .network.solver import critical_path, critical_path_pressure_drop

    results = extract_results(network)
    path = critical_path(network)
    total_dp = critical_path_pressure_drop(network)

    lines = [
        f"Network: {network.name}",
        f"Components: {len(network.components)}",
        f"Critical path length: {len(path)} nodes",
        f"Critical path pressure drop: {total_dp:.2f} Pa",
        "",
        "Component Summary:",
    ]
    for res in results:
        q = f"Q={res.flowrate_in:.3f}" if res.flowrate_in is not None else "Q=—"
        v = f"V={res.velocity_in:.2f}" if res.velocity_in is not None else "V=—"
        lines.append(f"  {res.component_id:<12} {res.component_type:<15} {q:<15} {v:<15} ΔP={res.pressure_drop:>7.2f}Pa")

    return "\n".join(lines)
