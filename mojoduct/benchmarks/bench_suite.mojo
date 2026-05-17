"""Comprehensive Mojo↔Python speedup benchmark.

Times every hot mojoduct kernel against its Python equivalent on a
representative workload. Prints a single summary table.

Run with:

    just mojo-suite
    # or
    uv run mojo run mojoduct/benchmarks/bench_suite.mojo
"""

from std.python import Python, PythonObject
from std.time import perf_counter_ns

from mojoduct.physics.friction import friction_factor
from mojoduct.physics.losses import local_pressure_drop, straight_pressure_drop
from mojoduct.sizing import (
    velocity_method_round,
    velocity_method_rectangular,
    equal_friction_method_round,
    equal_friction_method_rectangular,
    aspect_ratio_method,
    noise_limit_method_round,
)
from mojoduct.components.fittings_library import (
    rectangular_elbow,
    junction_tee_branch,
)


comptime N_FRICTION = 1_000_000
comptime N_LOSSES = 1_000_000
comptime N_SIZING = 50_000
comptime N_FITTINGS = 100_000


def _print_row(name: String, n: Int, mojo_ms: Float64, py_ms: Float64) -> None:
    var speedup = py_ms / mojo_ms
    print(
        "  ", name,
        "n=", n,
        " mojo=", mojo_ms, "ms",
        " python=", py_ms, "ms",
        " speedup=", speedup, "x",
    )


def _bench_friction_factor() raises:
    var py = Python.import_module("pyduct.physics.friction")
    var py_ff = py.friction_factor

    for i in range(1000):
        _ = friction_factor(3000.0 + Float64(i), 1.0e-4)
        _ = py_ff(3000.0 + Float64(i), 1.0e-4)

    var t0 = perf_counter_ns()
    var s: Float64 = 0.0
    for i in range(N_FRICTION):
        var eps = 1.0e-5 + Float64(i % 10) * 1.1e-4
        var re = 3000.0 + Float64(i) * 0.01
        s += friction_factor(re, eps)
    var mojo_ns = perf_counter_ns() - t0
    _ = s

    var t1 = perf_counter_ns()
    var ps: Float64 = 0.0
    for i in range(N_FRICTION):
        var eps = 1.0e-5 + Float64(i % 10) * 1.1e-4
        var re = 3000.0 + Float64(i) * 0.01
        ps += Float64(py=py_ff(re, eps))
    var py_ns = perf_counter_ns() - t1
    _ = ps

    _print_row(
        String("friction_factor"), N_FRICTION,
        Float64(mojo_ns) * 1.0e-6, Float64(py_ns) * 1.0e-6,
    )


def _bench_local_pressure_drop() raises:
    var py = Python.import_module("pyduct.physics.losses")
    var py_lpd = py.local_pressure_drop

    var t0 = perf_counter_ns()
    var s: Float64 = 0.0
    for i in range(N_LOSSES):
        # Vary all three args so the compiler can't constant-fold the loop.
        var z = 0.3 + Float64(i % 7) * 0.05
        var v = 1.0 + Float64(i % 100) * 0.05
        var rho = 1.15 + Float64(i % 5) * 0.01
        s += local_pressure_drop(z, v, rho)
    var mojo_ns = perf_counter_ns() - t0

    var t1 = perf_counter_ns()
    var ps: Float64 = 0.0
    for i in range(N_LOSSES):
        var z = 0.3 + Float64(i % 7) * 0.05
        var v = 1.0 + Float64(i % 100) * 0.05
        var rho = 1.15 + Float64(i % 5) * 0.01
        ps += Float64(py=py_lpd(z, v, rho))
    var py_ns = perf_counter_ns() - t1

    # Observe both sums so DCE can't drop the loops; print so the user
    # also sees that the two sides agree.
    print("  (local_pressure_drop checksums:", s, "/", ps, ")")
    _print_row(
        String("local_pressure_drop"), N_LOSSES,
        Float64(mojo_ns) * 1.0e-6, Float64(py_ns) * 1.0e-6,
    )


def main() raises:
    print("Mojoduct vs Pyduct — kernel benchmark")
    print("======================================================================")
    _bench_friction_factor()
    _bench_local_pressure_drop()

    # Sizing kernels — wrap each in a closure both sides can call by flow alone.
    var py_sizing = Python.import_module("pyduct.sizing")

    # velocity_method_round
    var t0 = perf_counter_ns()
    var s0: Float64 = 0.0
    for i in range(N_SIZING):
        var flow = 0.02 + Float64(i % 1000) * 5.0e-4
        var pair = velocity_method_round(flow, 4.0)
        s0 += pair[0].diameter
    var m_vmr = perf_counter_ns() - t0
    _ = s0
    var t1 = perf_counter_ns()
    var ps0: Float64 = 0.0
    for i in range(N_SIZING):
        var flow = 0.02 + Float64(i % 1000) * 5.0e-4
        var r = py_sizing.velocity_method(flow, "round", 4.0)
        ps0 += Float64(py=r[0].diameter)
    var p_vmr = perf_counter_ns() - t1
    _ = ps0
    _print_row(String("velocity_method_round"), N_SIZING,
               Float64(m_vmr) * 1.0e-6, Float64(p_vmr) * 1.0e-6)

    # velocity_method_rectangular
    var t2 = perf_counter_ns()
    var s2: Float64 = 0.0
    for i in range(N_SIZING):
        var flow = 0.05 + Float64(i % 1000) * 1.0e-3
        var pair = velocity_method_rectangular(flow, 4.0)
        s2 += pair[0].area
    var m_vmre = perf_counter_ns() - t2
    _ = s2
    var t3 = perf_counter_ns()
    var ps2: Float64 = 0.0
    for i in range(N_SIZING):
        var flow = 0.05 + Float64(i % 1000) * 1.0e-3
        var r = py_sizing.velocity_method(flow, "rectangular", 4.0)
        ps2 += Float64(py=r[0].area)
    var p_vmre = perf_counter_ns() - t3
    _ = ps2
    _print_row(String("velocity_method_rectangular"), N_SIZING,
               Float64(m_vmre) * 1.0e-6, Float64(p_vmre) * 1.0e-6)

    # equal_friction_method_round
    var t4 = perf_counter_ns()
    var s4: Float64 = 0.0
    for i in range(N_SIZING):
        var flow = 0.02 + Float64(i % 1000) * 5.0e-4
        var triple = equal_friction_method_round(flow, 1.0)
        s4 += triple[0].diameter
    var m_ef = perf_counter_ns() - t4
    _ = s4
    var t5 = perf_counter_ns()
    var ps4: Float64 = 0.0
    for i in range(N_SIZING):
        var flow = 0.02 + Float64(i % 1000) * 5.0e-4
        var r = py_sizing.equal_friction_method(flow, 1.0, "round")
        ps4 += Float64(py=r[0].diameter)
    var p_ef = perf_counter_ns() - t5
    _ = ps4
    _print_row(String("equal_friction_method_round"), N_SIZING,
               Float64(m_ef) * 1.0e-6, Float64(p_ef) * 1.0e-6)

    # aspect_ratio_method
    var t6 = perf_counter_ns()
    var s6: Float64 = 0.0
    for i in range(N_SIZING):
        var flow = 0.05 + Float64(i % 1000) * 1.0e-3
        var pair = aspect_ratio_method(flow, 4.0, 2.0)
        s6 += pair[0].width
    var m_ar = perf_counter_ns() - t6
    _ = s6
    var t7 = perf_counter_ns()
    var ps6: Float64 = 0.0
    for i in range(N_SIZING):
        var flow = 0.05 + Float64(i % 1000) * 1.0e-3
        var r = py_sizing.aspect_ratio_method(flow, 4.0, 2.0)
        ps6 += Float64(py=r[0].width)
    var p_ar = perf_counter_ns() - t7
    _ = ps6
    _print_row(String("aspect_ratio_method"), N_SIZING,
               Float64(m_ar) * 1.0e-6, Float64(p_ar) * 1.0e-6)

    # rectangular_elbow (representative fittings correlation)
    var py_fits = Python.import_module("pyduct.components.fittings_library")
    var py_re = py_fits.rectangular_elbow
    var t8 = perf_counter_ns()
    var s8: Float64 = 0.0
    for i in range(N_FITTINGS):
        var ang = 30.0 + Float64(i % 90) * 1.0
        s8 += rectangular_elbow(0.4, 0.3, 0.3, ang)
    var m_re = perf_counter_ns() - t8
    _ = s8
    var t9 = perf_counter_ns()
    var ps8: Float64 = 0.0
    for i in range(N_FITTINGS):
        var ang = 30.0 + Float64(i % 90) * 1.0
        ps8 += Float64(py=py_re(0.4, 0.3, 0.3, ang))
    var p_re = perf_counter_ns() - t9
    _ = ps8
    _print_row(String("rectangular_elbow"), N_FITTINGS,
               Float64(m_re) * 1.0e-6, Float64(p_re) * 1.0e-6)

    print("======================================================================")
