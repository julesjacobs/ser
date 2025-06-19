#!/usr/bin/env bash
set -euo pipefail

# Usage: ./compare_optimizations.sh TIMEOUT_SECONDS
# Runs each .ser example in examples/ser/ serially under all combinations of five optimization flags.
# Records mode, example, and elapsed time (ms) into compare_results.csv.

# Timeout (seconds) to pass to each `cargo run` invocation
TIMEOUT_SECONDS="${1:-30}"
TIMEOUT_CMD="${TIMEOUT_CMD:-timeout}"  # allow override if needed (e.g. on mac use gtimeout)

# Output CSV file
outfile="compare_results.csv"

echo "mode,example,elapsed_ms" > "$outfile"

echo "Running all .ser examples with timeout=${TIMEOUT_SECONDS}s"
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

# For each .ser file in examples/ser/
for ser_file in examples/ser/*.ser; do
  EXAMPLE=$(basename "$ser_file" .ser)
  echo "Processing example: $EXAMPLE"

  # Enumerate all subsets of flags (0 .. 2^n-1)
  for mask in $(seq 0 $((2**n - 1))); do
    subset=()
    for ((i=0; i<n; i++)); do
      if (( (mask>>i)&1 )); then
        subset+=("${flags[i]}")
      fi
    done

    # Build human-readable label
    if [ ${#subset[@]} -eq 0 ]; then
      label="regular"
      mode_args=""
    else
      # strip leading dashes and join by '+'
      label=$(printf "%s+" "${subset[@]#--without-}")
      label=${label%+}
      mode_args="${subset[*]}"
    fi

    echo "=== Mode: $label ==="

    # Run under timeout, measure in ms
    echo -n "  [timeout ${TIMEOUT_SECONDS}s] start: "
    date +"%T.%3N"
    t0=$(date +%s%3N)

    set +e
    $TIMEOUT_CMD --preserve-status ${TIMEOUT_SECONDS}s \
      cargo run -- "$ser_file" --timeout "$TIMEOUT_SECONDS" $mode_args
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

echo "Results written to $outfile"
