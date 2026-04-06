"""Extract results from a solved network into structured formats."""

from __future__ import annotations

from dataclasses import asdict, dataclass
from typing import Any, Iterator

from .components.base import Component
from .network.network import Network, port_node_id


@dataclass
class ComponentResult:
    """Results for a single component."""

    component_id: str
    name: str
    component_type: str
    flowrate_in: float | None
    flowrate_out: float | None
    velocity_in: float | None
    velocity_out: float | None
    pressure_drop: float

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)


def extract_results(network: Network) -> list[ComponentResult]:
    """Extract results from a solved network into a list of ComponentResult.

    Each component appears as one row, with aggregated data from its ports.
    Useful for creating tables, exporting to CSV, or building reports.

    Parameters
    ----------
    network:
        A Network after calling :func:`pyduct2.network.solve`.

    Returns
    -------
    results:
        A list of ComponentResult, one per component.
    """
    results = []
    for cid, comp in network.components.items():
        # Aggregate port data.
        in_ports = comp.inlets()
        out_ports = comp.outlets()

        flowrate_in = (
            in_ports[0].flowrate if in_ports and in_ports[0].flowrate is not None else None
        )
        flowrate_out = (
            out_ports[0].flowrate if out_ports and out_ports[0].flowrate is not None else None
        )
        velocity_in = in_ports[0].velocity if in_ports and in_ports[0].velocity is not None else None
        velocity_out = (
            out_ports[0].velocity if out_ports and out_ports[0].velocity is not None else None
        )

        # Total pressure drop across all ports.
        total_dp = sum(p.pressure_drop for p in comp.ports)

        result = ComponentResult(
            component_id=cid,
            name=comp.name,
            component_type=type(comp).__name__,
            flowrate_in=flowrate_in,
            flowrate_out=flowrate_out,
            velocity_in=velocity_in,
            velocity_out=velocity_out,
            pressure_drop=total_dp,
        )
        results.append(result)

    return results


def results_summary(network: Network) -> str:
    """Format results as a human-readable table.

    Parameters
    ----------
    network:
        A solved Network.

    Returns
    -------
    table:
        A string containing a formatted table of results.
    """
    results = extract_results(network)
    if not results:
        return "(no components)"

    # Column widths
    col_widths = {
        "ID": 12,
        "Name": 20,
        "Type": 18,
        "Q_in [m³/s]": 13,
        "V_in [m/s]": 12,
        "ΔP [Pa]": 11,
    }

    # Header
    header = " | ".join(
        f"{k:<{col_widths[k]}}" for k in col_widths.keys()
    )
    sep = "-" * (sum(col_widths.values()) + len(col_widths) * 3 - 3)

    lines = [sep, header, sep]

    # Rows
    for res in results:
        q_in = f"{res.flowrate_in:.3f}" if res.flowrate_in is not None else "—"
        v_in = f"{res.velocity_in:.2f}" if res.velocity_in is not None else "—"
        dp = f"{res.pressure_drop:.2f}"

        row = " | ".join(
            [
                f"{res.component_id:<{col_widths['ID']}}",
                f"{res.name:<{col_widths['Name']}}",
                f"{res.component_type:<{col_widths['Type']}}",
                f"{q_in:>{col_widths['Q_in [m³/s]'] - 1}}",
                f"{v_in:>{col_widths['V_in [m/s]'] - 1}}",
                f"{dp:>{col_widths['ΔP [Pa]'] - 1}}",
            ]
        )
        lines.append(row)

    lines.append(sep)
    return "\n".join(lines)


def results_as_dicts(network: Network) -> list[dict[str, Any]]:
    """Export results as a list of dictionaries (suitable for CSV/JSON).

    Parameters
    ----------
    network:
        A solved Network.

    Returns
    -------
    dicts:
        List of dictionaries, one per component.
    """
    return [r.to_dict() for r in extract_results(network)]


def results_as_csv(network: Network, delimiter: str = ",") -> str:
    """Export results as CSV text.

    Parameters
    ----------
    network:
        A solved Network.
    delimiter:
        CSV delimiter (default: comma).

    Returns
    -------
    csv_text:
        CSV-formatted string with header row.
    """
    results = extract_results(network)
    if not results:
        return ""

    # Header
    keys = list(results[0].to_dict().keys())
    lines = [delimiter.join(keys)]

    # Rows
    for res in results:
        values = [str(v) if v is not None else "" for v in res.to_dict().values()]
        lines.append(delimiter.join(values))

    return "\n".join(lines)
