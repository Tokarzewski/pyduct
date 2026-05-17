default:
    @just --list

lint:
    uvx ruff check .

check:
    uv run pytest

update:
    uv sync --upgrade