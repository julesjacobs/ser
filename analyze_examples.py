#!/usr/bin/env python3
"""
Script to analyze all .ser examples and generate a serializability report (optimized only)
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
            match = re.search(r'(\d+)m([\d.]+)s', line)
            if match:
                minutes = int(match.group(1))
                seconds = float(match.group(2))
                user_time = minutes * 60 + seconds
            else:
                match = re.search(r'([\d.]+)', line)
                if match:
                    user_time = float(match.group(1))
        elif 'sys' in line:
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


def run_single_analysis(file_path, timeout_arg, use_cache=False):
    """Run a single optimized analysis and return timing and status."""
    cmd = ['cargo', 'run', '--quiet', '--']
    if timeout_arg:
        cmd.extend(['--timeout', str(timeout_arg)])
    if use_cache:
        cmd.append('--use-cache')
    cmd.append(str(file_path))

    time_cmd = ['time'] + cmd
    start_time = time.time()

    try:
        result = subprocess.run(
            time_cmd,
            capture_output=True,
            text=True,
            timeout=(timeout_arg * 2) if timeout_arg else None
        )
    except subprocess.TimeoutExpired:
        return {
            'status': "⏱️ SMPT Timeout",
            'original_result': "SMPT Timeout",
            'proof_result': "SMPT Timeout",
            'trace_valid': None,
            'proof_verification': None,
            'cpu_time': 0.0,
            'is_timeout': True
        }

    cpu_time = parse_time_output(result.stderr)
    if cpu_time == 0.0:
        cpu_time = time.time() - start_time

    output = result.stdout + result.stderr
    is_smpt_timeout = "SMPT timeout:" in output or "Analysis timed out" in output

    original_result = "Unknown"
    proof_result = "Unknown"
    trace_valid = None
    proof_verification = None

    if result.returncode == 0:
        if "Original method: Serializable" in output:
            original_result = "Serializable"
        elif "Original method: Not serializable" in output:
            original_result = "Not serializable"

        if "Proof-based method: Proof" in output:
            proof_result = "Serializable"
            proof_verification = "✅ Proof certificate is VALID" in output
        elif "Proof-based method: CounterExample" in output:
            proof_result = "Not serializable"
            trace_valid = "✅ Trace is valid!" in output
    else:
        if is_smpt_timeout:
            original_result = proof_result = "SMPT Timeout"
        else:
            original_result = proof_result = "Error"

    if original_result == proof_result:
        if original_result == "Serializable":
            status = "✅ Serializable"
        elif original_result == "Not serializable":
            status = "❌ Not serializable"
        elif original_result == "SMPT Timeout":
            status = "⏱️ SMPT Timeout"
        else:
            status = "⚠️ Error" if original_result == "Error" else "❓ Unknown"
    else:
        status = "⚠️ Error"

    return {
        'status': status,
        'original_result': original_result,
        'proof_result': proof_result,
        'trace_valid': trace_valid,
        'proof_verification': proof_verification,
        'cpu_time': cpu_time,
        'is_timeout': is_smpt_timeout
    }


def analyze_file(file_path, timeout_arg, index, use_cache=False):
    filename = Path(file_path).stem
    print(f"[{index}] {filename}: Running optimized analysis...")
    result = run_single_analysis(file_path, timeout_arg, use_cache)
    duration_str = f"{result['cpu_time']:.2f}"
    print(f"[{index}] {filename}: {result['status']} ({duration_str}s CPU)")
    return {
        'filename': filename,
        'status': result['status'],
        'original_result': result['original_result'],
        'proof_result': result['proof_result'],
        'trace_valid': result['trace_valid'],
        'proof_verification': result['proof_verification'],
        'duration': duration_str,
        'index': index
    }


def main():
    parser = argparse.ArgumentParser(
        description="Analyze all .ser examples and generate a serializability report (optimized only)"
    )
    parser.add_argument('--timeout', type=int, help='Timeout in seconds for each analysis')
    parser.add_argument('--jobs', type=int, help='Number of parallel jobs')
    parser.add_argument('--use-cache', action='store_true', help='Enable SMPT result caching')
    args = parser.parse_args()

    timeout_value = args.timeout
    max_jobs = args.jobs or os.cpu_count() or 4

    print("🔍 Analyzing Serializability of .ser Examples (Optimized Only)")
    print("==============================================================")
    print(f"Using {max_jobs} parallel jobs")
    print(f"Timeout: {timeout_value}s" if timeout_value else "Timeout: none")
    print()

    ser_files = sorted(Path('examples/ser').glob('*.ser'))
    total_files = len(ser_files)
    print(f"Found {total_files} .ser files to analyze")
    print()

    output_file = "serializability_report.md"
    results = []
    with ThreadPoolExecutor(max_workers=max_jobs) as executor:
        futures = {executor.submit(analyze_file, fp, timeout_value, i, args.use_cache): i for i, fp in enumerate(ser_files)}
        for future in as_completed(futures):
            results.append(future.result())

    print()
    print("🔄 Collecting results...")
    results.sort(key=lambda x: x['index'])

    # Print non-validated cases
    non_valid_proofs = [r['filename'] for r in results if r['original_result'] == "Serializable" and r['proof_result'] == "Serializable" and r['proof_verification'] is False]
    non_valid_traces = [r['filename'] for r in results if r['original_result'] == "Not serializable" and r['proof_result'] == "Not serializable" and r['trace_valid'] is False]
    if non_valid_proofs:
        print("❌ Non-validated proof certificates for:", ", ".join(non_valid_proofs))
    if non_valid_traces:
        print("❌ Non-validated counterexample traces for:", ", ".join(non_valid_traces))

    with open(output_file, 'w') as f:
        f.write("# Serializability Analysis Report\n\n")
        f.write("Results for all `.ser` examples analyzed with optimizations only.\n\n")
        f.write("**Configuration:**\n")
        f.write(f"- Parallel jobs: {max_jobs}\n")
        f.write(f"- Timeout: {timeout_value}s\n" if timeout_value else "- Timeout: none\n")
        f.write(f"- SMPT caching: {'enabled' if args.use_cache else 'disabled'}\n")
        f.write(f"- Generated: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}\n\n")
        f.write("## Results\n\n")
        f.write("| Example | Original | Proof | CPU (s) | Trace Valid | Proof Valid |\n")
        f.write("|---------|----------|-------|---------|-------------|-------------|\n")
        for r in results:
            trace = "N/A" if r['trace_valid'] is None else ("✅" if r['trace_valid'] else "❌")
            proof = "N/A" if r['proof_verification'] is None else ("✅" if r['proof_verification'] else "❌")
            f.write(f"| `{r['filename']}` | {r['original_result']} | {r['proof_result']} | {r['duration']} | {trace} | {proof} |\n")

        # Summary counts
        serializable_count = sum(1 for r in results if r['original_result'] == "Serializable" and r['proof_result'] == "Serializable")
        valid_proofs = sum(1 for r in results if r['original_result'] == "Serializable" and r['proof_result'] == "Serializable" and r['proof_verification'])
        invalid_proofs = sum(1 for r in results if r['original_result'] == "Serializable" and r['proof_result'] == "Serializable" and not r['proof_verification'])
        not_serializable_count = sum(1 for r in results if r['original_result'] == "Not serializable" and r['proof_result'] == "Not serializable")
        valid_traces = sum(1 for r in results if r['original_result'] == "Not serializable" and r['proof_result'] == "Not serializable" and r['trace_valid'])
        invalid_traces = sum(1 for r in results if r['original_result'] == "Not serializable" and r['proof_result'] == "Not serializable" and not r['trace_valid'])
        unknown_count = sum(1 for r in results if r['original_result'] == "Unknown" or r['proof_result'] == "Unknown")
        timeout_count = sum(1 for r in results if r['status'] == "⏱️ SMPT Timeout")
        error_count = sum(1 for r in results if r['status'] == "⚠️ Error")

        f.write("\n## Summary\n\n")
        f.write(f"- **Serializable**: {serializable_count}\n")
        if serializable_count:
            f.write(f"  - Valid proofs: {valid_proofs}\n")
            f.write(f"  - Invalid proofs: {invalid_proofs}\n")
        f.write(f"- **Not serializable**: {not_serializable_count}\n")
        if not_serializable_count:
            f.write(f"  - Valid traces: {valid_traces}\n")
            f.write(f"  - Invalid traces: {invalid_traces}\n")
        f.write(f"- **Unknown**: {unknown_count}\n")
        f.write(f"- **SMPT Timeouts**: {timeout_count}\n")
        f.write(f"- **Errors**: {error_count}\n")
        f.write(f"- **Total**: {total_files}\n\n")

        f.write("## Legend\n\n")
        f.write("- ✅ Serializable\n")
        f.write("- ❌ Not serializable\n")
        f.write("- ❓ Unknown\n")
        f.write("- ⚠️ Error\n")
        f.write("- ⏱️ SMPT Timeout\n\n")
        f.write("- **Trace Valid**: ✅ valid, ❌ invalid, N/A\n")
        f.write("- **Proof Valid**: ✅ valid, ❌ invalid, N/A\n\n")
        f.write("*Report generated by analyze_examples.py*\n")

    print()
    print("✅ Analysis complete!")
    if non_valid_proofs:
        print("❌ Non-validated proof certificates for:", ", ".join(non_valid_proofs))
    if non_valid_traces:
        print("❌ Non-validated counterexample traces for:", ", ".join(non_valid_traces))
    print(f"📊 Results saved to: {output_file}")
    print()
    print(f"🔗 View the report: cat {output_file}")


if __name__ == "__main__":
    main()
