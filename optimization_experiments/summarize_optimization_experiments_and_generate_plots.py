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
# timeout_values        = [10000]  # List of timeout values you're interested in
timeout_values        = [30000]  # List of timeout values you're interested in

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

# Set professional style
plt.style.use('seaborn')
professional_colors = ['#4C72B0', '#55A868', '#C44E52', '#8172B2', '#CCB974', '#64B5CD']

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

# Generate cumulative solved instances plot
def plot_cumulative_solved(group, timeout_ms, output_dir):
    # Get unique combinations
    combinations = group['combination'].unique()

    plt.figure(figsize=(10, 6), facecolor='white')
    max_accumulated_time = 0
    # Generate curve for each combination
    for i, combo in enumerate(combinations):
        # Get all runs for this combination (both terminating and non-terminating)
        combo_runs = group[group['combination'] == combo]

        # Use actual time_ms_num for all runs (no replacement with timeout_ms)
        times = combo_runs['time_ms_num'].values

        # Sort times from smallest to largest
        sorted_times = np.sort(times)

        # Calculate cumulative percentage solved
        n = len(sorted_times)
        # cumulative_solved = np.array([np.sum(sorted_times <= t) for t in sorted_times]) / n * 100
        cumulative_solved = np.cumsum([1 if t < timeout_ms else 0 for t in sorted_times]) * 100.0 / len(sorted_times)

        if np.cumsum(sorted_times)[-1] > max_accumulated_time:
            max_accumulated_time = np.cumsum(sorted_times)[-1]

        # Plot the curve
        plt.plot(np.cumsum(sorted_times), cumulative_solved,
                 label=combo,
                 color=professional_colors[i % len(professional_colors)],
                 linewidth=2)

    plt.xlabel('Cumulative Time (ms)', fontsize=12)
    plt.ylabel('% of instances solved', fontsize=12)
    plt.title(f'Timeout: {timeout_ms} (ms): Cumulative Solved Instances (all runs)', fontsize=14, pad=20)
    plt.grid(True, linestyle='--', alpha=0.7)
    plt.legend(bbox_to_anchor=(1.05, 1), loc='upper left')


    # Add HORIZONTAL line at 100%
    plt.axhline(y=100, color='gray', linestyle='--', alpha=0.5,
                label='100% solved')

    plt.tight_layout()

    # Save the plot
    output_path = os.path.join(output_dir, f"timeout_{timeout_ms}_cumulative_solved_all_runs.pdf")
    plt.savefig(output_path, dpi=300, bbox_inches='tight')
    plt.close()
    print(f"Saved cumulative solved plot (all runs) to {output_path}")



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

        # Iterate over specified timeout values instead of all in the table
        for timeout_ms in timeout_values:
            # Filter for the current timeout value
            grp = df_f[df_f['timeout_ms'] == timeout_ms]

            if grp.empty:
                print(f"No data found for timeout value: {timeout_ms}")
                continue

            # Optionally filter out benchmarks that ever timed out
            group = filter_no_timeouts(grp) if not include_timeouts else grp

            # Compute metric values per combination
            metric_vals = group.groupby('combination')['time_ms_num']\
                               .agg(lambda x: aggregate_times(x, metric))\
                               .sort_values(ascending=False)

            # Plot 1: metric values
            fig, ax = plt.subplots(figsize=(8, max(4, len(metric_vals)*0.5)), facecolor='white')
            bars = ax.barh(metric_vals.index, metric_vals.values,
                          color=professional_colors[0],
                          edgecolor='white')

            # Add value labels
            for bar in bars:
                width = bar.get_width()
                ax.text(width + max(metric_vals.values)*0.02,
                       bar.get_y() + bar.get_height()/2,
                       f'{width:.1f}',
                       ha='left', va='center')

            ax.set_xlabel(f"{metric.capitalize()} time (ms)", fontsize=12)
            ax.set_ylabel('Optimization combination', fontsize=12)
            ax.set_title(f"{metric.capitalize()} vs Combination (timeout={timeout_ms} ms)",
                        fontsize=14, pad=20)
            ax.grid(axis='x', linestyle='--', alpha=0.7)
            plt.tight_layout()
            fig.savefig(os.path.join(output_dir, f"timeout_{timeout_ms}_{metric}_times.pdf"),
                       dpi=300, bbox_inches='tight')
            plt.close(fig)

            # Plot 2: global timeout percentage (in red)
            # Align y-axis to metric plot order
            pct = global_pct.reindex(metric_vals.index)
            fig2, ax2 = plt.subplots(figsize=(8, max(4, len(pct)*0.5)), facecolor='white')
            bars2 = ax2.barh(pct.index, pct.values,
                            color='#C44E52',  # Red color
                            edgecolor='white')

            # Add value labels
            for bar in bars2:
                width = bar.get_width()
                ax2.text(width + max(pct.values)*0.02,
                         bar.get_y() + bar.get_height()/2,
                         f'{width:.1f}%',
                         ha='left', va='center')

            ax2.set_xlabel('% of runs timed out (global)', fontsize=12)
            ax2.set_ylabel('Optimization combination', fontsize=12)
            ax2.set_title(f"Global Timeout % vs Combination",
                         fontsize=14, pad=20)
            ax2.grid(axis='x', linestyle='--', alpha=0.7)
            plt.tight_layout()
            fig2.savefig(os.path.join(output_dir, f"timeout_{timeout_ms}_global_timeout_pct.pdf"),
                        dpi=300, bbox_inches='tight')
            plt.close(fig2)

            # Plot 3: Cumulative solved instances
            plot_cumulative_solved(group, timeout_ms, output_dir)

    print(f"All plots written to {output_dir}")

if __name__ == '__main__':
    process()