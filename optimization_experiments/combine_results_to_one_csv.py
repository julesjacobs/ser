#!/usr/bin/env python3
import os
import re
import pandas as pd

# --- Configuration (hard-coded) ---
input_dir = '/home/guyamir/RustroverProjects/ser/optimization_experiments/csvs/raw_inputs'
output_file = '/home/guyamir/RustroverProjects/ser/optimization_experiments/csvs/combined_outputs/combined_results.csv'

# List of all optimizations
all_flags = [
    'bidirectional',
    'remove-redundant',
    'generate-less',
    'smart-kleene-order',
]

# Regex patterns to extract timeout (seconds) and file type
re_timeout = re.compile(r'timeout_(\d+)_seconds', re.IGNORECASE)
re_type = re.compile(r'_(json|ser)\.csv$', re.IGNORECASE)

combined_rows = []

# Iterate over all CSV files in the input directory
for fname in os.listdir(input_dir):
    if not fname.lower().endswith('.csv'):
        continue
    path = os.path.join(input_dir, fname)

    # Extract timeout in seconds, convert to ms
    tm_match = re_timeout.search(fname)
    if not tm_match:
        continue
    timeout_s = int(tm_match.group(1))
    timeout_ms = timeout_s * 1000

    # Extract file type
    ft_match = re_type.search(fname)
    if not ft_match:
        continue
    file_type = ft_match.group(1).lower()

    # Read this CSV
    df = pd.read_csv(path)
    for _, row in df.iterrows():
        mode = str(row['mode']).strip().lower()
        benchmark = row['example']
        time_ms = row['elapsed_ms']

        # Determine which flags are ON
        if mode == 'all-on':
            on_flags = set(all_flags)
        else:
            tokens = mode.split()
            off_flags = {tok.replace('without-', '') for tok in tokens if tok.startswith('without-')}
            on_flags = {flag for flag in all_flags if flag not in off_flags}

        # Build the record
        record = {f"{flag} ON": (1 if flag in on_flags else 0) for flag in all_flags}
        record.update({
            'benchmark': benchmark,
            'type': file_type,
            'timeout_ms': timeout_ms,
            'time_ms': time_ms,
            'terminated': 1 if pd.to_numeric(time_ms, errors='coerce') < timeout_ms else 0
        })
        combined_rows.append(record)

# Create combined DataFrame and write out
combined_df = pd.DataFrame(combined_rows)
combined_df.to_csv(output_file, index=False)
print(f"Combined results written to {output_file}")
