"""In-process tests for the ``wenta`` command-line interface.

Earlier draft used ``subprocess.run`` per case (~600 ms each — Python
startup dominated). This version calls ``cli.main(argv)`` directly and
captures stdout via ``capsys`` so the whole file runs in well under a
second.
"""

from __future__ import annotations

import json
from pathlib import Path

import pytest

from pyduct.cli import main as cli_main

REPO = Path(__file__).resolve().parents[2]
EXAMPLE = REPO / "python" / "pyduct" / "examples" / "network_yaml.yaml"


def _run(*argv: str, capsys) -> tuple[int, str]:
    """Invoke ``wenta`` in-process. Returns ``(exit_code, stdout)``."""
    try:
        rc = cli_main(list(argv))
    except SystemExit as e:  # argparse on parse error / _load on missing file
        rc = e.code if isinstance(e.code, int) else 1
    return rc, capsys.readouterr().out


def test_info_returns_zero_on_healthy_network(capsys) -> None:
    rc, out = _run("info", str(EXAMPLE), capsys=capsys)
    assert rc == 0
    assert "Components:" in out
    assert "Sources:" in out
    assert "Terminals:" in out


def test_solve_text_includes_pressure_drops(capsys) -> None:
    rc, out = _run("solve", str(EXAMPLE), capsys=capsys)
    assert rc == 0
    assert "Critical-path pressure drop" in out


def test_solve_json_is_parseable(capsys) -> None:
    rc, out = _run("solve", str(EXAMPLE), "--format", "json", capsys=capsys)
    assert rc == 0
    data = json.loads(out)
    assert "components" in data
    assert data["critical_path_dp_pa"] > 0


def test_solve_markdown_has_table_header(capsys) -> None:
    rc, out = _run("solve", str(EXAMPLE), "--format", "markdown", capsys=capsys)
    assert rc == 0
    assert "| ID | Name |" in out


def test_solve_csv_is_parseable(capsys) -> None:
    rc, out = _run("solve", str(EXAMPLE), "--format", "csv", capsys=capsys)
    assert rc == 0
    assert "component_id" in out.splitlines()[0]


def test_validate_passes_on_healthy_network(capsys) -> None:
    rc, _ = _run("validate", str(EXAMPLE), capsys=capsys)
    assert rc == 0


def test_unknown_format_rejected(capsys) -> None:
    rc, _ = _run("solve", str(EXAMPLE), "--format", "xlsx", capsys=capsys)
    assert rc != 0


def test_solve_output_flag_writes_file(tmp_path: Path, capsys) -> None:
    out_path = tmp_path / "result.md"
    rc, _ = _run(
        "solve", str(EXAMPLE),
        "--format", "markdown",
        "--output", str(out_path),
        capsys=capsys,
    )
    assert rc == 0
    assert out_path.exists()
    assert "Critical-path pressure drop" in out_path.read_text()


@pytest.mark.parametrize("subcmd", ["solve", "info", "validate"])
def test_missing_file_fails_cleanly(subcmd: str, tmp_path: Path, capsys) -> None:
    rc, _ = _run(subcmd, str(tmp_path / "nope.yaml"), capsys=capsys)
    assert rc != 0
