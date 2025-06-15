#!/usr/bin/env python3
"""
Script to analyze all .ser examples and generate a serializability report (parallelized)
Usage: python3 analyze_examples.py [--timeout <seconds>] [--jobs <number>]
"""

import argparse
import subprocess
import time
import os
import re
from pathlib import Path
from concurrent.futures import ThreadPoolExecutor, as_completed
from datetime import datetime


def parse_time_output(stderr_output):
    """Parse time command output to extract user and sys time."""
    lines = stderr_output.strip().split('\n')
    
    user_time = 0.0
    sys_time = 0.0
    
    for line in lines:
        if 'user' in line:
            # Extract time from formats like "user    0m0.123s" or "0.123"
            match = re.search(r'(\d+)m([\d.]+)s', line)
            if match:
                minutes = int(match.group(1))
                seconds = float(match.group(2))
                user_time = minutes * 60 + seconds
            else:
                # Try to find just seconds
                match = re.search(r'([\d.]+)', line)
                if match:
                    user_time = float(match.group(1))
        
        elif 'sys' in line:
            # Same parsing for sys time
            match = re.search(r'(\d+)m([\d.]+)s', line)
            if match:
                minutes = int(match.group(1))
                seconds = float(match.group(2))
                sys_time = minutes * 60 + seconds
            else:
                match = re.search(r'([\d.]+)', line)
                if match:
                    sys_time = float(match.group(1))
    
    return user_time + sys_time


def run_single_analysis(file_path, timeout_arg, with_optimizations=True):
    """Run a single analysis (with or without optimizations) and return timing and status."""
    # Build command
    cmd = ['cargo', 'run', '--quiet', '--']
    if timeout_arg:
        cmd.extend(['--timeout', str(timeout_arg)])
    if not with_optimizations:
        cmd.append('--without-optimizations')
    cmd.append(str(file_path))
    
    # Use time command to get CPU timing
    time_cmd = ['time'] + cmd
    
    start_time = time.time()
    result = subprocess.run(
        time_cmd,
        capture_output=True,
        text=True,
        timeout=timeout_arg * 2 if timeout_arg else None  # Give extra time for the timeout to work
    )
    
    # Parse CPU time from stderr (time command output)
    cpu_time = parse_time_output(result.stderr)
    if cpu_time == 0.0:
        # Fallback to wall clock time if time parsing failed
        cpu_time = time.time() - start_time
    
    # Extract program output (stdout and stderr combined)
    output = result.stdout + result.stderr
    
    # Check for SMPT timeout in the output - check multiple patterns
    is_smpt_timeout = (
        "SMPT timeout:" in output or 
        "SMPT verification failed: SMPT timeout:" in output or
        "Failed to run SMPT: SMPT timeout:" in output or
        "Analysis timed out" in output
    )
    
    # Determine results for each method separately
    original_result = "Unknown"
    proof_result = "Unknown"
    
    if result.returncode == 0:
        # Check original method results
        if "Original method: Serializable" in output:
            original_result = "Serializable"
        elif "Original method: Not serializable" in output:
            original_result = "Not serializable"
        
        # Check proof-based method results
        if "Proof-based method: Proof" in output:
            proof_result = "Serializable"
        elif "Proof-based method: CounterExample" in output:
            proof_result = "Not serializable"
    else:
        # Process failed - check if due to timeout
        if is_smpt_timeout:
            original_result = "SMPT Timeout"
            proof_result = "SMPT Timeout"
        else:
            original_result = "Error"
            proof_result = "Error"
    
    # Create combined status for backward compatibility
    if original_result == proof_result:
        if original_result == "Serializable":
            status = "✅ Serializable"
            console_status = "Serializable"
        elif original_result == "Not serializable":
            status = "❌ Not serializable"
            console_status = "Not serializable"
        elif original_result == "SMPT Timeout":
            status = "⏱️ SMPT Timeout"
            console_status = "SMPT Timeout"
        elif original_result == "Error":
            status = "⚠️ Error"
            console_status = "Error"
        else:
            status = "❓ Unknown"
            console_status = "Unknown"
    else:
        # Methods disagree
        status = f"⚠️ Inconsistent (orig: {original_result}, proof: {proof_result})"
        console_status = "Inconsistent"
    
    return {
        'status': status,
        'console_status': console_status,
        'original_result': original_result,
        'proof_result': proof_result,
        'cpu_time': cpu_time,
        'returncode': result.returncode,
        'is_timeout': is_smpt_timeout
    }


def analyze_file(file_path, timeout_arg, index):
    """Analyze a single .ser file twice (with and without optimizations) and return results."""
    filename = Path(file_path).stem
    
    try:
        # Run with optimizations (default)
        print(f"[{index}] {filename}: Running with optimizations...")
        opt_result = run_single_analysis(file_path, timeout_arg, with_optimizations=True)
        
        # Run without optimizations
        print(f"[{index}] {filename}: Running without optimizations...")
        no_opt_result = run_single_analysis(file_path, timeout_arg, with_optimizations=False)
        
        # Use the optimized result for the main status (they should be the same)
        status = opt_result['status']
        console_status = opt_result['console_status']
        
        # Check if results are consistent
        if opt_result['status'] != no_opt_result['status']:
            # Only accept inconsistency if one or both runs had SMPT timeouts
            if opt_result.get('is_timeout', False) or no_opt_result.get('is_timeout', False):
                print(f"[{index}] {filename}: Results differ due to SMPT timeout")
                # Use the non-timeout result if one succeeded
                if opt_result.get('is_timeout', False) and not no_opt_result.get('is_timeout', False):
                    status = no_opt_result['status'] + " (opt timed out)"
                    console_status = no_opt_result['console_status'] + " (opt timed out)"
                elif no_opt_result.get('is_timeout', False) and not opt_result.get('is_timeout', False):
                    status = opt_result['status'] + " (no-opt timed out)"
                    console_status = opt_result['console_status'] + " (no-opt timed out)"
                else:
                    # Both timed out
                    status = "⏱️ Both Timed Out"
                    console_status = "Both Timed Out"
            else:
                # Real inconsistency - this is a serious problem!
                print(f"[{index}] {filename}: WARNING - Results differ between optimized and non-optimized runs (not due to timeout)!")
                print(f"                    Optimized: {opt_result['status']}, No-opt: {no_opt_result['status']}")
                status = "⚠️ Inconsistent"
                console_status = "Inconsistent"
        
        # Format durations
        opt_duration_str = f"{opt_result['cpu_time']:.2f}"
        no_opt_duration_str = f"{no_opt_result['cpu_time']:.2f}"
        
        print(f"[{index}] {filename}: {console_status} (opt: {opt_duration_str}s, no-opt: {no_opt_duration_str}s CPU)")
        
        return {
            'filename': filename,
            'status': status,
            'opt_duration': opt_duration_str,
            'no_opt_duration': no_opt_duration_str,
            'opt_original_result': opt_result['original_result'],
            'opt_proof_result': opt_result['proof_result'],
            'no_opt_original_result': no_opt_result['original_result'],
            'no_opt_proof_result': no_opt_result['proof_result'],
            'index': index
        }
        
    except subprocess.TimeoutExpired:
        print(f"[{index}] {filename}: Timeout")
        return {
            'filename': filename,
            'status': "⚠️ Timeout",
            'opt_duration': "N/A",
            'no_opt_duration': "N/A",
            'index': index
        }
    except Exception as e:
        print(f"[{index}] {filename}: Error ({e})")
        return {
            'filename': filename,
            'status': "⚠️ Error",
            'opt_duration': "N/A",
            'no_opt_duration': "N/A",
            'index': index
        }


def main():
    parser = argparse.ArgumentParser(
        description="Analyze all .ser examples and generate a serializability report"
    )
    parser.add_argument('--timeout', type=int, help='Timeout in seconds for each analysis')
    parser.add_argument('--jobs', type=int, help='Number of parallel jobs')
    
    args = parser.parse_args()
    
    # Set defaults
    timeout_value = args.timeout
    max_jobs = args.jobs or os.cpu_count() or 4
    
    print("🔍 Analyzing Serializability of .ser Examples (Parallel)")
    print("======================================================")
    print(f"Using {max_jobs} parallel jobs")
    if timeout_value:
        print(f"Timeout: {timeout_value}s")
    else:
        print("Timeout: none")
    print()
    
    # Find all .ser files
    ser_files = sorted(Path('examples/ser').glob('*.ser'))
    total_files = len(ser_files)
    
    print(f"Found {total_files} .ser files to analyze")
    print()
    
    # Create output file
    output_file = "serializability_report.md"
    
    # Analyze files in parallel
    results = []
    with ThreadPoolExecutor(max_workers=max_jobs) as executor:
        # Submit all jobs
        future_to_index = {
            executor.submit(analyze_file, file_path, timeout_value, i): i
            for i, file_path in enumerate(ser_files)
        }
        
        # Collect results as they complete
        for future in as_completed(future_to_index):
            result = future.result()
            results.append(result)
    
    print()
    print("🔄 Collecting results...")
    
    # Sort results by original index to maintain order
    results.sort(key=lambda x: x['index'])
    
    # Generate report
    with open(output_file, 'w') as f:
        f.write("# Serializability Analysis Report\n\n")
        f.write("This report shows the serializability analysis results for all `.ser` examples using both original and proof-based methods, with and without optimizations.\n\n")
        f.write("**Analysis Configuration:**\n")
        f.write(f"- Parallel jobs: {max_jobs}\n")
        f.write(f"- Timeout: {timeout_value}s\n" if timeout_value else "- Timeout: none\n")
        f.write(f"- Generated: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}\n\n")
        f.write("## Results\n\n")
        f.write("| Example | Opt Original | Opt Proof | No-Opt Original | No-Opt Proof | Opt CPU (s) | No-Opt CPU (s) |\n")
        f.write("|---------|--------------|-----------|-----------------|--------------|-------------|---------------|\n")
        
        for result in results:
            # Get method results for all combinations
            opt_original = result.get('opt_original_result', 'Unknown')
            opt_proof = result.get('opt_proof_result', 'Unknown') 
            no_opt_original = result.get('no_opt_original_result', 'Unknown')
            no_opt_proof = result.get('no_opt_proof_result', 'Unknown')
            
            f.write(f"| `{result['filename']}` | {opt_original} | {opt_proof} | {no_opt_original} | {no_opt_proof} | {result['opt_duration']} | {result['no_opt_duration']} |\n")
        
        f.write("\n## Summary\n\n")
        f.write("- ✅ **Serializable**: Programs that maintain serializability properties\n")
        f.write("- ❌ **Not serializable**: Programs that violate serializability\n")
        f.write("- ❓ **Unknown**: Could not determine result\n")
        f.write("- ⚠️ **Error**: Analysis failed\n")
        f.write("- ⏱️ **SMPT Timeout**: SMPT verification timed out\n")
        f.write("- ⚠️ **Inconsistent**: Results differ between optimized and non-optimized runs (serious issue)\n\n")
        f.write("**Note**: Each example is analyzed twice - once with optimizations (default) and once with `--without-optimizations` flag. The table shows results for all four combinations: Optimized Original/Proof methods and Non-optimized Original/Proof methods. CPU times compare performance impact of optimizations.\n\n")
        f.write("---\n\n")
        f.write("*Report generated automatically by analyze_examples.py*\n")
    
    print()
    print("✅ Analysis complete!")
    print(f"📊 Results saved to: {output_file}")
    print()
    
    # Show summary
    serializable_count = sum(1 for r in results if "✅ Serializable" in r['status'])
    not_serializable_count = sum(1 for r in results if "❌ Not serializable" in r['status'])
    unknown_count = sum(1 for r in results if "❓ Unknown" in r['status'])
    timeout_count = sum(1 for r in results if "⏱️" in r['status'] or "timed out" in r['status'].lower())
    inconsistent_count = sum(1 for r in results if "⚠️ Inconsistent" in r['status'])
    error_count = sum(1 for r in results if "⚠️" in r['status'] and "Inconsistent" not in r['status'] and "⏱️" not in r['status'])
    
    print("📈 Summary:")
    print(f"   Serializable: {serializable_count}")
    print(f"   Not serializable: {not_serializable_count}")
    print(f"   Unknown: {unknown_count}")
    print(f"   SMPT Timeouts: {timeout_count}")
    print(f"   Inconsistent: {inconsistent_count}")
    print(f"   Errors: {error_count}")
    print(f"   Total: {total_files}")
    print()
    print(f"🔗 View the full report: cat {output_file}")


if __name__ == "__main__":
    main()