.PHONY: build test lint fmt fmt-check dev clean all

all: lint test

dev:
	uv sync
	maturin develop

build:
	cargo build
	maturin develop

test:
	cargo test
	uv run pytest

lint:
	cargo clippy -- -D warnings
	uv run ruff check python/

fmt:
	cargo fmt
	uv run ruff format python/

fmt-check:
	cargo fmt --check
	uv run ruff format --check python/

clean:
	cargo clean
	rm -rf dist/ .pytest_cache/
	find . \( -name "*.so" -o -name "*.dylib" -o -name "__pycache__" \) -exec rm -rf {} +
