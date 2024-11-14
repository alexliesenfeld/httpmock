#!/bin/bash

set -e  # Exit immediately if any command fails

# Read the comma-separated list of features
IFS=',' read -ra FEATURES <<< "$1"

# Calculate all feature combinations
COMBOS=$((1 << ${#FEATURES[@]}))

# Store all feature combinations
COMBINATIONS=()

# Generate all combinations
for ((i = 1; i < COMBOS; i++)); do
    FEATURE_LIST=""
    for ((j = 0; j < ${#FEATURES[@]}; j++)); do
        if ((i & (1 << j))); then
            FEATURE_LIST+="${FEATURES[j]},"
        fi
    done
    # Remove the trailing comma
    FEATURE_LIST="${FEATURE_LIST%,}"
    COMBINATIONS+=("$FEATURE_LIST")
done

# Print all combinations
echo "Will run the following feature combinations:"
for COMB in "${COMBINATIONS[@]}"; do
    echo "$COMB"
done

# Run cargo test for each combination
for COMB in "${COMBINATIONS[@]}"; do
    echo "-----------------------------------------------------------------"
    echo "Running: cargo test --features \"$COMB\""
    cargo test --features "$COMB" -- --test-threads=1
done
