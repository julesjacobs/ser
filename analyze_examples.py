#!/usr/bin/env python3
"""
Script to analyze all .ser examples and generate a serializability report (optimized only)
Usage: python3 analyze_examples.py [--timeout <seconds>] [--jobs <number>] [--use-cache]
                                  [--without-remove-redundant] [--without-generate-less]
                                  [--without-smart-kleene-order] [--without-bidirectional]
                                  [--<other_flags>]
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
    user_time = sys_time = 0.0
    for line in lines:
        if 'user' in line:
            m = re.search(r'(\d+)m([\d.]+)s', line)
            if m:
                user_time = int(m.group(1)) * 60 + float(m.group(2))
            else:
                m = re.search(r'([\d.]+)', line)
                if m:
                    user_time = float(m.group(1))
        elif 'sys' in line:
            m = re.search(r'(\d+)m([\d.]+)s', line)
            if m:
                sys_time = int(m.group(1)) * 60 + float(m.group(2))
            else:
                m = re.search(r'([\d.]+)', line)
                if m:
                    sys_time = float(m.group(1))
    return user_time + sys_time


def run_single_analysis(file_path, timeout_arg, extra_flags, use_cache=False):
    """Run a single optimized analysis with optional extra flags."""
    cmd = ['cargo', 'run', '--quiet', '--'] + extra_flags
    if timeout_arg:
        cmd += ['--timeout', str(timeout_arg)]
    if use_cache:
        cmd.append('--use-cache')
    cmd.append(str(file_path))

    time_cmd = ['time'] + cmd
    start = time.time()
    try:
        result = subprocess.run(
            time_cmd,
            capture_output=True,
            text=True,
            timeout=(timeout_arg * 2) if timeout_arg else None
        )
    except subprocess.TimeoutExpired:
        return {
            'status': '⏱️ SMPT Timeout',
            'original_result': 'SMPT Timeout',
            'proof_result': 'SMPT Timeout',
            'trace_valid': None,
            'proof_verification': None,
            'cpu_time': 0.0,
            'is_timeout': True
        }

    cpu = parse_time_output(result.stderr)
    if cpu == 0.0:
        cpu = time.time() - start

    out = result.stdout + result.stderr
    timeout_flag = 'SMPT timeout:' in out or 'Analysis timed out' in out

    orig = 'Unknown'
    proof = 'Unknown'
    trace_valid = None
    proof_valid = None
    if result.returncode == 0:
        if 'Original method: Serializable' in out:
            orig = 'Serializable'
        elif 'Original method: Not serializable' in out:
            orig = 'Not serializable'
        if 'Proof-based method: Proof' in out:
            proof = 'Serializable'
            proof_valid = '✅ Proof certificate is VALID' in out
        elif 'Proof-based method: CounterExample' in out:
            proof = 'Not serializable'
            trace_valid = '✅ Trace is valid!' in out
    else:
        if timeout_flag:
            orig = proof = 'SMPT Timeout'
        else:
            orig = proof = 'Error'

    if orig == proof:
        if orig == 'Serializable':
            status = '✅ Serializable'
        elif orig == 'Not serializable':
            status = '❌ Not serializable'
        elif orig == 'SMPT Timeout':
            status = '⏱️ SMPT Timeout'
        else:
            status = '⚠️ Error'
    else:
        status = '⚠️ Error'

    return {
        'status': status,
        'original_result': orig,
        'proof_result': proof,
        'trace_valid': trace_valid,
        'proof_verification': proof_valid,
        'cpu_time': cpu,
        'is_timeout': timeout_flag
    }


def analyze_file(fp, timeout, idx, extra_flags, use_cache):
    name = Path(fp).stem
    print(f"[{idx}] `{name}`: Running optimized analysis...")
    res = run_single_analysis(fp, timeout, extra_flags, use_cache)
    dur = f"{res['cpu_time']:.2f}"
    print(f"[{idx}] `{name}`: {res['status']} ({dur}s CPU)")
    res.update({'filename': name, 'duration': dur, 'index': idx})
    return res


def main():
    parser = argparse.ArgumentParser(
        description="Analyze .ser examples with optional cargo flags (optimized only)",
        allow_abbrev=False
    )
    parser.add_argument('--timeout', type=int, help='Timeout seconds')
    parser.add_argument('--jobs', type=int, help='Parallel jobs')
    parser.add_argument('--use-cache', action='store_true', help='Enable caching')
    parser.add_argument('--without-remove-redundant', action='store_true', help='Disable redundant removal')
    parser.add_argument('--without-generate-less', action='store_true', help='Disable generate-less optimization')
    parser.add_argument('--without-smart-kleene-order', action='store_true', help='Disable smart Kleene ordering')
    parser.add_argument('--without-bidirectional', action='store_true',
                        help='Disable bidirectional optimization')  # <— new
    known_args, extra = parser.parse_known_args()

    timeout = known_args.timeout
    jobs = known_args.jobs or os.cpu_count() or 4
    cache = known_args.use_cache
    # collect flags to forward
    extras = []
    if known_args.without_remove_redundant:
        extras.append('--without-remove-redundant')
    if known_args.without_generate_less:
        extras.append('--without-generate-less')
    if known_args.without_smart_kleene_order:
        extras.append('--without-smart-kleene-order')
    if known_args.without_bidirectional:
        extras.append('--without-bidirectional')            # <— new
    # append other unknown flags
    extras.extend(extra)

    print(f"🔍 Analyzing (.ser) with {jobs} jobs, timeout={timeout or 'none'}, "
          f"cache={'on' if cache else 'off'}, extras={extras}")
    files = sorted(Path('examples/ser').glob('*.ser'))
    print(f"Found {len(files)} examples")

    results = []
    with ThreadPoolExecutor(max_workers=jobs) as ex:
        futures = {ex.submit(analyze_file, fp, timeout, i, extras, cache): i for i, fp in enumerate(files)}
        for fut in as_completed(futures):
            results.append(fut.result())
    results.sort(key=lambda r: r['index'])

    # Print non-validated cases
    nv_proofs = [r['filename'] for r in results
                 if r['original_result'] == 'Serializable'
                 and r['proof_result'] == 'Serializable'
                 and r['proof_verification'] is False]
    nv_traces = [r['filename'] for r in results
                 if r['original_result'] == 'Not serializable'
                 and r['proof_result'] == 'Not serializable'
                 and r['trace_valid'] is False]
    if nv_proofs:
        print('❌ Non-validated proofs for:', ', '.join(nv_proofs))
    if nv_traces:
        print('❌ Non-validated traces for:', ', '.join(nv_traces))

    out_md = 'serializability_report.md'
    with open(out_md, 'w') as f:
        f.write(f"# Serializability Analysis Report\n"
                f"Generated: {datetime.now():%Y-%m-%d %H:%M:%S}\n"
                f"Extras: {extras}\n\n")
        f.write("|Example|Orig|Proof|CPU(s)|Trace|Proof Cert|\n|--|--|--|--|--|--|\n")
        for r in results:
            t = 'N/A' if r['trace_valid'] is None else ('✅' if r['trace_valid'] else '❌')
            p = 'N/A' if r['proof_verification'] is None else ('✅' if r['proof_verification'] else '❌')
            # wrap filename in backticks for nicer Markdown
            f.write(f"| `{r['filename']}` |{r['original_result']}|{r['proof_result']}|"
                    f"{r['duration']}|{t}|{p}|\n")

        # summary counts
        s_cnt = sum(r['original_result'] == 'Serializable' and r['proof_result'] == 'Serializable'
                    for r in results)
        vp = sum(r['proof_verification']
                 for r in results
                 if r['original_result'] == 'Serializable'
                 and r['proof_result'] == 'Serializable')
        ip = s_cnt - vp
        ns_cnt = sum(r['original_result'] == 'Not serializable' and r['proof_result'] == 'Not serializable'
                     for r in results)
        vt = sum(r['trace_valid']
                 for r in results
                 if r['original_result'] == 'Not serializable'
                 and r['proof_result'] == 'Not serializable')
        it = ns_cnt - vt
        to_cnt = sum(r['status'] == '⏱️ SMPT Timeout' for r in results)
        err = sum(r['status'] == '⚠️ Error' for r in results)
        f.write("\n## Summary\n")
        f.write(f"- Serializable: {s_cnt} (valid proofs: {vp}, invalid: {ip})\n")
        f.write(f"- Not serializable: {ns_cnt} (valid traces: {vt}, invalid: {it})\n")
        f.write(f"- Timeouts: {to_cnt}, Errors: {err}, Total: {len(results)}\n")

    print("✅ Done. Report:", out_md)


if __name__ == '__main__':
    main()
