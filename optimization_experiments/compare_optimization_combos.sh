#!/usr/bin/env bash
set -euo pipefail

# Usage: ./compare_optimizations.sh TIMEOUT_SECONDS [json|ser]
# Runs each file in a hard-coded examples directory under all combinations of five optimization flags.
# Records mode, example, and elapsed time (ms) into /home/guyamir/RustroverProjects/ser/optimization_experiments/csvs/raw_inputs/compare_results_all_combos_timeout_${TIMEOUT_SECONDS}_seconds_<type>.csv.

# Timeout (seconds) to pass to each `cargo run` invocation
TIMEOUT_SECONDS="${1:-60}"
# File type: json or ser
FILE_TYPE="${2:-ser}"

TIMEOUT_CMD="${TIMEOUT_CMD:-timeout}"  # allow override if needed (e.g. on mac use gtimeout)

# Hard-coded directory paths
OUTPUT_DIR="/home/guyamir/RustroverProjects/ser/optimization_experiments/csvs/raw_inputs"
EXAMPLES_ROOT="/home/guyamir/RustroverProjects/ser/examples"
EXAMPLES_DIR="${EXAMPLES_ROOT}/${FILE_TYPE}"

# Ensure necessary directories exist
mkdir -p "$OUTPUT_DIR"

# Output CSV file
outfile="${OUTPUT_DIR}/compare_results_all_combos_timeout_${TIMEOUT_SECONDS}_seconds_${FILE_TYPE}.csv"

echo "mode,example,elapsed_ms" > "$outfile"

echo "Running all .${FILE_TYPE} examples with timeout=${TIMEOUT_SECONDS}s"
echo "Examples directory: $EXAMPLES_DIR"
echo "Results will be written to: $outfile"
echo

# Define all individual --without- flags
flags=(
  "--without-bidirectional"
  "--without-remove-redundant-parts"
  "--without-remove-redundant-sets"
  "--without-generate-less"
  "--without-smart-kleene-order"
)
n=${#flags[@]}

# For each file in the hard-coded examples directory
for file in "${EXAMPLES_DIR}"/*.${FILE_TYPE}; do
  EXAMPLE=$(basename "$file" .${FILE_TYPE})
  echo "Processing example: $EXAMPLE"

  # Enumerate all subsets of flags (0 .. 2^n-1)
  for mask in $(seq 0 $((2**n - 1))); do
    subset=()
    for ((i=0; i<n; i++)); do
      if (( (mask>>i)&1 )); then
        subset+=("${flags[i]}")
      fi
    done

    # Build explicit label: strip leading '--' and join by '--'
    if [ ${#subset[@]} -eq 0 ]; then
      label="all-on"
      mode_args=""
    else
      stripped=()
      for f in "${subset[@]}"; do
        stripped+=( "${f#--}" )
      done
      IFS='--' read -r label <<< "${stripped[*]}"
      mode_args="${subset[*]}"
    fi

    echo "=== Mode: $label ==="

    # Run under timeout, measure in ms
    echo -n "  [timeout ${TIMEOUT_SECONDS}s] start: "
    date +"%T.%3N"
    t0=$(date +%s%3N)

    set +e
    $TIMEOUT_CMD --preserve-status ${TIMEOUT_SECONDS}s \
      cargo run -- "$file" --timeout "$TIMEOUT_SECONDS" $mode_args
    ret=$?
    set -e
    t1=$(date +%s%3N)

    echo -n "  [timeout ${TIMEOUT_SECONDS}s] end:   "
    date +"%T.%3N"

    # Determine elapsed or timeout
    if [[ $ret -eq 124 ]]; then
      dt="timeout"
      echo "  → timed out (recording as 'timeout')"
    elif [[ $ret -eq 0 ]]; then
      dt=$((t1 - t0))
      echo "  → elapsed: ${dt} ms"
    else
      dt=$((t1 - t0))
      echo "  → error (exit $ret), elapsed: ${dt} ms"
    fi
    echo

    # Append to CSV
    echo "$label,$EXAMPLE,$dt" >> "$outfile"
  done
  echo
 done

echo "All runs complete. Results written to: $outfile"
