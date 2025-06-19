#!/usr/bin/env python3
import os
import re
import pandas as pd

def combine_results():
    # Hard-coded paths
    input_dir = '/home/guyamir/RustroverProjects/ser/csvs/raw_inputs'  # directory containing your CSVs
    output_file = '/home/guyamir/RustroverProjects/ser/csvs/combined_outputs/combined_results.csv'

    all_flags = [
        "bidirectional",
        "remove-redundant-parts",
        "remove-redundant-sets",
        "generate-less",
        "smart-kleene-order",
    ]

    # Regex to extract timeout seconds and file type
    re_timeout = re.compile(r"timeout_(\d+)_seconds", re.IGNORECASE)
    re_type    = re.compile(r"_(json|ser)\.csv$", re.IGNORECASE)

    combined_rows = []

    for fname in os.listdir(input_dir):
        if not fname.endswith(".csv"):
            continue

        # Extract timeout (in seconds) from filename
        tm_match = re_timeout.search(fname)
        if not tm_match:
            continue
        timeout_s = int(tm_match.group(1))
        timeout_ms = timeout_s * 1000

        # Extract file type (json or ser)
        ft_match = re_type.search(fname)
        if not ft_match:
            continue
        file_type = ft_match.group(1).lower()

        # Read the CSV
        path = os.path.join(input_dir, fname)
        df = pd.read_csv(path)

        for _, row in df.iterrows():
            mode = str(row["mode"])
            benchmark = row["example"]
            elapsed_raw = row["elapsed_ms"]

            # Convert elapsed to numeric if possible
            try:
                elapsed_val = float(elapsed_raw)
            except ValueError:
                elapsed_val = None

            # Determine disabled flags
            if mode.lower() == "regular":
                disabled = []
            else:
                disabled = mode.split("+")

            # Build output record
            out = {f"{flag} ON": (0 if flag in disabled else 1) for flag in all_flags}
            out.update({
                "benchmark": benchmark,
                "file_type": file_type,
                "timeout_ms": timeout_ms,
                "time_ms": elapsed_val if elapsed_val is not None else elapsed_raw,
                "terminated": 1 if (elapsed_val is not None and elapsed_val < timeout_ms) else 0
            })
            combined_rows.append(out)

    # Create DataFrame and write CSV
    combined_df = pd.DataFrame(combined_rows)
    combined_df.to_csv(output_file, index=False)
    print(f"Combined results written to {output_file}")

if __name__ == "__main__":
    combine_results()
