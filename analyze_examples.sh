#!/bin/bash

# Script to analyze all .ser examples and generate a serializability report (parallelized)
# Usage: ./analyze_examples.sh [--timeout <seconds>] [--jobs <number>]

set -e

OUTPUT_FILE="serializability_report.md"
TEMP_DIR=$(mktemp -d)

# Parse command line arguments
TIMEOUT_ARG=""
MAX_JOBS=$(nproc 2>/dev/null || echo 16)  # Default to number of CPU cores, fallback to 4

while [[ $# -gt 0 ]]; do
    case $1 in
        --timeout)
            TIMEOUT_ARG="--timeout $2"
            shift 2
            ;;
        --jobs)
            MAX_JOBS="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: ./analyze_examples.sh [--timeout <seconds>] [--jobs <number>]"
            exit 1
            ;;
    esac
done

echo "🔍 Analyzing Serializability of .ser Examples (Parallel)"
echo "======================================================"
echo "Using $MAX_JOBS parallel jobs"
echo ""

# Temporary directory already created with mktemp -d
# Set up cleanup trap to ensure temp directory is removed
trap 'rm -rf "$TEMP_DIR"' EXIT

# Create markdown header
cat > "$OUTPUT_FILE" << 'EOF'
# Serializability Analysis Report

This report shows the serializability analysis results for all `.ser` examples using both original and proof-based methods.

## Results

| Example | Result |
|---------|--------|
EOF

# Function to analyze a single file
analyze_file() {
    local file="$1"
    local index="$2"
    local timeout_arg="$3"
    local temp_dir="$4"
    
    local filename=$(basename "$file" .ser)
    local result_file="$temp_dir/result_$index.txt"
    
    # Run the analysis and capture output
    if output=$(cargo run --quiet -- $timeout_arg "$file" 2>&1); then
        # Extract results from both methods
        if echo "$output" | grep -q "Original method: Serializable" && echo "$output" | grep -q "Proof-based method: Yes"; then
            echo "| \`$filename\` | ✅ Serializable |" > "$result_file"
            echo "[$index] $filename: Serializable"
        elif echo "$output" | grep -q "Original method: Not serializable" && echo "$output" | grep -q "Proof-based method: No"; then
            echo "| \`$filename\` | ❌ Not serializable |" > "$result_file"
            echo "[$index] $filename: Not serializable"
        else
            echo "| \`$filename\` | ❓ Unknown |" > "$result_file"
            echo "[$index] $filename: Unknown"
        fi
    else
        echo "| \`$filename\` | ⚠️ Error |" > "$result_file"
        echo "[$index] $filename: Error"
    fi
}

# Export function for parallel execution
export -f analyze_file

# Find all .ser files and count them
files=($(find examples/ser -name "*.ser" | sort))
total=${#files[@]}

echo "Found $total .ser files to analyze"
echo ""

# Process files in parallel with job control
job_count=0
file_index=0

for file in "${files[@]}"; do
    # Wait if we've reached the maximum number of jobs
    while [ $(jobs -r | wc -l) -ge $MAX_JOBS ]; do
        sleep 0.1
    done
    
    # Start analysis in background
    analyze_file "$file" "$file_index" "$TIMEOUT_ARG" "$TEMP_DIR" &
    file_index=$((file_index + 1))
done

# Wait for all background jobs to complete
wait

echo ""
echo "🔄 Collecting results..."

# Collect results in order
for i in $(seq 0 $((total - 1))); do
    result_file="$TEMP_DIR/result_$i.txt"
    if [ -f "$result_file" ]; then
        cat "$result_file" >> "$OUTPUT_FILE"
    fi
done

# Temporary directory will be cleaned up by EXIT trap

# Add footer to markdown
cat >> "$OUTPUT_FILE" << 'EOF'

## Summary

- ✅ **Serializable**: Programs that maintain serializability properties
- ❌ **Not serializable**: Programs that violate serializability  
- ❓ **Unknown**: Could not determine result
- ⚠️ **Error**: Analysis failed or timed out

---

*Report generated automatically by analyze_examples.sh*
EOF

echo ""
echo "✅ Analysis complete!"
echo "📊 Results saved to: $OUTPUT_FILE"
echo ""

# Show summary
serializable_count=$(grep -c "✅ Serializable" "$OUTPUT_FILE" || true)
not_serializable_count=$(grep -c "❌ Not serializable" "$OUTPUT_FILE" || true)
unknown_count=$(grep -c "❓ Unknown" "$OUTPUT_FILE" || true)
error_count=$(grep -c "⚠️ Error" "$OUTPUT_FILE" || true)

echo "📈 Summary:"
echo "   Serializable: $serializable_count"
echo "   Not serializable: $not_serializable_count"
echo "   Unknown: $unknown_count"
echo "   Errors: $error_count"
echo "   Total: $total"

echo ""
echo "🔗 View the full report: cat $OUTPUT_FILE"