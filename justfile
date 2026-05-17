default:
    @just --list

# Python side
lint:
    uvx ruff check .

check:
    uv run --extra dev pytest python/tests

types:
    uv run --extra dev mypy python/pyduct

# Mojo side — unit tests (closed-form expected values)
mojo-test:
    uv run mojo run mojoduct/tests/test_core.mojo

# Mojo↔Python parity: every Mojo function diff-tested vs. the Python reference
# via std.python interop. Tolerance 1e-9 (transcendentals) / 1e-12 (closed-form).
mojo-parity:
    uv run mojo run mojoduct/tests/test_parity.mojo

# Mojo↔Python micro-benchmark (friction_factor, 1M calls).
mojo-bench:
    uv run mojo run mojoduct/benchmarks/bench_friction.mojo

# Everything — both sides, both kinds of Mojo tests
test-all: check mojo-test mojo-parity
    @echo "All tests passed."

update:
    uv sync --upgrade
