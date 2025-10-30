#!/bin/bash
# Fast check script - runs minimal checks quickly

echo "⚡ Running fast quality checks..."

# Format code
cargo fmt

# Run clippy with minimal checks
cargo clippy --all-targets --all-features -- -D warnings -A dead_code

# Run only fast unit tests
cargo test --lib --profile dev-fast

echo "✅ Fast checks completed!"