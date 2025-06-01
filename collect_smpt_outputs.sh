#!/bin/bash

# Script to collect raw SMPT outputs from all examples to understand proof/model formats

OUTPUT_DIR="smpt_output_collection"
mkdir -p "$OUTPUT_DIR"

echo "=== SMPT Output Collection Script ==="
echo "Collecting outputs from all examples to understand proof/model formats"
echo

# Function to run SMPT on a single example and collect outputs
run_example() {
    local example="$1"
    local basename=$(basename "$example" .ser)
    
    echo "Processing: $example"
    
    # Create subdirectory for this example
    local example_dir="$OUTPUT_DIR/$basename"
    mkdir -p "$example_dir"
    
    # Run the serializability checker to generate SMPT files
    echo "  Generating SMPT files..."
    timeout 60s cargo run -- "$example" > "$example_dir/tool_output.txt" 2>&1
    local exit_code=$?
    
    if [ $exit_code -eq 124 ]; then
        echo "  TIMEOUT after 60s"
        echo "TIMEOUT" > "$example_dir/status.txt"
        return
    elif [ $exit_code -ne 0 ]; then
        echo "  FAILED with exit code $exit_code"
        echo "FAILED" > "$example_dir/status.txt"
        # Still try to collect SMPT files if they were generated
    else
        echo "  SUCCESS"
        echo "SUCCESS" > "$example_dir/status.txt"
    fi
    
    # Look for generated SMPT files
    local out_subdir="out/$basename"
    if [ -d "$out_subdir" ]; then
        # Copy the generated files for reference
        cp "$out_subdir"/*.xml "$example_dir/" 2>/dev/null || true
        cp "$out_subdir"/*.net "$example_dir/" 2>/dev/null || true
        
        # Find the XML and NET files
        local xml_file=$(find "$out_subdir" -name "*.xml" | head -1)
        local net_file=$(find "$out_subdir" -name "*.net" | head -1)
        
        if [ -n "$xml_file" ] && [ -n "$net_file" ]; then
            echo "  Running SMPT with different options..."
            
            # Standard run
            echo "    Standard run..."
            timeout 30s ./smpt_wrapper.sh -n "$net_file" --xml "$xml_file" > "$example_dir/smpt_standard.txt" 2>&1
            
            # With --show-model
            echo "    With --show-model..."
            timeout 30s ./smpt_wrapper.sh -n "$net_file" --xml "$xml_file" --show-model > "$example_dir/smpt_show_model.txt" 2>&1
            
            # With --check-proof
            echo "    With --check-proof..."
            timeout 30s ./smpt_wrapper.sh -n "$net_file" --xml "$xml_file" --check-proof > "$example_dir/smpt_check_proof.txt" 2>&1
            
            # With --export-proof (create proof file)
            echo "    With --export-proof..."
            timeout 30s ./smpt_wrapper.sh -n "$net_file" --xml "$xml_file" --export-proof "$example_dir/proof.out" > "$example_dir/smpt_export_proof.txt" 2>&1
            
            # Combined: --show-model --check-proof
            echo "    Combined --show-model --check-proof..."
            timeout 30s ./smpt_wrapper.sh -n "$net_file" --xml "$xml_file" --show-model --check-proof > "$example_dir/smpt_combined.txt" 2>&1
            
            # Also save the raw files for analysis
            cp "$xml_file" "$example_dir/constraints.xml" 2>/dev/null || true
            cp "$net_file" "$example_dir/petri.net" 2>/dev/null || true
        else
            echo "  No SMPT files found in $out_subdir"
        fi
    else
        echo "  No output directory found: $out_subdir"
    fi
    
    echo "  Done."
    echo
}

# Process all .ser examples
echo "Processing .ser examples..."
for example in examples/ser/*.ser; do
    if [ -f "$example" ]; then
        run_example "$example"
    fi
done

# Process JSON examples too
echo "Processing .json examples..."
for example in examples/json/*.json; do
    if [ -f "$example" ]; then
        local basename=$(basename "$example" .json)
        echo "Processing: $example"
        
        local example_dir="$OUTPUT_DIR/$basename"
        mkdir -p "$example_dir"
        
        timeout 60s cargo run -- "$example" > "$example_dir/tool_output.txt" 2>&1
        local exit_code=$?
        
        if [ $exit_code -eq 124 ]; then
            echo "  TIMEOUT after 60s"
            echo "TIMEOUT" > "$example_dir/status.txt"
        elif [ $exit_code -ne 0 ]; then
            echo "  FAILED with exit code $exit_code"
            echo "FAILED" > "$example_dir/status.txt"
        else
            echo "  SUCCESS"
            echo "SUCCESS" > "$example_dir/status.txt"
        fi
        echo
    fi
done

echo "=== Collection Complete ==="
echo "Results saved in: $OUTPUT_DIR/"
echo
echo "Summary:"
find "$OUTPUT_DIR" -name "status.txt" -exec dirname {} \; | while read dir; do
    status=$(cat "$dir/status.txt")
    basename=$(basename "$dir")
    echo "  $basename: $status"
done

echo
echo "To analyze results:"
echo "  find $OUTPUT_DIR -name '*.txt' -exec echo '=== {} ===' \; -exec cat {} \;"
echo "  find $OUTPUT_DIR -name 'proof.out' -exec echo '=== {} ===' \; -exec cat {} \;"