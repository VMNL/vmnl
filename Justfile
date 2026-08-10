# List available recipes.
default:
    @just --list

# Build an example without opening a window.
build example='d2_shapes':
    cargo build -p '{{example}}'

# Build every workspace target.
build-workspace:
    cargo build --workspace --all-targets

# Run an example.
run example='d2_shapes':
    cargo run -p '{{example}}'

# Run the headless test suites.
test: test-unit test-api test-smoke

# Run unit tests.
test-unit:
    cargo test --workspace --lib --exclude vmnl-api-tests --exclude vmnl-gpu-tests --exclude vmnl-smoke-tests

# Run public API tests.
test-api:
    cargo test -p vmnl-api-tests

# Run smoke tests.
test-smoke:
    cargo run -p vmnl-smoke-tests

# Compile GPU tests without running them.
test-gpu-compile:
    cargo test -p vmnl-gpu-tests --no-run

# Run GPU/display tests.
test-gpu:
    cargo test -p vmnl-gpu-tests -- --ignored

# Run Rustdoc examples.
doctest:
    cargo test --workspace --all-features --doc

# Run non-mutating checks.
check: check-fmt check-clippy

# Check formatting.
check-fmt:
    cargo fmt --all --check

# Run Clippy with warnings denied.
check-clippy:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

# Apply formatting and automatic fixes.
lint:
    cargo fmt --all
    cargo fix --workspace --all-targets --all-features --allow-dirty --allow-staged
    cargo clippy --workspace --all-targets --all-features --fix --allow-dirty --allow-staged
    cargo fmt --all

# Build Rustdoc with warnings denied.
docs:
    RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps

# Run the complete non-GPU validation sequence.
validate: build-workspace check doctest docs test

# Install Linux system dependencies.
bootstrap:
    ./deps
