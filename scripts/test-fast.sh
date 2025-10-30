#!/bin/bash
# Fast development test script

echo "🚀 Running fast tests..."

# Run only unit tests with fast profile
cargo test --lib --profile dev-fast

echo "✅ Fast tests completed!"