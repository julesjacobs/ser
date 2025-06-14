#!/usr/bin/env python3
"""
Test script to check for empty proof files when running fred.ser
"""
import subprocess
import os
import glob
import shutil
import sys
from pathlib import Path

def run_test(num_runs=5):
    """Run the test multiple times and check for empty proof files"""
    empty_proof_counts = []
    
    for i in range(num_runs):
        print(f"\rRun {i+1}/{num_runs}", end="", flush=True)
        
        # Clean up previous output
        if os.path.exists("out/fred"):
            shutil.rmtree("out/fred")
        
        # Run the command
        try:
            result = subprocess.run(
                ["cargo", "run", "--", "examples/ser/fred.ser"],
                capture_output=True,
                text=True,
                timeout=30
            )
            
            if result.returncode != 0:
                print(f"\nError on run {i+1}: {result.stderr}")
                continue
                
        except subprocess.TimeoutExpired:
            print(f"\nTimeout on run {i+1}")
            continue
        
        # Check for empty proof files
        proof_files = glob.glob("out/fred/smpt_constraints_disjunct_*_proof.txt")
        empty_count = 0
        
        for proof_file in proof_files:
            size = os.path.getsize(proof_file)
            if size == 0:
                empty_count += 1
        
        empty_proof_counts.append((empty_count, len(proof_files)))
    
    print("\n\nResults:")
    print(f"Total runs: {len(empty_proof_counts)}")
    
    # Calculate statistics
    runs_with_empty = sum(1 for count, _ in empty_proof_counts if count > 0)
    total_empty = sum(count for count, _ in empty_proof_counts)
    total_proofs = sum(total for _, total in empty_proof_counts)
    
    print(f"Runs with at least one empty proof file: {runs_with_empty} ({runs_with_empty/len(empty_proof_counts)*100:.1f}%)")
    print(f"Total empty proof files: {total_empty} out of {total_proofs} ({total_empty/total_proofs*100:.1f}% if proofs > 0 else 0)")
    
    # Show distribution
    if runs_with_empty > 0:
        print("\nDistribution of empty proof files per run:")
        distribution = {}
        for count, total in empty_proof_counts:
            if count > 0:
                key = f"{count}/{total}"
                distribution[key] = distribution.get(key, 0) + 1
        
        for key, freq in sorted(distribution.items()):
            print(f"  {key} empty proofs: {freq} runs")
    
    return empty_proof_counts

def main():
    print("Testing for empty proof files in fred.ser")
    print("=" * 50)
    
    # Check if we're in the right directory
    if not os.path.exists("Cargo.toml"):
        print("Error: Must run from the SMPT directory (where Cargo.toml is located)")
        sys.exit(1)
    
    # First, make sure the project builds
    print("Building project...")
    result = subprocess.run(["cargo", "build"], capture_output=True)
    if result.returncode != 0:
        print("Error building project")
        sys.exit(1)
    
    # Run the test
    results = run_test(5)
    
    # Save results for comparison
    with open("test_results_with_patch.txt", "w") as f:
        f.write(f"Results with patch:\n")
        for i, (empty, total) in enumerate(results):
            f.write(f"Run {i+1}: {empty}/{total} empty\n")

if __name__ == "__main__":
    main()