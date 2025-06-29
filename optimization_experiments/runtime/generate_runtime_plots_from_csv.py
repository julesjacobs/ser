#!/usr/bin/env python3
import os
import pandas as pd
import numpy as np
import matplotlib.pyplot as plt

# --- Configuration (hard-coded) ---
input_file          = '/home/guyamir/RustroverProjects/ser/optimization_experiments/runtime/compare_results_timeout_10s_combined.csv'
output_dir          = '/home/guyamir/RustroverProjects/ser/optimization_experiments/runtime'
filter_by_flag_sums = True   # only plot combos whose sum of ON-flags is in allowed_sums
allowed_flag_sums   = [0, 1, 4]
timeout_values      = [10_000]  # ms

# ensure output folder exists
os.makedirs(output_dir, exist_ok=True)

# read and preprocess CSV
df = pd.read_csv(input_file)
TIMEOUT_MS = timeout_values[0]
df['elapsed_ms_num'] = pd.to_numeric(df['elapsed_ms'], errors='coerce').fillna(TIMEOUT_MS)
df['timeout_ms']     = TIMEOUT_MS
df['benchmark']      = df['example'].str.replace(r'\.(json|ser)$','',regex=True)
# mark timeouts
df['timeout'] = df['elapsed_ms_num'] >= TIMEOUT_MS

# flag columns
flag_cols = [
    'bidirectional_pruning ON',
    'remove_redundant ON',
    'generate_less ON',
    'smart_order ON'
]

# helper to create a label for each row
def make_label(row):
    on = [c.replace(' ON','') for c in flag_cols if row[c] == 1]
    if not on:
        return 'none'
    if len(on) == len(flag_cols):
        return 'all ON'
    return '+'.join(on)

df['combination'] = df.apply(make_label, axis=1)

# --- DEFINE A CONSISTENT COLOR MAP FOR ALL PLOTS ---
# get all unique combination labels once (sorted for consistency)
combination_labels = sorted(df['combination'].unique())
# grab the default matplotlib color cycle
colors = plt.rcParams['axes.prop_cycle'].by_key()['color']
# map each combination label to a specific, fixed color
color_map = {
    combo: colors[i % len(colors)]
    for i, combo in enumerate(combination_labels)
}

# Matplotlib style settings
plt.style.use('seaborn-whitegrid')
plt.rcParams.update({
    'font.size': 18,
    'figure.facecolor': 'white',
    'axes.facecolor': 'white',
    'axes.edgecolor': 'black',
    'grid.color': '#dddddd',
    'grid.linestyle': '--',
    'grid.linewidth': 0.5
})

# simple aggregation helper for bar plots
def agg(series):
    return series.mean()

# function to plot cumulative solved percentages
def plot_cumulative_solved(group, timeout_ms, log_scale=False):
    fig, ax = plt.subplots(figsize=(10, 6))
    if log_scale:
        ax.set_xscale('log')

    for combo in combination_labels:
        # extract and sort times for this combo
        times = np.sort(group.loc[group['combination'] == combo, 'elapsed_ms_num'].values)
        solved_cum = np.cumsum(times < timeout_ms) / len(times) * 100
        # plot with fixed color
        ax.plot(np.minimum(times, timeout_ms),
                solved_cum,
                linewidth=2,
                label=combo,
                color=color_map[combo])

    ax.axhline(100, linestyle='--', color='gray', alpha=0.5)
    ax.set_xlabel('Time (ms)', fontsize=18)
    ax.set_ylabel('% of instances solved', fontsize=18)
    ax.grid(True)
    ax.tick_params(axis='both', labelsize=18)
    ax.legend(bbox_to_anchor=(1.05, 1), loc='upper left', fontsize=14)
    fig.tight_layout()
    suffix = 'log' if log_scale else 'linear'
    out = os.path.join(output_dir, f"timeout_{timeout_ms}_cumulative_solved_{suffix}.pdf")
    fig.savefig(out, dpi=300, bbox_inches='tight')
    plt.close(fig)
    print(f"Wrote {out}")

# main processing function
def process():
    df_f = df.copy()

    for timeout_ms in timeout_values:
        grp = df_f[df_f['timeout_ms'] == timeout_ms]

        # Print total timeouts per combination
        print(f"Timeout counts (timeout={timeout_ms} ms):")
        for combo in combination_labels:
            sub = grp[grp['combination'] == combo]
            count_to = (sub['elapsed_ms_num'] >= timeout_ms).sum()
            print(f"  {combo}: {count_to} rows with elapsed_ms >= {timeout_ms}")

        # apply optional filter by flag sums
        if filter_by_flag_sums:
            grp = grp[grp[flag_cols].sum(axis=1).isin(allowed_flag_sums)]

        # First plot: average time bar chart
        metric = grp.groupby('combination')['elapsed_ms_num'] \
                    .agg(agg).sort_values(ascending=False)
        fig, ax = plt.subplots(figsize=(8, max(4, len(metric) * 0.5)))
        # assign bar colors in same fixed order
        bar_colors = [color_map[combo] for combo in metric.index]
        bars = ax.barh(metric.index, metric.values, color=bar_colors, edgecolor='black')

        # annotate bars with values
        for bar in bars:
            w = bar.get_width()
            ax.text(
                w + max(metric.values)*0.01,
                bar.get_y() + bar.get_height()/2,
                f"{w:.1f}",
                va='center',
                fontsize=14
            )

        ax.set_xlabel("Average time (ms)", fontsize=18)
        ax.grid(axis='x')
        ax.tick_params(axis='both', labelsize=18)
        ax.set_xlim(0, 1300)
        fig.tight_layout()
        out1 = os.path.join(output_dir, f"timeout_{timeout_ms}_avg_common.pdf")
        fig.savefig(out1, dpi=300, bbox_inches='tight')
        plt.close(fig)
        print(f"Wrote {out1}")

        # Cumulative-solved plots (linear and log)
        plot_cumulative_solved(grp, timeout_ms, log_scale=False)
        plot_cumulative_solved(grp, timeout_ms, log_scale=True)

if __name__ == '__main__':
    process()
