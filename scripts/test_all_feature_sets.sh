#!/usr/bin/env bash
set -euo pipefail

# $1 = comma-separated features (e.g. "featA,featB,featC")
# $2 = optional comma-separated test dirs (e.g. "tests/examples,tests/fast")
FEATURES_CSV="${1:-}"
DIRS_CSV="${2:-}"

IFS=',' read -ra FEATURES <<< "$FEATURES_CSV"

# Build all non-empty feature combinations (power set minus empty set)
COMBOS=$((1 << ${#FEATURES[@]}))
COMBINATIONS=()
for ((i=1; i<COMBOS; i++)); do
  lst=()
  for ((j=0; j<${#FEATURES[@]}; j++)); do
    (( i & (1<<j) )) && lst+=("${FEATURES[j]}")
  done
  COMBINATIONS+=("$(IFS=','; echo "${lst[*]}")")
done

echo "Will run the following feature combinations:"
printf '%s\n' "${COMBINATIONS[@]}"

# Collect test targets if dirs provided (each *.rs under those dirs becomes a --test <file-stem>)
TEST_TARGETS=()
if [[ -n "$DIRS_CSV" ]]; then
  IFS=',' read -ra DIRS <<< "$DIRS_CSV"
  for d in "${DIRS[@]}"; do
    if [[ -d "$d" ]]; then
      while IFS= read -r f; do
        TEST_TARGETS+=("$(basename "${f%.rs}")")
      done < <(find "$d" -maxdepth 1 -type f -name '*.rs' | sort)
    else
      echo "WARN: directory not found: $d" >&2
    fi
  done
  # de-duplicate
  mapfile -t TEST_TARGETS < <(printf '%s\n' "${TEST_TARGETS[@]}" | awk '!(seen[$0]++)')
fi

for COMB in "${COMBINATIONS[@]}"; do
  echo "-----------------------------------------------------------------"
  if [[ ${#TEST_TARGETS[@]} -eq 0 ]]; then
    echo "Running: cargo test --features \"$COMB\""
    cargo test --features "$COMB" -- --test-threads=1
  else
    for t in "${TEST_TARGETS[@]}"; do
      echo "Running: cargo test --features \"$COMB\" --test $t"
      cargo test --features "$COMB" --test "$t" -- --test-threads=1
    done
  fi
done
