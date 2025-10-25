#!/bin/bash
# Run slow tests that are excluded from normal cargo test runs

set -e

echo "========================================="
echo "Running Slow Tests"
echo "========================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to run tests from slow_tests directory
run_slow_test() {
    local test_file=$1
    local package=$2

    echo -e "${BLUE}Running: $test_file${NC}"

    # Copy test file temporarily back to package tests directory
    local package_dir="$package"
    local temp_test_dir="$package_dir/tests"
    mkdir -p "$temp_test_dir"

    local test_name=$(basename "$test_file" .rs)
    cp "$test_file" "$temp_test_dir/"

    # Run the test
    cd "$package_dir"
    cargo test --test "$test_name" -- --ignored --nocapture

    # Clean up
    rm "$temp_test_dir/$test_name.rs"
    cd ..

    echo ""
}

# Run MATLAB integration tests
if [ -d "slow_tests/xdl-matlab" ]; then
    echo -e "${GREEN}=== MATLAB Integration Tests ===${NC}"
    for test_file in slow_tests/xdl-matlab/*.rs; do
        if [ -f "$test_file" ]; then
            run_slow_test "$test_file" "xdl-matlab"
        fi
    done
fi

echo -e "${GREEN}=========================================${NC}"
echo -e "${GREEN}All slow tests completed!${NC}"
echo -e "${GREEN}=========================================${NC}"
