"""Micro-benchmark: Mojo `friction_factor` vs Python `pyduct` reference.

The Python reference is the same friction_factor that powers the solver;
the Mojo implementation is a bit-for-bit port. This bench gives a real
number to the Mojo speedup so the port pays for itself at the level
that matters (the per-component inner loop).

Run via `just mojo-bench` (or directly):

    uv run mojo run mojoduct/benchmarks/bench_friction.mojo
"""

from std.python import Python
from std.time import perf_counter_ns

from mojoduct.physics.friction import friction_factor


comptime N_CALLS = 1_000_000


def _mojo_loop(n: Int) -> Float64:
    """Sum friction_factor over a sweep of (Re, eps) pairs; sum prevents DCE."""
    var s: Float64 = 0.0
    for i in range(n):
        # 10 evenly spaced eps values in [1e-5, 1e-3]; Re grows past laminar.
        var eps = 1.0e-5 + Float64(i % 10) * 1.1e-4
        var re = 3000.0 + Float64(i) * 0.01
        s += friction_factor(re, eps)
    return s


def main() raises:
    var py_friction = Python.import_module("pyduct.physics.friction")
    var py_ff = py_friction.friction_factor

    # Warm up both sides so the comparison is steady-state.
    _ = _mojo_loop(1000)
    for i in range(1000):
        _ = py_ff(3000.0 + Float64(i), 1.0e-4)

    var t0 = perf_counter_ns()
    var mojo_sum = _mojo_loop(N_CALLS)
    var mojo_ns = perf_counter_ns() - t0

    var t1 = perf_counter_ns()
    var py_sum: Float64 = 0.0
    for i in range(N_CALLS):
        var eps = 1.0e-5 + Float64(i % 10) * 1.1e-4
        var re = 3000.0 + Float64(i) * 0.01
        py_sum += Float64(py=py_ff(re, eps))
    var py_ns = perf_counter_ns() - t1

    var mojo_ms = Float64(mojo_ns) * 1.0e-6
    var py_ms = Float64(py_ns) * 1.0e-6
    var speedup = Float64(py_ns) / Float64(mojo_ns)

    print("friction_factor benchmark —", N_CALLS, "calls")
    print("  Mojo   :", mojo_ms, "ms   sum =", mojo_sum)
    print("  Python :", py_ms,   "ms   sum =", py_sum)
    print("  speedup:", speedup, "x")
