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


def _topological_layout(
    G, *, seed: int = 42, x_spacing: float = 1.5, y_spacing: float = 1.0
) -> dict:
    """Left-to-right layout: x = topological depth, y stacks parallel branches.

    Works well for pyduct DAGs because real airflow runs source → terminal,
    so the depth axis mirrors the physical layout. Disconnected nodes fall
    into depth 0.
    """
    import networkx as nx

    depth: dict[str, int] = {}
    for n in nx.topological_sort(G):
        preds = list(G.predecessors(n))
        depth[n] = 1 + max((depth[p] for p in preds), default=-1)

    layers: dict[int, list[str]] = {}
    for n, d in depth.items():
        layers.setdefault(d, []).append(n)

    # Order each layer to roughly match its predecessors' y to keep edges
    # short and avoid criss-crosses.
    pos: dict[str, tuple[float, float]] = {}
    for d in sorted(layers):
        nodes = layers[d]
        if d == 0:
            ordered = sorted(nodes)
        else:
            def _pred_y(n: str) -> float:
                ys = [pos[p][1] for p in G.predecessors(n) if p in pos]
                return sum(ys) / len(ys) if ys else 0.0
            ordered = sorted(nodes, key=_pred_y)
        n_layer = len(ordered)
        for i, n in enumerate(ordered):
            y = (i - (n_layer - 1) / 2) * y_spacing
            pos[n] = (d * x_spacing, y)
    return pos


def draw_network(
    network: Network,
    figsize: tuple[float, float] = (14, 10),
    seed: int = 42,
    show: bool = True,
) -> tuple[plt.Figure, plt.Axes]:
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

    # Separate nodes by critical path membership.
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


def plot_diagnostics(
    network: Network,
    figsize: tuple[float, float] = (20, 12),
    seed: int = 42,
    save_path: str | None = None,
    show: bool = False,
) -> tuple[plt.Figure, list[plt.Axes]]:
    """Render a 5-panel diagnostic figure of a solved :class:`Network`.

    Panels (left-to-right, top-to-bottom):

    1. **IDs** — graph node ids.
    2. **Name** — component / port names.
    3. **Critical path** — critical-path nodes green, others orange,
       edge labels show node pressure drops.
    4. **Flowrate [m³/s]** — edge labels show source-node flowrate.
    5. **Pressure drop [Pa]** — edge labels show target-node pressure drop.

    All five share the same spring layout for visual comparison. Run
    :func:`pyduct.solve` on the network first so flowrates and pressure
    drops are populated.
    """
    import matplotlib.pyplot as plt
    import networkx as nx

    from .network.solver import critical_path_pressure_drop

    G = network.graph
    # Seed Kamada-Kawai with a topological layout so airflow runs roughly
    # left-to-right; then nudge with spring iterations so long trunk runs
    # don't collapse onto each other.
    init_pos = _topological_layout(G)
    kk_pos = nx.kamada_kawai_layout(G, pos=init_pos)
    pos = nx.spring_layout(G, pos=kk_pos, iterations=60, k=0.45, seed=seed)
    crit_set = set(critical_path(network))

    fig, axes_grid = plt.subplots(2, 3, figsize=figsize)
    axes = list(axes_grid.flat)
    axes[5].axis("off")  # only 5 panels
    node_size = 220

    def _draw_base(ax: plt.Axes, title: str) -> None:
        ax.set_title(title, fontsize=11)
        ax.axis("off")
        ax.margins(0.12)
        nx.draw_networkx_edges(
            G, pos, ax=ax, edge_color="lightgray", arrows=True,
            arrowsize=10, arrowstyle="-|>", width=1.0, node_size=node_size,
        )

    def _node_label(n: str, mode: str) -> str:
        attrs = G.nodes[n]
        if mode == "id":
            return n
        if mode == "name":
            return attrs["component"].name if attrs["kind"] == "component" else attrs["port"].name
        return ""

    def _edge_label(u: str, v: str, mode: str) -> str:
        if mode == "flowrate":
            val = G.nodes[u].get("flowrate", 0.0)
        elif mode == "pressure" or mode == "critical_pressure":
            val = G.nodes[v].get("pressure_drop", 0.0)
        else:
            return ""
        return f"{val:.2f}" if val else "0"

    # Panel 1: IDs
    _draw_base(axes[0], "IDs")
    nx.draw_networkx_nodes(G, pos, ax=axes[0], node_color="tab:blue", node_size=node_size)
    nx.draw_networkx_labels(
        G, pos, {n: _node_label(n, "id") for n in G.nodes},
        font_size=6, ax=axes[0], font_color="white",
    )

    # Panel 2: Names
    _draw_base(axes[1], "Name")
    nx.draw_networkx_nodes(G, pos, ax=axes[1], node_color="tab:blue", node_size=node_size)
    nx.draw_networkx_labels(
        G, pos, {n: _node_label(n, "name") for n in G.nodes},
        font_size=6, ax=axes[1],
    )

    # Panel 3: Critical path
    total_dp = critical_path_pressure_drop(network)
    _draw_base(axes[2], f"Critical Path — Pressure drop {total_dp:.2f} [Pa]")
    crit_nodes = [n for n in G.nodes if n in crit_set]
    non_crit = [n for n in G.nodes if n not in crit_set]
    nx.draw_networkx_nodes(G, pos, ax=axes[2], nodelist=non_crit, node_color="tab:orange", node_size=node_size)
    nx.draw_networkx_nodes(G, pos, ax=axes[2], nodelist=crit_nodes, node_color="tab:green", node_size=node_size)
    nx.draw_networkx_edge_labels(
        G, pos, ax=axes[2],
        edge_labels={(u, v): _edge_label(u, v, "critical_pressure") for u, v in G.edges},
        font_size=6,
    )

    # Panel 4: Flowrate
    _draw_base(axes[3], "Flowrate [m³/s]")
    nx.draw_networkx_nodes(G, pos, ax=axes[3], node_color="tab:blue", node_size=node_size)
    nx.draw_networkx_edge_labels(
        G, pos, ax=axes[3],
        edge_labels={(u, v): _edge_label(u, v, "flowrate") for u, v in G.edges},
        font_size=6,
    )

    # Panel 5: Pressure drop
    _draw_base(axes[4], "Pressure drop [Pa]")
    nx.draw_networkx_nodes(G, pos, ax=axes[4], node_color="tab:blue", node_size=node_size)
    nx.draw_networkx_edge_labels(
        G, pos, ax=axes[4],
        edge_labels={(u, v): _edge_label(u, v, "pressure") for u, v in G.edges},
        font_size=6,
    )

    fig.tight_layout()
    if save_path:
        fig.savefig(save_path, dpi=120, bbox_inches="tight")
    if show:
        plt.show()
    return fig, axes


def summary_text(network: Network) -> str:
    """Return a text summary of the network and critical path."""
    from .network.solver import critical_path, critical_path_pressure_drop
    from .results import extract_results

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
