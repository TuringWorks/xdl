#!/bin/bash
# Test all XDL and MATLAB examples

set -e  # Exit on error

echo "========================================"
echo "Testing XDL Examples"
echo "========================================"
echo ""

echo "--- Test 1: Hello World ---"
xdl examples/xdl/01_hello_world.xdl
echo ""

echo "--- Test 2: Arrays and Loops ---"
xdl examples/xdl/02_arrays_and_loops.xdl
echo ""

echo "--- Test 3: Plotting (generates xdl_plot.png) ---"
xdl examples/xdl/03_plotting_basics.xdl
echo ""

echo "--- Test 4: Trigonometry ---"
xdl examples/xdl/04_trigonometry.xdl
echo ""

echo "--- Test 5: Conditionals ---"
xdl examples/xdl/05_conditionals.xdl
echo ""

echo "========================================"
echo "Testing MATLAB Examples"
echo "========================================"
echo ""

echo "--- Test 1: Simple Math ---"
xdl examples/matlab/01_simple_math.m
echo ""

echo "--- Test 2: Trigonometry ---"
xdl examples/matlab/02_trigonometry.m
echo ""

echo "--- Test 3: Simple Operations ---"
xdl examples/matlab/03_simple_operations.m
echo ""

echo "========================================"
echo "All tests completed successfully!"
echo "========================================"
