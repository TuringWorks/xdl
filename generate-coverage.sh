#!/usr/bin/env bash
# Generate code coverage report for SonarQube
# This script runs tests with coverage and generates an LCOV report

set -e

echo "🧪 Running tests with coverage..."

# Clean previous coverage data
cargo llvm-cov clean --workspace

# Generate coverage report in LCOV format (compatible with SonarQube)
cargo llvm-cov --workspace --lcov --output-path coverage.lcov

echo "✅ Coverage report generated: coverage.lcov"
echo "📊 View coverage locally with: cargo llvm-cov --workspace --html"
echo "🔍 Run sonar-scanner to upload to SonarQube"
