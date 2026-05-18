"""Smoke tests for the ``wenta`` command-line interface."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

import pytest

REPO = Path(__file__).resolve().parents[2]
EXAMPLE = REPO / "python" / "pyduct" / "examples" / "network_yaml.yaml"


def _run(*args: str) -> subprocess.CompletedProcess:
    return subprocess.run(
        [sys.executable, "-m", "pyduct", *args],
        capture_output=True,
        text=True,
        cwd=REPO,
        check=False,
    )


def test_info_returns_zero_on_healthy_network() -> None:
    r = _run("info", str(EXAMPLE))
    assert r.returncode == 0, r.stderr
    assert "Components:" in r.stdout
    assert "Sources:" in r.stdout
    assert "Terminals:" in r.stdout


def test_solve_text_includes_pressure_drops() -> None:
    r = _run("solve", str(EXAMPLE))
    assert r.returncode == 0, r.stderr
    assert "Critical-path pressure drop" in r.stdout


def test_solve_json_is_parseable() -> None:
    r = _run("solve", str(EXAMPLE), "--format", "json")
    assert r.returncode == 0, r.stderr
    data = json.loads(r.stdout)
    assert "components" in data
    assert "critical_path_dp_pa" in data
    assert data["critical_path_dp_pa"] > 0


def test_solve_markdown_has_table_header() -> None:
    r = _run("solve", str(EXAMPLE), "--format", "markdown")
    assert r.returncode == 0, r.stderr
    assert "| ID | Name |" in r.stdout


def test_solve_csv_is_parseable() -> None:
    r = _run("solve", str(EXAMPLE), "--format", "csv")
    assert r.returncode == 0, r.stderr
    assert "component_id" in r.stdout.splitlines()[0]


def test_validate_passes_on_healthy_network() -> None:
    r = _run("validate", str(EXAMPLE))
    assert r.returncode == 0


def test_unknown_format_rejected() -> None:
    r = _run("solve", str(EXAMPLE), "--format", "xlsx")
    assert r.returncode != 0


def test_solve_output_flag_writes_file(tmp_path: Path) -> None:
    out = tmp_path / "result.md"
    r = _run("solve", str(EXAMPLE), "--format", "markdown", "--output", str(out))
    assert r.returncode == 0, r.stderr
    assert out.exists()
    assert "Critical-path pressure drop" in out.read_text()


@pytest.mark.parametrize("subcmd", ["solve", "info", "validate"])
def test_missing_file_fails_cleanly(subcmd: str, tmp_path: Path) -> None:
    r = _run(subcmd, str(tmp_path / "nope.yaml"))
    assert r.returncode != 0
