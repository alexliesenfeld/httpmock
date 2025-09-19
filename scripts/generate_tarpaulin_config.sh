#!/bin/bash

# Populate the features variable
features=$(awk '/^\[features\]/ {flag=1; next} /^\[/ {flag=0} flag {print}' Cargo.toml | \
    sed 's/^[ \t]*//;s/[ \t]*$//' | \
    grep -v '^\s*#' | \
    grep -v '^\s*$' | \
    cut -d'=' -f1 | \
    grep -vE "(default|color)")

# Convert the multiline string to an array
IFS=$'\n' read -r -d '' -a feature_array <<< "$features"

generate_powerset() {
  local items=("$@")
  local len=${#items[@]}
  local total=$((1 << len))

  for ((i=0; i<total; i++)); do
    subset=()
    quoted_subset=()  # This will hold the quoted version of the subset
    for ((j=0; j<len; j++)); do
      if (( (i & (1 << j)) != 0 )); then
        # Remove spaces from each item
        local cleaned_item="${items[j]// /}"
        subset+=("$cleaned_item")
        # Add the same item but wrapped in quotes for the quoted version
        quoted_subset+=("\"$cleaned_item\"")
      fi
    done

    if [ ${#subset[@]} -eq 0 ]; then
      echo "[none]"
    else
       # Print subset with elements joined by an underscore, no quotes
      local IFS='.'
        echo "[${subset[*]}]"
        local IFS=','
        echo "features = \"${subset[*]}\""
      fi

      # Print quoted subset with elements joined by a space, with quotes
      echo "release = true"
      echo ""
  done
}
generate_powerset "${feature_array[@]}"

echo '[report]'
echo 'out = ["Html", "Xml"]'

