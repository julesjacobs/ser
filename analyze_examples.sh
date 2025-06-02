#!/bin/bash

# Script to analyze all .ser examples and generate a serializability report
# Usage: ./analyze_examples.sh [--timeout <seconds>]

set -e

OUTPUT_FILE="serializability_report.md"
TEMP_FILE="temp_results.txt"

# Parse command line arguments
TIMEOUT_ARG=""
while [[ $# -gt 0 ]]; do
    case $1 in
        --timeout)
            TIMEOUT_ARG="--timeout $2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: ./analyze_examples.sh [--timeout <seconds>]"
            exit 1
            ;;
    esac
done

echo "🔍 Analyzing Serializability of .ser Examples"
echo "=============================================="
echo ""

# Create markdown header
cat > "$OUTPUT_FILE" << 'EOF'
# Serializability Analysis Report

This report shows the serializability analysis results for all `.ser` examples using the new method with Petri net pruning.

## Results

| Example | Result | Description |
|---------|--------|-------------|
EOF

# Find all .ser files and count them
files=($(find examples/ser -name "*.ser" | sort))
total=${#files[@]}
current=0

echo "Found $total .ser files to analyze"
echo ""

# Process each file
for file in "${files[@]}"; do
    current=$((current + 1))
    filename=$(basename "$file" .ser)
    
    printf "[$current/$total] Processing %-40s" "$filename..."
    
    # Run the analysis and capture output
    # always run this script with the optimizations
    if output=$(cargo run --quiet -- $TIMEOUT_ARG "$file" 2>&1); then
        # Extract the new method result
        if echo "$output" | grep -q "New method (with pruning): Serializable"; then
            result="✅ Serializable"
            status="Serializable"
        elif echo "$output" | grep -q "New method (with pruning): Not serializable"; then
            result="❌ Not serializable"
            status="Non-serializable"
        else
            result="❓ Unknown"
            status="Unknown"
        fi
        
        echo " $status"
        
        # Add description based on filename
        description=""
        case "$filename" in
            simple_ser) description="Basic serializable program" ;;
            simple_nonser*) description="Basic non-serializable program" ;;
            bank*) description="Banking transaction example" ;;
            arithmetic) description="Arithmetic operations" ;;
            boolean_ops) description="Boolean logic operations" ;;
            BGP_routing) description="BGP routing protocol" ;;
            snapshot_isolation*) description="Snapshot isolation example" ;;
            stateful_firewall*) description="Stateful firewall example" ;;
            multiple_requests) description="Multiple concurrent requests" ;;
            fred*) description="Fred example variant" ;;
            flag_*) description="Flag-based synchronization" ;;
            *while*) description="While loop constructs" ;;
            *if*) description="Conditional constructs" ;;
            *yield*) description="Yield-based concurrency" ;;
            *nondet*) description="Non-deterministic behavior" ;;
            *) description="General example" ;;
        esac
        
        # Append to markdown file
        echo "| \`$filename\` | $result | $description |" >> "$OUTPUT_FILE"
        
    else
        echo " Error"
        echo "| \`$filename\` | ⚠️ Error | Analysis failed or timed out |" >> "$OUTPUT_FILE"
    fi
done

# Add footer to markdown
cat >> "$OUTPUT_FILE" << 'EOF'

## Summary

- ✅ **Serializable**: Programs that maintain serializability properties
- ❌ **Not serializable**: Programs that violate serializability  
- ❓ **Unknown**: Could not determine result
- ⚠️ **Error**: Analysis failed or timed out

## Method

This analysis uses the new serializability checking method with Petri net pruning that:

1. Extracts zero variables from constraints using `extract_zero_variables`
2. Identifies nonzero variables (target places for filtering)  
3. Applies bidirectional iterative filtering to keep only relevant transitions
4. Uses SMPT (Satisfiability Modulo Petri Nets) for final reachability analysis

The pruning optimization removes transitions that cannot contribute to reaching nonzero places, potentially improving both performance and accuracy.

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