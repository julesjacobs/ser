#!/usr/bin/env bash
set -euo pipefail

# Usage: ./compare_optimizations.sh TIMEOUT_SECONDS
# Runs each .ser example in examples/ser/ serially under five optimization modes.
# Records mode, example, and elapsed time (ms) into compare_results.csv.

# Timeout (seconds) to pass to each `cargo run` invocation
TIMEOUT_SECONDS="${1:-30}"
TIMEOUT_CMD="${TIMEOUT_CMD:-timeout}"  # allow override if needed (e.g. on mac use gtimeout)

# Output CSV file
outfile="compare_results.csv"

echo "mode,example,elapsed_ms" > "$outfile"

echo "Running all .ser examples with timeout=${TIMEOUT_SECONDS}s"
echo

# For each .ser file in examples/ser/
for ser_file in examples/ser/*.ser; do
  EXAMPLE=$(basename "$ser_file" .ser)
  echo "Processing example: $EXAMPLE"

  # The five optimization modes to compare
  MODES=(
    ""  # regular
    "--without-bidirectional"
    "--without-remove-redundant-parts"
    "--without-remove-redundant-sets"
    "--without-generate-less"
  )

  for mode in "${MODES[@]}"; do
    # human-readable label (strip leading dashes)
    if [[ -z "$mode" ]]; then
      label="regular"
    else
      label="${mode#--}"
    fi

    echo "=== Mode: $label ==="

    # Run the binary under `timeout`, but prevent script from exiting on non-zero
    echo -n "  [timeout ${TIMEOUT_SECONDS}s] start: "
    date +"%T.%3N"
    t0=$(date +%s%3N)

    set +e
    $TIMEOUT_CMD --preserve-status ${TIMEOUT_SECONDS}s \
      cargo run -- "$ser_file" --timeout "$TIMEOUT_SECONDS" $mode
    ret=$?
    set -e
    t1=$(date +%s%3N)

    echo -n "  [timeout ${TIMEOUT_SECONDS}s] end:   "
    date +"%T.%3N"

    # Handle exit codes
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

    # Append to CSV: mode,example,elapsed
    echo "$label,$EXAMPLE,$dt" >> "$outfile"
  done
  echo
 done

echo "Results written to $outfile"
