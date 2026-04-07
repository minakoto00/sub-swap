# justfile

# Fast compile check
check:
    cargo check

# Run all tests
test:
    cargo test

# Run clippy with warnings as errors
lint:
    cargo clippy -- -D warnings

# Apply formatting in-place
fmt:
    cargo fmt

# Validate all quality gates in sequence; stops on first failure
validate:
    cargo fmt --check && cargo clippy -- -D warnings && cargo test
