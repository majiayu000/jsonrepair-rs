# Contributing

## Setup

```bash
git clone https://github.com/majiayu000/jsonrepair-rs.git
cd jsonrepair-rs
cargo build
cargo test
```

## Development

```bash
# Check compilation
cargo check

# Run tests
cargo test

# Format code
cargo fmt

# Check for warnings (CI-equivalent)
RUSTFLAGS="-Dwarnings" cargo check --all-targets
```

## Pre-commit

```bash
# Install pre-commit with uv
uv tool install pre-commit

# Install commit + push hooks
pre-commit install
pre-commit install --hook-type pre-push

# Run commit hooks manually
pre-commit run --all-files

# Run push hooks manually
pre-commit run --all-files --hook-stage pre-push
```

## Adding a new repair pattern

1. Add test case(s) in `tests/repair_tests.rs`
2. Run `cargo test` — confirm the test fails (RED)
3. Implement the fix in the relevant module under `src/parser/`
4. Run `cargo test` — confirm all tests pass (GREEN)
5. Submit a PR
