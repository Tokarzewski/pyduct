default:
    @just --list

# Rust native core per-kernel micro-benchmarks (dependency-free, no criterion)
bench:
    cargo run --release --bin bench --manifest-path venti/Cargo.toml

# Python side
lint:
    uvx ruff check .

check:
    uv run --extra dev pytest python/tests

types:
    uv run --extra dev mypy python/wenta

# Mojo side — unit tests (closed-form expected values)
mojo-test:
    uv run mojo run wentamojo/tests/test_core.mojo

# Mojo↔Python parity: every Mojo function diff-tested vs. the Python reference
# via std.python interop. Tolerance 1e-9 (transcendentals) / 1e-12 (closed-form).
mojo-parity:
    uv run mojo run wentamojo/tests/test_parity.mojo

# Mojo↔Python micro-benchmark (friction_factor, 1M calls).
mojo-bench:
    uv run mojo run wentamojo/benchmarks/bench_friction.mojo

# Comprehensive kernel-by-kernel benchmark across the full Mojo surface.
mojo-suite:
    uv run mojo run wentamojo/benchmarks/bench_suite.mojo

# Everything — both sides, both kinds of Mojo tests
test-all: check mojo-test mojo-parity csharp-parity
    @echo "All tests passed."

# C# side — build Wenta.Core + parity runner and replay the vectors
# (551 assertions: sizing, solver, fittings, spline, units, catalog, bom, balancing, room).
csharp-parity:
    cmd /c csharp\\build.cmd
    csharp\\bin\\Wenta.Core.Tests.exe

# Regenerate the C# parity vectors from the wentamojo formula source
# (needs tools/.venv with scipy: uv pip install --python csharp/tools/.venv/Scripts/python.exe scipy numpy).
csharp-vectors:
    csharp\\tools\\.venv\\Scripts\\python.exe csharp\\tools\\gen_vectors.py

# Build the ZWCAD plugin (wenta C# core compiled in; run csharp-parity first).
zwcad-build:
    cmd /c zwcad-plugin\\build.cmd
    python zwcad-plugin\\make_cuix.py zwcad-plugin\\bin

update:
    uv sync --upgrade
