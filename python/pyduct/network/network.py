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

from collections.abc import Iterator
from dataclasses import dataclass, field
from typing import TYPE_CHECKING

import networkx as nx

from ..components.base import Component, Port

if TYPE_CHECKING:
    from ..components.fitting import Source, Terminal
    from ..core.fluid import Fluid


def port_node_id(component_id: str, port_name: str) -> str:
    """Stable graph-node id for a port."""
    return f"{component_id}:{port_name}"


@dataclass(repr=False)
class Network:
    """A directed graph of ductwork components.

    Build a network by calling :meth:`add` for each component and then
    :meth:`connect` for each physical airflow connection. Call :meth:`solve`
    (or the pure functions in :mod:`pyduct.network.solver`) to compute.
    """

    name: str = ""
    components: dict[str, Component] = field(default_factory=dict)
    graph: nx.DiGraph = field(default_factory=nx.DiGraph)
    _topo_cache: list[str] | None = field(default=None, init=False, repr=False)
    _preds_cache: dict[str, list[str]] | None = field(default=None, init=False, repr=False)
    _terminals_cache: list[Terminal] | None = field(default=None, init=False, repr=False)
    # Int-indexed projection for Mojo kernels. Built lazily.
    _int_index_cache: dict[str, int] | None = field(default=None, init=False, repr=False)
    _int_topo_cache: list[int] | None = field(default=None, init=False, repr=False)
    _int_preds_cache: list[list[int]] | None = field(default=None, init=False, repr=False)
    # Component-view caches for the Mojo batch compute kernel.
    _comp_types_cache: list[int] | None = field(default=None, init=False, repr=False)
    _comp_params_cache: list[float] | None = field(default=None, init=False, repr=False)
    _comp_port_idx_cache: list[int] | None = field(default=None, init=False, repr=False)
    # Pre-flattened (Port, flat_index) list for fast scatter/gather.
    _flat_ports_cache: list[tuple[object, int]] | None = field(
        default=None, init=False, repr=False
    )

    # ---- building the network ----------------------------------------------

    def add(self, component_id: str, component: Component) -> Component:
        """Register a component in the network under `component_id`.

        Returns the component so the call can be used inline.
        """
        if component_id in self.components:
            raise ValueError(f"duplicate component id: {component_id!r}")

        self.components[component_id] = component
        self.graph.add_node(
            component_id, kind="component", component=component,
            flowrate=0.0, pressure_drop=0.0,
        )

        for p in component.ports:
            pid = port_node_id(component_id, p.name)
            p.node_id = pid
            self.graph.add_node(
                pid,
                kind="port",
                component_id=component_id,
                port=p,
                flowrate=0.0,
                pressure_drop=0.0,
            )
            if p.direction == "in":
                # air enters the component through this port: port -> component
                self.graph.add_edge(pid, component_id)
            else:
                # air leaves the component through this port: component -> port
                self.graph.add_edge(component_id, pid)
        self._invalidate_caches()
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
        self._invalidate_caches()

    def _invalidate_caches(self) -> None:
        self._topo_cache = None
        self._preds_cache = None
        self._terminals_cache = None
        self._int_index_cache = None
        self._int_topo_cache = None
        self._int_preds_cache = None
        self._comp_types_cache = None
        self._comp_params_cache = None
        self._comp_port_idx_cache = None
        self._flat_ports_cache = None

    # ---- analysis ----------------------------------------------------------

    def topo_order(self) -> list[str]:
        """Topological order of graph nodes, cached until the graph changes."""
        if self._topo_cache is None:
            self._topo_cache = list(nx.topological_sort(self.graph))
        return self._topo_cache

    def predecessors_map(self) -> dict[str, list[str]]:
        """``node_id -> [predecessor_ids]``, cached until the graph changes.

        Materialising this avoids hitting NetworkX's PredView wrapper on every
        solver iteration.
        """
        if self._preds_cache is None:
            G = self.graph
            self._preds_cache = {n: list(G.predecessors(n)) for n in G.nodes}
        return self._preds_cache

    def int_topo_view(self) -> tuple[dict[str, int], list[int], list[list[int]]]:
        """Int-indexed projection used by the Mojo solver kernel.

        Returns ``(node_index, int_topo, int_preds)``:
          * ``node_index[node_id]`` → integer index in ``int_topo``
          * ``int_topo`` is the topological order with integer node ids
          * ``int_preds[i]`` lists predecessors of node ``i`` as integers
        Cached until the graph changes.
        """
        if self._int_index_cache is None:
            topo = self.topo_order()
            self._int_index_cache = {n: i for i, n in enumerate(topo)}
            self._int_topo_cache = list(range(len(topo)))
            preds = self.predecessors_map()
            idx = self._int_index_cache
            self._int_preds_cache = [[idx[p] for p in preds[n]] for n in topo]
        return (
            self._int_index_cache,
            self._int_topo_cache,
            self._int_preds_cache,
        )

    def component_view(self) -> tuple[list[int], list[float], list[int]]:
        """Flat-array projection used by the Mojo ``batch_compute`` kernel.

        Returns ``(types, params, port_indices)``:
          * ``types[i]``         — tag identifying component i's type
                                   (0=Source, 1=Terminal, 2=RigidDuct,
                                   3=FlexDuct, 4=TwoPortFitting, 5=Tee)
          * ``params``           — flat ``len(components) * 6`` floats;
                                   per-type packing documented in
                                   ``wenta.network.compute_batch``
          * ``port_indices``     — flat ``len(components) * 3`` ints; each
                                   triple is (port0, port1, port2) where
                                   unused slots are ``-1``. The integer
                                   index targets ``int_topo_view()``'s
                                   ``node_index`` mapping.
        Cached until the graph changes.
        """
        if self._comp_types_cache is None:
            from math import pi
            from ..components.duct import FlexDuct, RigidDuct
            from ..components.fitting import Source, Tee, Terminal, TwoPortFitting

            node_index, _, _ = self.int_topo_view()
            types: list[int] = []
            params: list[float] = []
            port_idx: list[int] = []
            for comp in self.components.values():
                # Pad each row to fixed width.
                row_params = [0.0] * 6
                row_ports = [-1, -1, -1]
                if isinstance(comp, Source):
                    types.append(0)
                    row_ports[0] = node_index[comp.ports[0].node_id]
                elif isinstance(comp, Terminal):
                    types.append(1)
                    row_ports[0] = node_index[comp.ports[0].node_id]
                    if comp.cross_section is not None:
                        row_params[0] = comp.cross_section.area
                        row_params[1] = comp.zeta
                elif isinstance(comp, RigidDuct):
                    types.append(2)
                    row_ports[0] = node_index[comp.ports[0].node_id]  # inlet
                    row_ports[1] = node_index[comp.ports[1].node_id]  # outlet
                    row_params[0] = comp.cross_section.area
                    row_params[1] = comp.cross_section.hydraulic_diameter
                    row_params[2] = comp.length
                    row_params[3] = comp.absolute_roughness
                elif isinstance(comp, FlexDuct):
                    types.append(3)
                    row_ports[0] = node_index[comp.ports[0].node_id]
                    row_ports[1] = node_index[comp.ports[1].node_id]
                    row_params[0] = pi * (comp.diameter / 2) ** 2  # area
                    row_params[1] = comp.diameter
                    row_params[2] = comp.length
                    row_params[3] = comp.pressure_drop_per_meter
                    row_params[4] = comp.stretch_percentage
                elif isinstance(comp, Tee):
                    types.append(5)
                    row_ports[0] = node_index[comp.ports[0].node_id]  # combined in
                    row_ports[1] = node_index[comp.ports[1].node_id]  # straight out
                    row_ports[2] = node_index[comp.ports[2].node_id]  # branch out
                    row_params[0] = comp.cross_section.area
                    row_params[1] = comp.zeta_straight
                    row_params[2] = comp.zeta_branch
                elif isinstance(comp, TwoPortFitting):
                    types.append(4)
                    row_ports[0] = node_index[comp.ports[0].node_id]
                    row_ports[1] = node_index[comp.ports[1].node_id]
                    row_params[0] = comp.cross_section.area
                    row_params[1] = comp.zeta
                else:
                    raise TypeError(f"unsupported component type for batch view: {type(comp).__name__}")
                params.extend(row_params)
                port_idx.extend(row_ports)
            self._comp_types_cache = types
            self._comp_params_cache = params
            self._comp_port_idx_cache = port_idx
        return (
            self._comp_types_cache,
            self._comp_params_cache,
            self._comp_port_idx_cache,
        )

    def flat_ports(self) -> list[tuple]:
        """Cached ``[(port, flat_index), ...]`` for fast scatter/gather.

        Used by ``compute_pressure_drops`` to avoid a nested
        components × ports loop and to skip the per-port node_index
        dict lookup on every solve.
        """
        if self._flat_ports_cache is None:
            node_index, _, _ = self.int_topo_view()
            self._flat_ports_cache = [
                (p, node_index[p.node_id])
                for comp in self.components.values()
                for p in comp.ports
            ]
        return self._flat_ports_cache

    def terminals(self) -> list[Terminal]:
        """Cached list of :class:`Terminal` components in the network."""
        if self._terminals_cache is None:
            from ..components.fitting import Terminal as _Terminal

            self._terminals_cache = [
                c for c in self.components.values() if isinstance(c, _Terminal)
            ]
        return self._terminals_cache

    def sources(self) -> list[Source]:
        """List of :class:`Source` components in the network."""
        from ..components.fitting import Source as _Source

        return [c for c in self.components.values() if isinstance(c, _Source)]

    def validate(self) -> list[str]:
        """Return a list of structural problems with the network.

        An empty list means the network is healthy (has at least one source
        and one terminal, and every registered component is wired to at
        least one other component).
        """
        problems: list[str] = []
        if not self.sources():
            problems.append("no Source component")
        if not self.terminals():
            problems.append("no Terminal component")
        G = self.graph
        for cid, comp in self.components.items():
            connected = False
            for p in comp.ports:
                neighbours = (
                    G.successors(p.node_id)
                    if p.direction == "out"
                    else G.predecessors(p.node_id)
                )
                for n in neighbours:
                    if G.nodes[n].get("kind") == "port":
                        connected = True
                        break
                if connected:
                    break
            if not connected:
                problems.append(f"component {cid!r} is not connected")
        return problems

    def solve(self, fluid: Fluid | None = None) -> float:
        """Run the full solver and return critical-path pressure drop [Pa]."""
        from ..core.fluid import STANDARD_AIR
        from .solver import solve

        return solve(self, fluid if fluid is not None else STANDARD_AIR)

    def summary(self) -> dict[str, float | int]:
        """One-shot stats for the network.

        Returns counts and the total terminal demand. ``critical_path_dp`` is
        only meaningful after :meth:`solve`; otherwise it is reported as 0.
        """
        from ..components.fitting import Terminal
        from .solver import critical_path_pressure_drop

        n_terminals = sum(isinstance(c, Terminal) for c in self.components.values())
        total_flow = sum(
            c.flowrate for c in self.components.values() if isinstance(c, Terminal)
        )
        return {
            "components": len(self.components),
            "terminals": n_terminals,
            "total_flowrate_m3s": total_flow,
            "critical_path_dp_pa": critical_path_pressure_drop(self),
        }

    # ---- serialization (thin wrappers around pyduct.io) --------------------

    @classmethod
    def from_dict(cls, data: dict) -> Network:
        from ..io import load_network_from_dict
        return load_network_from_dict(data)

    def to_dict(self) -> dict:
        from ..io import save_network_to_dict
        return save_network_to_dict(self)

    @classmethod
    def from_yaml(cls, filepath: str) -> Network:
        from ..io import load_from_yaml
        return load_from_yaml(filepath)

    def to_yaml(self, filepath: str) -> None:
        from ..io import save_to_yaml
        save_to_yaml(self, filepath)

    @classmethod
    def from_json(cls, filepath: str) -> Network:
        from ..io import load_from_json
        return load_from_json(filepath)

    def to_json(self, filepath: str) -> None:
        from ..io import save_to_json
        save_to_json(self, filepath)

    # ---- iteration & indexing ---------------------------------------------

    def iter_components(self) -> Iterator[tuple[str, Component]]:
        return iter(self.components.items())

    def __len__(self) -> int:
        return len(self.components)

    def __contains__(self, component_id: object) -> bool:
        return component_id in self.components

    def __getitem__(self, component_id: str) -> Component:
        return self.components[component_id]

    def __repr__(self) -> str:
        return (
            f"Network(name={self.name!r}, "
            f"components={len(self.components)}, "
            f"connections={self.graph.number_of_edges()})"
        )

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
