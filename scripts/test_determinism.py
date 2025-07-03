#!/usr/bin/env python3
"""
Test script to check for nondeterminism in analyze_examples.py
Runs the analysis multiple times and compares the outputs.
"""

import os
import subprocess
import shutil
import difflib
import hashlib
from datetime import datetime
import argparse
from pathlib import Path


def run_analysis(run_num, output_dir, timeout=10):
    """Run analyze_examples.py and save the report."""
    print(f"Run {run_num}: Running analysis...")
    
    # Run the analysis
    cmd = ["python3", "scripts/analyze_examples.py", "--timeout", str(timeout)]
    try:
        result = subprocess.run(cmd, capture_output=True, text=True)
        if result.returncode != 0:
            print(f"  Error: {result.stderr}")
            return False
    except Exception as e:
        print(f"  Exception: {e}")
        return False
    
    # Copy the report to the output directory
    src_report = "out/serializability_report.md"
    if os.path.exists(src_report):
        dst_report = os.path.join(output_dir, f"report_{run_num:03d}.md")
        shutil.copy2(src_report, dst_report)
        print(f"  Report saved to {dst_report}")
        return True
    else:
        print(f"  Error: Report file {src_report} not found")
        return False


def compute_file_hash(filepath):
    """Compute SHA-256 hash of a file."""
    sha256_hash = hashlib.sha256()
    with open(filepath, "rb") as f:
        for byte_block in iter(lambda: f.read(4096), b""):
            sha256_hash.update(byte_block)
    return sha256_hash.hexdigest()


def normalize_report(content):
    """Normalize report content by removing timestamps and other variable elements."""
    lines = content.split('\n')
    normalized_lines = []
    
    for line in lines:
        # Skip lines that contain timestamps or dates
        if 'Generated at:' in line or 'Analysis timestamp:' in line:
            continue
        # Remove timing information (e.g., "0.123s" -> "Xs")
        # This regex-free approach looks for patterns like digit.digit+s
        if 's |' in line and any(c.isdigit() for c in line):
            # Simple normalization of timing values
            parts = line.split('|')
            for i, part in enumerate(parts):
                if 's' in part and any(c.isdigit() for c in part):
                    # Replace numeric values before 's' with 'X'
                    words = part.split()
                    for j, word in enumerate(words):
                        if word.endswith('s') and len(word) > 1:
                            if word[:-1].replace('.', '').isdigit():
                                words[j] = 'X.XXXs'
                    parts[i] = ' '.join(words)
            line = '|'.join(parts)
        normalized_lines.append(line)
    
    return '\n'.join(normalized_lines)


def compare_reports(report1_path, report2_path):
    """Compare two reports and return differences."""
    with open(report1_path, 'r') as f1:
        content1 = f1.read()
    with open(report2_path, 'r') as f2:
        content2 = f2.read()
    
    # First check if files are byte-identical
    if content1 == content2:
        return True, []
    
    # Normalize and compare
    norm1 = normalize_report(content1)
    norm2 = normalize_report(content2)
    
    if norm1 == norm2:
        return True, ["Files differ only in timestamps/timing"]
    
    # Generate detailed diff
    diff = list(difflib.unified_diff(
        norm1.splitlines(keepends=True),
        norm2.splitlines(keepends=True),
        fromfile=report1_path,
        tofile=report2_path,
        n=3
    ))
    
    return False, diff


def create_diff_matrix(output_dir, num_runs):
    """Create an NxN matrix showing differences between all pairs of runs."""
    print("\n=== Difference Matrix ===")
    print("(✓ = identical, ~ = timing only, ✗ = different)")
    
    # Print header
    print("     ", end="")
    for i in range(1, num_runs + 1):
        print(f" R{i:02d}", end="")
    print()
    
    # Compare all pairs
    diff_details = {}
    for i in range(1, num_runs + 1):
        print(f"R{i:02d}: ", end="")
        for j in range(1, num_runs + 1):
            if i == j:
                print("  - ", end="")
            else:
                report1 = os.path.join(output_dir, f"report_{i:03d}.md")
                report2 = os.path.join(output_dir, f"report_{j:03d}.md")
                
                if not os.path.exists(report1) or not os.path.exists(report2):
                    print("  ? ", end="")
                    continue
                
                identical, diff = compare_reports(report1, report2)
                if identical:
                    if diff:  # Timing differences only
                        print("  ~ ", end="")
                    else:  # Completely identical
                        print("  ✓ ", end="")
                else:
                    print("  ✗ ", end="")
                    diff_key = (min(i, j), max(i, j))
                    if diff_key not in diff_details:
                        diff_details[diff_key] = diff
        print()
    
    return diff_details


def save_diff_reports(output_dir, diff_details):
    """Save detailed diff reports for pairs that differ."""
    if not diff_details:
        print("\nNo significant differences found!")
        return
    
    diff_dir = os.path.join(output_dir, "diffs")
    os.makedirs(diff_dir, exist_ok=True)
    
    print(f"\n=== Detailed Differences ===")
    print(f"Saving {len(diff_details)} diff reports to {diff_dir}/")
    
    for (i, j), diff in diff_details.items():
        diff_file = os.path.join(diff_dir, f"diff_{i:03d}_vs_{j:03d}.txt")
        with open(diff_file, 'w') as f:
            f.writelines(diff)
        print(f"  Saved: diff_{i:03d}_vs_{j:03d}.txt")


def find_equivalence_classes(output_dir, num_runs):
    """Find equivalence classes of reports based on normalized content."""
    equivalence_classes = []
    assigned = set()
    
    for i in range(1, num_runs + 1):
        if i in assigned:
            continue
            
        report_i = os.path.join(output_dir, f"report_{i:03d}.md")
        if not os.path.exists(report_i):
            continue
            
        # Start new equivalence class with this report
        current_class = [i]
        assigned.add(i)
        
        # Find all reports equivalent to this one
        for j in range(i + 1, num_runs + 1):
            if j in assigned:
                continue
                
            report_j = os.path.join(output_dir, f"report_{j:03d}.md")
            if not os.path.exists(report_j):
                continue
                
            identical, _ = compare_reports(report_i, report_j)
            if identical:
                current_class.append(j)
                assigned.add(j)
        
        equivalence_classes.append(current_class)
    
    return equivalence_classes


def save_class_representatives(output_dir, equivalence_classes):
    """Save one representative from each equivalence class."""
    repr_dir = os.path.join(output_dir, "representatives")
    os.makedirs(repr_dir, exist_ok=True)
    
    for class_idx, members in enumerate(equivalence_classes):
        # Use the first member as representative
        repr_run = members[0]
        src_report = os.path.join(output_dir, f"report_{repr_run:03d}.md")
        dst_report = os.path.join(repr_dir, f"class_{class_idx + 1}_representative.md")
        
        if os.path.exists(src_report):
            shutil.copy2(src_report, dst_report)
            
            # Also create a summary file for this class
            summary_file = os.path.join(repr_dir, f"class_{class_idx + 1}_members.txt")
            with open(summary_file, 'w') as f:
                f.write(f"Equivalence Class {class_idx + 1}\n")
                f.write(f"Members: {len(members)} runs\n")
                f.write(f"Runs: {', '.join(f'R{r:02d}' for r in members)}\n")
                f.write(f"Representative: R{repr_run:02d} (report_{repr_run:03d}.md)\n")


def main():
    parser = argparse.ArgumentParser(description="Test analyze_examples.py for nondeterminism")
    parser.add_argument("-n", "--num-runs", type=int, default=5,
                        help="Number of times to run the analysis (default: 5)")
    parser.add_argument("-t", "--timeout", type=int, default=10,
                        help="Timeout for each analysis in seconds (default: 10)")
    parser.add_argument("-o", "--output-dir", type=str, 
                        default=f"determinism_test_{datetime.now().strftime('%Y%m%d_%H%M%S')}",
                        help="Output directory for reports")
    parser.add_argument("--keep-reports", action="store_true",
                        help="Keep the serializability_report.md file after each run")
    
    args = parser.parse_args()
    
    # Create output directory
    os.makedirs(args.output_dir, exist_ok=True)
    print(f"Output directory: {args.output_dir}")
    print(f"Running analysis {args.num_runs} times with timeout={args.timeout}s")
    print()
    
    # Run analyses
    successful_runs = 0
    for i in range(1, args.num_runs + 1):
        if run_analysis(i, args.output_dir, args.timeout):
            successful_runs += 1
        
        # Clean up unless requested to keep
        if not args.keep_reports and os.path.exists("serializability_report.md"):
            os.remove("serializability_report.md")
    
    print(f"\nSuccessful runs: {successful_runs}/{args.num_runs}")
    
    if successful_runs < 2:
        print("Not enough successful runs to compare.")
        return
    
    # Find equivalence classes
    print("\n=== Finding Equivalence Classes ===")
    equivalence_classes = find_equivalence_classes(args.output_dir, args.num_runs)
    
    print(f"Found {len(equivalence_classes)} equivalence class(es):")
    for idx, members in enumerate(equivalence_classes):
        print(f"  Class {idx + 1}: {len(members)} members - Runs {', '.join(f'{r}' for r in members)}")
    
    # Save representatives
    save_class_representatives(args.output_dir, equivalence_classes)
    
    # If more than one class, show differences between representatives
    if len(equivalence_classes) > 1:
        print("\n=== Differences Between Classes ===")
        repr_dir = os.path.join(args.output_dir, "representatives")
        diff_dir = os.path.join(args.output_dir, "class_diffs")
        os.makedirs(diff_dir, exist_ok=True)
        
        for i in range(len(equivalence_classes)):
            for j in range(i + 1, len(equivalence_classes)):
                repr_i = os.path.join(repr_dir, f"class_{i + 1}_representative.md")
                repr_j = os.path.join(repr_dir, f"class_{j + 1}_representative.md")
                
                _, diff = compare_reports(repr_i, repr_j)
                
                diff_file = os.path.join(diff_dir, f"class_{i + 1}_vs_class_{j + 1}.diff")
                with open(diff_file, 'w') as f:
                    f.writelines(diff)
                print(f"  Saved diff: class_{i + 1}_vs_class_{j + 1}.diff")
    
    # Still create the full matrix for detailed analysis
    print("\n=== Full Difference Matrix ===")
    diff_details = create_diff_matrix(args.output_dir, args.num_runs)
    
    # Summary
    print("\n=== Summary ===")
    if len(equivalence_classes) == 1:
        print("✅ All runs produced identical results (ignoring timestamps).")
        print("   The analysis appears to be deterministic!")
    else:
        print(f"⚠️  Found {len(equivalence_classes)} different equivalence classes.")
        print("   Check the class representatives and diffs for details.")
        
        # Show hash summary per class
        print("\n=== Hashes by Equivalence Class ===")
        for idx, members in enumerate(equivalence_classes):
            print(f"Class {idx + 1}:")
            for run in members[:3]:  # Show first 3 members
                report_path = os.path.join(args.output_dir, f"report_{run:03d}.md")
                if os.path.exists(report_path):
                    hash_val = compute_file_hash(report_path)
                    print(f"  Run {run}: {hash_val[:16]}...")
            if len(members) > 3:
                print(f"  ... and {len(members) - 3} more")


if __name__ == "__main__":
    main()