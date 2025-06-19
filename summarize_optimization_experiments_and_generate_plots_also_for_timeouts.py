#!/usr/bin/env python3
import os
import pandas as pd
import numpy as np
import matplotlib.pyplot as plt

# --- Configuration (hard-coded) ---
input_file            = '/home/guyamir/RustroverProjects/ser/optimization_experiments/csvs/combined_outputs/combined_results_merge_redundant_optimizations.csv'
output_dir            = '/home/guyamir/RustroverProjects/ser/optimization_experiments/plots'
include_timeouts      = True  # Set to False to exclude benchmarks that timeout in any combination
filter_by_flag_sums   = True   # Set to True to only plot rows whose sum of ON flags is in allowed_sums
allowed_flag_sums     = [0, 1, 4]  # Allowed sums of ON flags when filtering

# Create output directory if missing
os.makedirs(output_dir, exist_ok=True)

# Read data
df = pd.read_csv(input_file)
df['time_ms_num'] = pd.to_numeric(df['time_ms'], errors='coerce')

# Configuration for processing
types   = ['both']
metric  = 'average'  # Only 'average' for now
flag_cols = [
    'bidirectional ON',
    'remove-redundant ON',
    'generate-less ON',
    'smart-kleene-order ON'
]

# Build label for each row based on flags
def make_label(row):
    flags = [col.replace(' ON','') for col in flag_cols if row[col]==1]
    if len(flags)==0: return 'none'
    if len(flags)==len(flag_cols): return 'all ON'
    return '+'.join(flags)

df['combination'] = df.apply(make_label, axis=1)

# Aggregation helper
def aggregate_times(series, metric):
    x = series.dropna()
    if metric=='average': return x.mean()
    if metric=='harmonic': return len(x)/((1.0/x).sum()) if len(x)>0 else np.nan
    raise ValueError(f"Unknown metric: {metric}")

# Exclude benchmarks that timeout in any combination
def filter_no_timeouts(group):
    combos = group['combination'].unique()
    success = group[group['terminated']==1].groupby('benchmark')['combination'].nunique()
    valid = success[success==len(combos)].index
    return group[group['benchmark'].isin(valid)]

# Precompute global timeout percentages per combination (out of all runs)
def compute_global_timeout_pct(df_f):
    counts = df_f.groupby('combination')['terminated'].agg(['size','sum'])
    # size = total runs, sum = number of terminations
    counts['timeout_pct'] = 100 * (counts['size'] - counts['sum']) / counts['size']
    return counts['timeout_pct']

# Main processing
def process():
    for filter_type in types:
        # Filter by file type if needed
        df_f = df if filter_type=='both' else df[df['type']==filter_type]

        # Optional flag-sum filter
        if filter_by_flag_sums:
            df_f = df_f[df_f[flag_cols].sum(axis=1).isin(allowed_flag_sums)]

        # Compute global timeout % for this subset
        global_pct = compute_global_timeout_pct(df_f)

        # Iterate each timeout threshold
        for timeout_ms, grp in df_f.groupby('timeout_ms'):
            # Optionally filter out benchmarks that ever timed out
            group = filter_no_timeouts(grp) if not include_timeouts else grp

            # Compute metric values per combination
            metric_vals = group.groupby('combination')['time_ms_num']\
                               .agg(lambda x: aggregate_times(x, metric))\
                               .sort_values(ascending=False)

            # Plot 1: metric values
            fig, ax = plt.subplots(figsize=(8, max(4,len(metric_vals)*0.5)))
            ax.barh(metric_vals.index, metric_vals.values)
            ax.set_xlabel(f"{metric.capitalize()} time (ms)")
            ax.set_ylabel('Optimization combination')
            ax.set_title(f"{metric.capitalize()} vs Combination (timeout={timeout_ms} ms)")
            plt.tight_layout()
            fig.savefig(os.path.join(output_dir, f"timeout_{timeout_ms}_{metric}_times.png"))
            plt.close(fig)

            # Plot 2: global timeout percentage
            # Align y-axis to metric plot order
            pct = global_pct.reindex(metric_vals.index)
            fig2, ax2 = plt.subplots(figsize=(8, max(4,len(pct)*0.5)))
            ax2.barh(pct.index, pct.values)
            ax2.set_xlabel('% of runs timed out (global)')
            ax2.set_ylabel('Optimization combination')
            ax2.set_title(f"Global Timeout % vs Combination")
            plt.tight_layout()
            fig2.savefig(os.path.join(output_dir, f"timeout_{timeout_ms}_global_timeout_pct.png"))
            plt.close(fig2)

    print(f"All plots written to {output_dir}")

if __name__=='__main__':
    process()
