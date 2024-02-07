#!/bin/bash

# Base directory of your Rust project
DIRECTORY="$HOME/codes/shade/contracts/liquidity_book"

process_directory() {
    local directory=$1
    local depth=$2
    local indent=$(printf '  %.0s' $(seq 1 $depth))

    # Process .rs files
    for filename in "$directory"/*.rs; do
        if [ -f "$filename" ]; then
            echo "${indent}- $(basename "$filename")"
            # Simple pattern to match function definitions
            grep -Eo 'fn [^(]*\(' "$filename" | sed -E 's/fn (.*)\(/  - \1/' | sed "s/^/${indent}  /"
        fi
    done

    # Recursively process subdirectories
    for subdir in "$directory"/*/; do
        if [ -d "$subdir" ]; then
            echo "${indent}- $(basename "$subdir")"
            process_directory "$subdir" $((depth+1))
        fi
    done
}

process_directory "$DIRECTORY" 0