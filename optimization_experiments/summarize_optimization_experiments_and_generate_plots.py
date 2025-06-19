#!/usr/bin/env python3
import os
import pandas as pd
import numpy as np
import matplotlib.pyplot as plt

# --- Configuration (hard-coded) ---
input_file  = '/home/guyamir/RustroverProjects/ser/optimization_experiments/csvs/combined_outputs/combined_results.csv'
output_dir  = '/home/guyamir/RustroverProjects/ser/optimization_experiments/plots'

# Create output directory if missing
os.makedirs(output_dir, exist_ok=True)

# Read the merged CSV
df = pd.read_csv(input_file)
# Ensure time_ms is numeric; non-numeric (timeouts) → NaN
df['time_ms_num'] = pd.to_numeric(df['time_ms'], errors='coerce')

# Define filter types and metrics to process
types = ['ser', 'json', 'both']
metrics = ['harmonic', 'average']

# Build a human-readable "combination" label from the ON flags
flag_cols = [
    'bidirectional ON',
    'remove-redundant-parts ON',
    'remove-redundant-sets ON',
    'generate-less ON',
    'smart-kleene-order ON'
]

def make_label(row):
    on_flags = [col.replace(' ON','') for col in flag_cols if row[col] == 1]
    if len(on_flags) == len(flag_cols):
        return 'all ON'
    if not on_flags:
        return 'none'
    return '+'.join(on_flags)

df['combination'] = df.apply(make_label, axis=1)

# Harmonic‐mean function
def harmonic_mean(x):
    x_nonan = x.dropna()
    return len(x_nonan) / ( (1.0 / x_nonan).sum() ) if len(x_nonan) > 0 else np.nan

# Process each filter type and metric
def aggregate_times(series, metric):
    if metric == 'harmonic':
        return harmonic_mean(series)
    elif metric == 'average':
        return series.dropna().mean()
    else:
        raise ValueError(f"Unknown metric: {metric}")

for filter_type in types:
    # Apply file-type filter if requested
    if filter_type in ('json', 'ser'):
        df_filtered = df[df['type'] == filter_type]
    else:
        df_filtered = df.copy()

    # Determine distinct timeouts in this subset
    timeouts = sorted(df_filtered['timeout_ms'].unique())
    print(f"\nNumber of distinct timeouts for filter={filter_type}: {len(timeouts)}")

    for metric in metrics:
        for timeout_ms, group in df_filtered.groupby('timeout_ms'):
            # Compute aggregated metric per combination and sort descending
            agg = (
                group
                .groupby('combination')['time_ms_num']
                .agg(lambda x: aggregate_times(x, metric))
                .sort_values(ascending=False)
            )

            # Compute totals and terminated/timeouts as before
            totals = group.groupby('combination')['benchmark'].nunique().reindex(agg.index)
            terminated_counts = group.groupby('combination')['terminated'].sum().reindex(agg.index)
            timeout_counts = totals - terminated_counts

            # Print summary
            print(f"\nSummary for filter={filter_type}, metric={metric}, timeout={timeout_ms} ms:")
            for combo in agg.index:
                val = agg[combo]
                print(f"  {combo}: {val:.2f} ms, timeouts={int(timeout_counts[combo])}")

            # Plot
            fig, ax = plt.subplots(figsize=(8, max(4, len(agg)*0.5)))
            bars = ax.barh(agg.index, agg.values)
            ax.set_ylabel('Optimization combination')
            ax.set_xlabel(f'{metric.capitalize()} time (ms)')
            ax.set_title(f'Filter={filter_type}, Metric={metric}, Timeout={timeout_ms} ms')

            # Annotate timeout counts
            for bar, to_count in zip(bars, timeout_counts):
                ax.text(bar.get_width(), bar.get_y() + bar.get_height()/2, f"{int(to_count)}", va='center')

            plt.tight_layout()

            # Save figure with metric and filter in filename
            out_name = f"timeout_{timeout_ms}_filter_{filter_type}_metric_{metric}.png"
            out_path = os.path.join(output_dir, out_name)
            plt.savefig(out_path)
            plt.close()

print(f"\nAll plots written to {output_dir}")
