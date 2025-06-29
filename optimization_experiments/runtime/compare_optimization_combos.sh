#!/usr/bin/env bash
set -euo pipefail

# Runs each .json and .ser file under six optimization modes,
# writing a single combined CSV with one row per run:
#   example, bidirectional_pruning ON, remove_redundant ON,
#   generate_less ON, smart_order ON, elapsed_ms

TIMEOUT_SECONDS=10
TIMEOUT_CMD="${TIMEOUT_CMD:-timeout}"

OUTPUT_DIR="/home/guyamir/RustroverProjects/ser/optimization_experiments/runtime"
EXAMPLES_ROOT="/home/guyamir/RustroverProjects/ser/examples"
mkdir -p "$OUTPUT_DIR"

# The four --without-… flags, in fixed order
flags=(
  "--without-bidirectional"
  "--without-remove-redundant"
  "--without-generate-less"
  "--without-smart-kleene-order"
)
n=${#flags[@]}
mask_all=$((2**n - 1))

# Build our six masks: none, all, and each all-with-one-off
masks=(0 $mask_all)
for ((i=0; i<n; i++)); do
  masks+=($(( mask_all ^ (1<<i) )))
done

# Combined CSV output
outfile="${OUTPUT_DIR}/compare_results_timeout_${TIMEOUT_SECONDS}s_combined.csv"
echo "example,bidirectional_pruning ON,remove_redundant ON,generate_less ON,smart_order ON,elapsed_ms" > "$outfile"

for FILE_TYPE in json ser; do
  EXAMPLES_DIR="${EXAMPLES_ROOT}/${FILE_TYPE}"
  for file in "${EXAMPLES_DIR}"/*.${FILE_TYPE}; do
    example=$(basename "$file")  # includes .json or .ser

    for mask in "${masks[@]}"; do
      # gather any --without flags for this mask
      subset=()
      for ((i=0; i<n; i++)); do
        if (( (mask>>i)&1 )); then
          subset+=("${flags[i]}")
        fi
      done

      # compute ON bits: 1 if optimization is on (i.e. flag NOT used), 0 otherwise
      bidirectional_on=$(( ((mask>>0)&1) ? 0 : 1 ))
      remove_redundant_on=$(( ((mask>>1)&1) ? 0 : 1 ))
      generate_less_on=$(( ((mask>>2)&1) ? 0 : 1 ))
      smart_order_on=$(( ((mask>>3)&1) ? 0 : 1 ))

      # run & time
      t0=$(date +%s%3N)
      set +e
      $TIMEOUT_CMD --preserve-status ${TIMEOUT_SECONDS}s \
        cargo run -- "$file" --timeout "$TIMEOUT_SECONDS" "${subset[@]}"
      ret=$?
      set -e
      t1=$(date +%s%3N)

      if [[ $ret -eq 124 ]]; then
        dt="timeout"
      else
        dt=$((t1 - t0))
      fi

      # append one row
      echo \
"${example},${bidirectional_on},${remove_redundant_on},${generate_less_on},${smart_order_on},${dt}" \
        >> "$outfile"
    done
  done
done
