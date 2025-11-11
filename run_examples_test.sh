#!/bin/bash

# Script to run all XDL examples and tests, collect results

OUTPUT_DIR="examples_output"
mkdir -p "$OUTPUT_DIR"

# Function to run a single file
run_file() {
    local file="$1"
    local basename=$(basename "$file" .${file##*.})
    local output_file="$OUTPUT_DIR/${basename}_output.txt"
    local error_file="$OUTPUT_DIR/${basename}_error.txt"

    echo "Running $file..."

    # Check if file exists and is not empty
    if [ ! -s "$file" ]; then
        echo "EMPTY: $file" >> "$OUTPUT_DIR/empty_files.txt"
        return
    fi

    # Run the file based on extension
    case "${file##*.}" in
        xdl)
            # Try to run with xdl interpreter
            if command -v xdl &> /dev/null; then
                timeout 30 xdl "$file" > "$output_file" 2> "$error_file"
                local exit_code=$?
            else
                echo "XDL interpreter not found" > "$error_file"
                local exit_code=1
            fi
            ;;
        m)
            # Try to run MATLAB/Octave
            if command -v matlab &> /dev/null; then
                timeout 30 matlab -batch "run('$file')" > "$output_file" 2> "$error_file"
                local exit_code=$?
            elif command -v octave &> /dev/null; then
                timeout 30 octave "$file" > "$output_file" 2> "$error_file"
                local exit_code=$?
            else
                echo "MATLAB/Octave not found" > "$error_file"
                local exit_code=1
            fi
            ;;
        pro)
            # IDL/GDL procedure
            if command -v gdl &> /dev/null; then
                timeout 30 gdl -e "@$file" > "$output_file" 2> "$error_file"
                local exit_code=$?
            else
                echo "GDL not found" > "$error_file"
                local exit_code=1
            fi
            ;;
        *)
            echo "Unknown file type" > "$error_file"
            local exit_code=1
            ;;
    esac

    # Record result
    if [ $exit_code -eq 0 ]; then
        echo "SUCCESS: $file" >> "$OUTPUT_DIR/success_files.txt"
        # Check if it generated any plots/visualization
        if grep -q -i "plot\|viz\|render\|chart" "$output_file" 2>/dev/null; then
            echo "VISUAL: $file" >> "$OUTPUT_DIR/visual_files.txt"
        fi
    else
        echo "FAILED: $file (exit code: $exit_code)" >> "$OUTPUT_DIR/failed_files.txt"
    fi
}

# Export function for parallel execution
export -f run_file
export OUTPUT_DIR

# Find all example and test files
find examples tests -name "*.xdl" -o -name "*.m" -o -name "*.pro" | \
    xargs -n 1 -P 4 bash -c 'run_file "$@"' _

echo "Example testing complete. Results in $OUTPUT_DIR/"
