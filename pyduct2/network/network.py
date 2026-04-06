"""Ductwork network: a directed graph of components and ports.

The graph is built so that:

* every component is a node identified by its `component_id`;
* every port is a node identified by ``f"{component_id}:{port_name}"``;
* internal edges connect a component to its ports following the *physical*
  airflow direction (``in`` ports → component → ``out`` ports);
* connection edges between components go from one component's ``out`` port to
  another component's ``in`` port.

This makes ``networkx.dag_longest_path`` immediately usable for critical-path
analysis: the longest path from a :class:`Source` to a :class:`Terminal`,
weighted by the per-port pressure drop, is exactly the worst-case static
pressure required at the source.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Iterator

import networkx as nx

from ..components.base import Component, Port


def port_node_id(component_id: str, port_name: str) -> str:
    """Stable graph-node id for a port."""
    return f"{component_id}:{port_name}"


@dataclass
class Network:
    """A directed graph of ductwork components.

    Build a network by calling :meth:`add` for each component and then
    :meth:`connect` for each physical airflow connection. Solve it with
    :func:`pyduct2.network.solver.solve` (or the individual solver functions).
    """

    name: str = ""
    components: dict[str, Component] = field(default_factory=dict)
    graph: nx.DiGraph = field(default_factory=nx.DiGraph)

    # ---- building the network ----------------------------------------------

    def add(self, component_id: str, component: Component) -> Component:
        """Register a component in the network under `component_id`.

        Returns the component so the call can be used inline.
        """
        if component_id in self.components:
            raise ValueError(f"duplicate component id: {component_id!r}")

        self.components[component_id] = component
        self.graph.add_node(component_id, kind="component", component=component)

        for p in component.ports:
            pid = port_node_id(component_id, p.name)
            self.graph.add_node(
                pid,
                kind="port",
                component_id=component_id,
                port=p,
            )
            if p.direction == "in":
                # air enters the component through this port: port -> component
                self.graph.add_edge(pid, component_id)
            else:
                # air leaves the component through this port: component -> port
                self.graph.add_edge(component_id, pid)
        return component

    def connect(self, source: str, target: str) -> None:
        """Add a physical-airflow connection from `source` to `target`.

        Each endpoint is either ``"<component_id>"`` (the default port is used)
        or ``"<component_id>.<port_name>"``. The source must be an ``out``
        port and the target must be an ``in`` port.

        For two-port components (ducts, fittings) the default port is
        unambiguous. For multi-leg components (Tee) the port name must be
        given explicitly, e.g. ``"tee1.straight"``.
        """
        src_cid, src_port = self._resolve(source, "out")
        dst_cid, dst_port = self._resolve(target, "in")
        self.graph.add_edge(
            port_node_id(src_cid, src_port.name),
            port_node_id(dst_cid, dst_port.name),
        )

    # ---- iteration ---------------------------------------------------------

    def iter_components(self) -> Iterator[tuple[str, Component]]:
        return iter(self.components.items())

    # ---- internals ---------------------------------------------------------

    def _resolve(
        self, ref: str, expected_direction: str
    ) -> tuple[str, Port]:
        if "." in ref:
            cid, pname = ref.split(".", 1)
        else:
            cid, pname = ref, None

        if cid not in self.components:
            raise KeyError(f"unknown component id: {cid!r}")
        component = self.components[cid]

        if pname is not None:
            port = component.port(pname)
        else:
            matching = [p for p in component.ports if p.direction == expected_direction]
            if len(matching) == 0:
                raise ValueError(
                    f"component {cid!r} has no {expected_direction!r} ports"
                )
            if len(matching) > 1:
                raise ValueError(
                    f"component {cid!r} has multiple {expected_direction!r} "
                    f"ports {[p.name for p in matching]!r}; specify one with "
                    f"{cid!r} + '.<port_name>'"
                )
            port = matching[0]

        if port.direction != expected_direction:
            raise ValueError(
                f"port {cid}.{port.name} is {port.direction!r}, expected "
                f"{expected_direction!r}"
            )
        return cid, port
