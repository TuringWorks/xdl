#!/bin/bash
# Quick test script to transpile a MATLAB file

if [ -z "$1" ]; then
    echo "Usage: $0 <matlab_file>"
    exit 1
fi

cargo run --bin matlab-transpiler -- "$1"
