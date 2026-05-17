default:
    @just --list

# Python side
lint:
    uvx ruff check .

check:
    uv run --extra dev pytest python/tests

types:
    uv run --extra dev mypy python/pyduct

# Mojo side
mojo-test:
    uv run mojo run mojoduct/tests/test_core.mojo

# Everything
test-all: check mojo-test
    @echo "All tests passed."

update:
    uv sync --upgrade
