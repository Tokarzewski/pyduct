"""Network serialization and deserialization (YAML/JSON I/O).

Load networks from YAML/JSON files, serialize solved networks back to disk,
and validate inputs using Pydantic schemas.
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from .components import (
    Component,
    FlexDuct,
    RigidDuct,
    Source,
    Tee,
    Terminal,
    TwoPortFitting,
)
from .core import Rectangular, Round
from .network import Network
from .schemas import (
    CrossSectionSchema,
    FlexDuctSchema,
    NetworkDesignSchema,
    RigidDuctSchema,
    SourceSchema,
    TeeSchema,
    TerminalSchema,
    TwoPortFittingSchema,
)


def _build_cross_section(cs: CrossSectionSchema) -> Round | Rectangular:
    if cs.shape == "round":
        assert cs.diameter is not None
        return Round(diameter=cs.diameter)
    assert cs.width is not None and cs.height is not None
    return Rectangular(width=cs.width, height=cs.height)


def _build_component(schema: Any) -> Component:
    """Concrete Component from a per-type Pydantic schema."""
    if isinstance(schema, RigidDuctSchema):
        return RigidDuct(
            name=schema.name,
            cross_section=_build_cross_section(schema.cross_section),
            length=schema.length,
            absolute_roughness=schema.absolute_roughness,
        )
    if isinstance(schema, FlexDuctSchema):
        return FlexDuct(
            name=schema.name,
            diameter=schema.diameter,
            length=schema.length,
            pressure_drop_per_meter=schema.pressure_drop_per_meter,
            stretch_percentage=schema.stretch_percentage,
        )
    if isinstance(schema, SourceSchema):
        return Source(name=schema.name)
    if isinstance(schema, TerminalSchema):
        return Terminal(name=schema.name, flowrate=schema.flowrate, zeta=schema.zeta)
    if isinstance(schema, TwoPortFittingSchema):
        return TwoPortFitting(
            name=schema.name,
            cross_section=_build_cross_section(schema.cross_section),
            zeta=schema.zeta,
        )
    if isinstance(schema, TeeSchema):
        return Tee(
            name=schema.name,
            cross_section=_build_cross_section(schema.cross_section),
            zeta_straight=schema.zeta_straight,
            zeta_branch=schema.zeta_branch,
        )
    raise TypeError(f"Unsupported component schema: {type(schema).__name__}")


def load_network_from_dict(data: dict[str, Any]) -> Network:
    """Load a network from a dictionary (from YAML/JSON).

    The input is fully validated by :class:`NetworkDesignSchema`, including
    per-component field validation via a Pydantic discriminated union, so an
    invalid component (wrong type, missing field, out-of-range value) raises
    :class:`pydantic.ValidationError` before any object is constructed.
    """
    schema = NetworkDesignSchema(**data)
    net = Network(schema.name)
    for cid, comp_schema in schema.components.items():
        net.add(cid, _build_component(comp_schema))
    for conn in schema.connections:
        src = conn.source.split(":")[0] if ":" in conn.source else conn.source
        tgt = conn.target.split(":")[0] if ":" in conn.target else conn.target
        net.connect(src, tgt)
    return net


def save_network_to_dict(net: Network) -> dict[str, Any]:
    """Serialize a network to a dictionary (for YAML/JSON export).

    Parameters
    ----------
    net:
        A Network (may or may not be solved).

    Returns
    -------
    dict:
        Dictionary matching NetworkDesignSchema.
    """
    components = {}
    for cid, comp in net.components.items():
        comp_dict = _component_to_dict(comp)
        components[cid] = comp_dict

    connections = []
    for u, v in net.graph.edges:
        # Only record port-to-port connections (not internal component-to-port).
        u_kind = net.graph.nodes[u].get("kind")
        v_kind = net.graph.nodes[v].get("kind")
        if u_kind == "port" and v_kind == "port":
            connections.append({"source": u, "target": v})

    return {
        "name": net.name,
        "fluid": None,  # Assume standard air for now
        "components": components,
        "connections": connections,
    }


def _cross_section_to_dict(cs: Round | Rectangular) -> dict[str, Any]:
    """Helper to serialize a CrossSection."""
    if isinstance(cs, Round):
        return {"shape": "round", "diameter": cs.diameter, "width": None, "height": None}
    return {"shape": "rectangular", "diameter": None, "width": cs.width, "height": cs.height}


# Each entry lists (a) whether the component carries a cross_section, and
# (b) the plain attributes that need to round-trip through YAML/JSON.
_COMPONENT_FIELDS: dict[str, tuple[bool, tuple[str, ...]]] = {
    "RigidDuct":      (True,  ("length", "absolute_roughness")),
    "FlexDuct":       (False, ("diameter", "length", "pressure_drop_per_meter", "stretch_percentage")),
    "TwoPortFitting": (True,  ("zeta",)),
    "Tee":            (True,  ("zeta_straight", "zeta_branch")),
    "Source":         (False, ()),
    "Terminal":       (False, ("flowrate", "zeta")),
}


def _component_to_dict(comp: Any) -> dict[str, Any]:
    """Helper to serialize a component."""
    comp_type = type(comp).__name__
    result: dict[str, Any] = {"type": comp_type, "name": comp.name}
    spec = _COMPONENT_FIELDS.get(comp_type)
    if spec is None:
        return result
    has_cs, attrs = spec
    if has_cs:
        result["cross_section"] = _cross_section_to_dict(comp.cross_section)
    for a in attrs:
        result[a] = getattr(comp, a)
    return result


def load_from_yaml(filepath: str | Path) -> Network:
    """Load a network from a YAML file.

    Parameters
    ----------
    filepath:
        Path to a .yaml or .yml file.

    Returns
    -------
    network:
        Constructed Network.

    Requires
    --------
    pyyaml (install via `pip install pyyaml`)
    """
    try:
        import yaml
    except ImportError as err:
        raise ImportError(
            "pyyaml is required for YAML I/O. "
            "Install it via: pip install pyyaml"
        ) from err

    filepath = Path(filepath)
    with open(filepath) as f:
        data = yaml.safe_load(f)

    return load_network_from_dict(data)


def save_to_yaml(net: Network, filepath: str | Path) -> None:
    """Save a network to a YAML file.

    Parameters
    ----------
    net:
        Network to save.
    filepath:
        Output .yaml or .yml file path.

    Requires
    --------
    pyyaml
    """
    try:
        import yaml
    except ImportError as err:
        raise ImportError(
            "pyyaml is required for YAML I/O. "
            "Install it via: pip install pyyaml"
        ) from err

    data = save_network_to_dict(net)
    filepath = Path(filepath)
    with open(filepath, "w") as f:
        yaml.safe_dump(data, f, default_flow_style=False, sort_keys=False)


def load_from_json(filepath: str | Path) -> Network:
    """Load a network from a JSON file."""
    filepath = Path(filepath)
    with open(filepath) as f:
        data = json.load(f)
    return load_network_from_dict(data)


def save_to_json(net: Network, filepath: str | Path, indent: int = 2) -> None:
    """Save a network to a JSON file."""
    data = save_network_to_dict(net)
    filepath = Path(filepath)
    with open(filepath, "w") as f:
        json.dump(data, f, indent=indent)
