"""``wentamojo`` command-line interface.

Sub-commands:

    wentamojo solve  PATH              load a YAML/JSON network, solve, print results
    wentamojo info   PATH              load and report structure (no solve)
    wentamojo validate PATH            schema-validate the file; exit non-zero on error

Common flags:

    --format {text,json,csv,markdown}  output format (default: text)
    --output PATH                       write to file instead of stdout
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


def _load(path: Path):
    """Dispatch on the file extension. Exits cleanly on missing or invalid files."""
    from . import Network

    if not path.exists():
        raise SystemExit(f"no such file: {path}")
    suffix = path.suffix.lower()
    if suffix in (".yaml", ".yml"):
        return Network.from_yaml(str(path))
    if suffix == ".json":
        return Network.from_json(str(path))
    raise SystemExit(f"unsupported input format: {suffix} (expected .yaml/.yml/.json)")


def _emit(text: str, output: str | None) -> None:
    if output:
        Path(output).write_text(text)
    else:
        sys.stdout.write(text + ("" if text.endswith("\n") else "\n"))


def _format_solved(network, fmt: str, dp: float) -> str:
    from . import extract_results, results_as_csv, results_summary

    results = extract_results(network)
    if fmt == "json":
        return json.dumps(
            {
                "name": network.name,
                "critical_path_dp_pa": dp,
                "components": [r.to_dict() for r in results],
            },
            indent=2,
        )
    if fmt == "csv":
        return results_as_csv(network)
    if fmt == "markdown":
        lines = [
            f"# {network.name or 'wentamojo network'}",
            "",
            f"**Critical-path pressure drop:** {dp:.2f} Pa",
            "",
            "| ID | Name | Type | Q [m³/s] | v [m/s] | ΔP [Pa] |",
            "|----|------|------|---------:|--------:|--------:|",
        ]
        for r in results:
            q = f"{r.flowrate_in:.3f}" if r.flowrate_in is not None else "—"
            v = f"{r.velocity_in:.2f}" if r.velocity_in is not None else "—"
            lines.append(
                f"| {r.component_id} | {r.name} | {r.component_type} | {q} | {v} | {r.pressure_drop:.2f} |"
            )
        return "\n".join(lines)
    # text (default)
    header = f"Network: {network.name or '(unnamed)'}\nCritical-path pressure drop: {dp:.2f} Pa\n"
    return header + results_summary(network)


def _cmd_solve(args: argparse.Namespace) -> int:
    network = _load(Path(args.path))
    dp = network.solve()
    _emit(_format_solved(network, args.format, dp), args.output)
    return 0


def _cmd_info(args: argparse.Namespace) -> int:
    network = _load(Path(args.path))
    problems = network.validate()
    summary = (
        f"Network: {network.name or '(unnamed)'}\n"
        f"Components: {len(network)}\n"
        f"Sources:    {len(network.sources())}\n"
        f"Terminals:  {len(network.terminals())}\n"
        f"Edges:      {network.graph.number_of_edges()}\n"
    )
    if problems:
        summary += "\nValidation problems:\n" + "".join(f"  - {p}\n" for p in problems)
    _emit(summary, args.output)
    return 1 if problems else 0


def _cmd_validate(args: argparse.Namespace) -> int:
    try:
        network = _load(Path(args.path))
    except Exception as e:
        _emit(f"schema error: {e}", args.output)
        return 2
    problems = network.validate()
    if problems:
        _emit("validation problems:\n" + "\n".join(f"  - {p}" for p in problems), args.output)
        return 1
    _emit(f"OK: {args.path}", args.output)
    return 0


def main(argv: list[str] | None = None) -> int:
    p = argparse.ArgumentParser(
        prog="wentamojo",
        description="Ductwork design utility (Mojo-backed solver).",
    )
    sub = p.add_subparsers(dest="cmd", required=True)

    def _add_common(sp: argparse.ArgumentParser) -> None:
        sp.add_argument("path", help="Path to a network YAML or JSON file.")
        sp.add_argument(
            "--format",
            choices=("text", "json", "csv", "markdown"),
            default="text",
            help="Output format (default: text).",
        )
        sp.add_argument("--output", help="Write to file instead of stdout.")

    for name, fn in (("solve", _cmd_solve), ("info", _cmd_info), ("validate", _cmd_validate)):
        sp = sub.add_parser(name, help=f"{name} a network file")
        _add_common(sp)
        sp.set_defaults(func=fn)

    args = p.parse_args(argv)
    return args.func(args)


if __name__ == "__main__":
    raise SystemExit(main())
