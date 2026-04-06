"""Network serialization and deserialization (YAML/JSON I/O).

Load networks from YAML/JSON files, serialize solved networks back to disk,
and validate inputs using Pydantic schemas.
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import TYPE_CHECKING, Any

from .components import (
    ElbowRound,
    FlexDuct,
    RigidDuct,
    Source,
    Tee,
    Terminal,
    TwoPortFitting,
)
from .components.elbow import ElbowRound
from .core import Rectangular, Round, STANDARD_AIR, Fluid
from .network import Network
from .schemas import NetworkDesignSchema, SizingRequestSchema

if TYPE_CHECKING:
    import yaml


def load_network_from_dict(data: dict[str, Any]) -> Network:
    """Load a network from a dictionary (from YAML/JSON).

    Parameters
    ----------
    data:
        Dictionary matching NetworkDesignSchema.

    Returns
    -------
    network:
        A constructed (but unsolved) Network.

    Raises
    ------
    ValidationError:
        If the schema is invalid.
    """
    schema = NetworkDesignSchema(**data)

    # Create network
    net = Network(schema.name)

    # Resolve fluid (default to standard air)
    if schema.fluid:
        fluid = Fluid(
            density=schema.fluid.density,
            dynamic_viscosity=schema.fluid.dynamic_viscosity,
        )
    else:
        fluid = STANDARD_AIR

    # Add components
    for cid, comp_dict in schema.components.items():
        comp_type = comp_dict.pop("type")

        if comp_type == "RigidDuct":
            cs_dict = comp_dict.pop("cross_section")
            section = _make_cross_section(cs_dict)
            comp = RigidDuct(
                cross_section=section,
                **comp_dict,
            )
        elif comp_type == "FlexDuct":
            comp = FlexDuct(**comp_dict)
        elif comp_type == "Source":
            comp = Source(**comp_dict)
        elif comp_type == "Terminal":
            comp = Terminal(**comp_dict)
        elif comp_type == "TwoPortFitting":
            cs_dict = comp_dict.pop("cross_section")
            section = _make_cross_section(cs_dict)
            comp = TwoPortFitting(
                cross_section=section,
                **comp_dict,
            )
        elif comp_type == "Tee":
            cs_dict = comp_dict.pop("cross_section")
            section = _make_cross_section(cs_dict)
            comp = Tee(cross_section=section, **comp_dict)
        else:
            raise ValueError(f"Unknown component type: {comp_type}")

        net.add(cid, comp)

    # Add connections
    # Note: ignore port-specific notation (component.port) for now;
    # Network.connect() handles default port disambiguation.
    for conn in schema.connections:
        # Strip port notation if present (e.g. "ahu:outlet" -> "ahu")
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


def _make_cross_section(cs_dict: dict[str, Any]) -> Round | Rectangular:
    """Helper to construct a CrossSection from a dict."""
    shape = cs_dict["shape"]
    if shape == "round":
        return Round(diameter=cs_dict["diameter"])
    else:  # rectangular
        return Rectangular(width=cs_dict["width"], height=cs_dict["height"])


def _component_to_dict(comp: Any) -> dict[str, Any]:
    """Helper to serialize a component."""
    comp_type = type(comp).__name__

    result = {
        "type": comp_type,
        "name": comp.name,
    }

    if comp_type in ("RigidDuct", "FlexDuct"):
        if comp_type == "RigidDuct":
            result["cross_section"] = {
                "shape": "round" if isinstance(comp.cross_section, Round) else "rectangular",
                "diameter": comp.cross_section.diameter if isinstance(comp.cross_section, Round) else None,
                "width": comp.cross_section.width if isinstance(comp.cross_section, Rectangular) else None,
                "height": comp.cross_section.height if isinstance(comp.cross_section, Rectangular) else None,
            }
            result["length"] = comp.length
            result["absolute_roughness"] = comp.absolute_roughness
        else:  # FlexDuct
            result["diameter"] = comp.diameter
            result["length"] = comp.length
            result["pressure_drop_per_meter"] = comp.pressure_drop_per_meter
            result["stretch_percentage"] = comp.stretch_percentage

    elif comp_type in ("TwoPortFitting", "Tee"):
        result["cross_section"] = {
            "shape": "round" if isinstance(comp.cross_section, Round) else "rectangular",
            "diameter": comp.cross_section.diameter if isinstance(comp.cross_section, Round) else None,
            "width": comp.cross_section.width if isinstance(comp.cross_section, Rectangular) else None,
            "height": comp.cross_section.height if isinstance(comp.cross_section, Rectangular) else None,
        }
        if comp_type == "TwoPortFitting":
            result["zeta"] = comp.zeta
        else:  # Tee
            result["zeta_straight"] = comp.zeta_straight
            result["zeta_branch"] = comp.zeta_branch

    elif comp_type == "Terminal":
        result["flowrate"] = comp.flowrate
        result["zeta"] = comp.zeta

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
    except ImportError:
        raise ImportError(
            "pyyaml is required for YAML I/O. "
            "Install it via: pip install pyyaml"
        )

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
    except ImportError:
        raise ImportError(
            "pyyaml is required for YAML I/O. "
            "Install it via: pip install pyyaml"
        )

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
