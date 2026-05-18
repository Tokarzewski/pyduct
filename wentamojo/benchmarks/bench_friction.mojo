"""Micro-benchmark: Mojo `friction_factor` vs Python `wenta` reference.

The Python reference is the same friction_factor that powers the solver;
the Mojo implementation is a bit-for-bit port. This bench gives a real
number to the Mojo speedup so the port pays for itself at the level
that matters (the per-component inner loop).

Run via `just mojo-bench` (or directly):

    uv run mojo run wentamojo/benchmarks/bench_friction.mojo
"""

from std.python import Python
from std.time import perf_counter_ns

from wentamojo.physics.friction import friction_factor
from wentamojo.sizing import velocity_method_round, equal_friction_method_round


comptime N_CALLS = 1_000_000
comptime N_SIZING = 100_000


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
    var py_friction = Python.import_module("wenta.physics.friction")
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

    # --- velocity_method_round ---------------------------------------------
    var py_sizing = Python.import_module("wenta.sizing")
    var py_vm = py_sizing.velocity_method

    # Warm up both sides.
    for i in range(1000):
        _ = velocity_method_round(0.05 + Float64(i) * 1.0e-5, 4.0)
        _ = py_vm(0.05 + Float64(i) * 1.0e-5, "round", 4.0)

    var t2 = perf_counter_ns()
    var mojo_d_sum: Float64 = 0.0
    for i in range(N_SIZING):
        var flow = 0.02 + Float64(i % 1000) * 5.0e-4
        var pair = velocity_method_round(flow, 4.0)
        mojo_d_sum += pair[0].diameter
    var mojo_size_ns = perf_counter_ns() - t2

    var t3 = perf_counter_ns()
    var py_d_sum: Float64 = 0.0
    for i in range(N_SIZING):
        var flow = 0.02 + Float64(i % 1000) * 5.0e-4
        var r = py_vm(flow, "round", 4.0)
        py_d_sum += Float64(py=r[0].diameter)
    var py_size_ns = perf_counter_ns() - t3

    var mojo_size_ms = Float64(mojo_size_ns) * 1.0e-6
    var py_size_ms = Float64(py_size_ns) * 1.0e-6
    var size_speedup = Float64(py_size_ns) / Float64(mojo_size_ns)
    print("")
    print("velocity_method_round benchmark —", N_SIZING, "calls")
    print("  Mojo   :", mojo_size_ms, "ms   diam sum =", mojo_d_sum)
    print("  Python :", py_size_ms,   "ms   diam sum =", py_d_sum)
    print("  speedup:", size_speedup, "x")

    # --- equal_friction_method_round ---------------------------------------
    var py_ef = py_sizing.equal_friction_method

    for i in range(500):
        _ = equal_friction_method_round(0.05 + Float64(i) * 1.0e-5, 1.0)
        _ = py_ef(0.05 + Float64(i) * 1.0e-5, 1.0, "round")

    var t4 = perf_counter_ns()
    var mojo_ef_sum: Float64 = 0.0
    for i in range(N_SIZING):
        var flow = 0.02 + Float64(i % 1000) * 5.0e-4
        var triple = equal_friction_method_round(flow, 1.0)
        mojo_ef_sum += triple[0].diameter
    var mojo_ef_ns = perf_counter_ns() - t4

    var t5 = perf_counter_ns()
    var py_ef_sum: Float64 = 0.0
    for i in range(N_SIZING):
        var flow = 0.02 + Float64(i % 1000) * 5.0e-4
        var r = py_ef(flow, 1.0, "round")
        py_ef_sum += Float64(py=r[0].diameter)
    var py_ef_ns = perf_counter_ns() - t5

    var mojo_ef_ms = Float64(mojo_ef_ns) * 1.0e-6
    var py_ef_ms = Float64(py_ef_ns) * 1.0e-6
    var ef_speedup = Float64(py_ef_ns) / Float64(mojo_ef_ns)
    print("")
    print("equal_friction_method_round benchmark —", N_SIZING, "calls")
    print("  Mojo   :", mojo_ef_ms, "ms   diam sum =", mojo_ef_sum)
    print("  Python :", py_ef_ms,   "ms   diam sum =", py_ef_sum)
    print("  speedup:", ef_speedup, "x")
