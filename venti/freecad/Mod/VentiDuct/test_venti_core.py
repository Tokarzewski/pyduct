# Tests for the FreeCAD workbench's pure-Python venti core (no FreeCAD needed).
# Run: pytest test_venti_core.py   (skips if no backend / wasmtime absent)

import pytest

from venti_core import get_core


def _core():
    try:
        return get_core()
    except Exception:
        pytest.skip("no venti backend available (wasmtime or native lib)")


def test_friction_matches_reference():
    core = _core()
    try:
        assert abs(core.friction_factor(50000.0, 0.0009) - 0.0236446) < 1e-4
    finally:
        core.close()


def test_velocity_method_round_returns_200mm():
    core = _core()
    try:
        d, v = core.velocity_method_round(0.1, 4.0)
        assert abs(d - 0.2) < 1e-3
        assert v <= 4.0
        assert abs(v - 0.1 / (3.14159 * 0.01)) < 0.01
    finally:
        core.close()


def test_insulation_condensation_positive():
    core = _core()
    try:
        t = core.insulation_condensation(8.0, 15.8, 24.0, 0.035, 0.2)
        assert t is not None and t > 0.0
    finally:
        core.close()


def test_trace_straight_run():
    core = _core()
    try:
        res = core.trace_network([[(0.0, 0.0), (5.0, 0.0)]])
        try:
            # source + duct + terminal
            assert res.component_count() == 3
            assert res.solve() >= 0.0
        finally:
            res.free()
    finally:
        core.close()


def test_trace_tee_split():
    core = _core()
    try:
        # two polylines sharing the junction at (2, 0): a trunk passing
        # through it plus a branch -> source + tee + 2 ducts + 2 terminals.
        res = core.trace_network([[(0.0, 0.0), (2.0, 0.0), (4.0, 0.0)],
                                  [(2.0, 0.0), (2.0, 2.0)]])
        try:
            assert res.component_count() >= 6
            assert res.solve() >= 0.0
        finally:
            res.free()
    finally:
        core.close()


def test_trace_rejects_unsupported_geometry():
    core = _core()
    try:
        # 4-way cross (degree-4 junction) is rejected -> -1 -> ValueError.
        with pytest.raises(ValueError):
            core.trace_network([
                [(-2.0, 0.0), (0.0, 0.0), (2.0, 0.0)],
                [(0.0, -2.0), (0.0, 0.0), (0.0, 2.0)],
            ])
    finally:
        core.close()