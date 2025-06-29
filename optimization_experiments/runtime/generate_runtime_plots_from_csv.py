#!/usr/bin/env python3
import os
import pandas as pd
import numpy as np
import matplotlib.pyplot as plt

# --- Configuration (hard-coded) ---
input_file          = '/home/guyamir/RustroverProjects/ser/optimization_experiments/runtime/compare_results_timeout_10s_combined.csv'
output_dir          = '/home/guyamir/RustroverProjects/ser/optimization_experiments/runtime'
include_timeouts    = True   # False to drop any benchmark that ever timed out
filter_by_flag_sums = True   # only plot combos whose sum of ON-flags is in allowed_sums
allowed_flag_sums   = [0, 1, 4]
timeout_values      = [10_000]  # ms

# ensure output folder exists
os.makedirs(output_dir, exist_ok=True)

# read and massage the CSV
df = pd.read_csv(input_file)
TIMEOUT_MS = timeout_values[0]
df['terminated']      = df['elapsed_ms'].astype(str) != 'timeout'
df['elapsed_ms_num']  = pd.to_numeric(df['elapsed_ms'], errors='coerce').fillna(TIMEOUT_MS)
df['timeout_ms']      = TIMEOUT_MS
df['benchmark']       = df['example'].str.replace(r'\.(json|ser)$','',regex=True)

flag_cols = [
    'bidirectional_pruning ON',
    'remove_redundant ON',
    'generate_less ON',
    'smart_order ON'
]

def make_label(row):
    on = [c.replace(' ON','') for c in flag_cols if row[c]==1]
    if   not on:             return 'none'
    elif len(on)==len(flag_cols): return 'all ON'
    else:                    return '+'.join(on)

df['combination'] = df.apply(make_label, axis=1)

# Set white background and font size 18
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

def agg(series, metric='average'):
    x = series.dropna()
    if metric=='average':    return x.mean()
    elif metric=='harmonic': return len(x)/((1.0/x).sum()) if len(x)>0 else np.nan
    else:                    raise ValueError(metric)

def compute_global_timeout_pct(df_):
    ct = df_.groupby('combination')['terminated'].agg(total='size', succ='sum')
    return 100*(ct['total']-ct['succ'])/ct['total']

def filter_no_timeouts(grp):
    keep = grp.groupby('benchmark')['terminated'].all()
    return grp[grp['benchmark'].isin(keep[keep].index)]

def plot_cumulative_solved(group, timeout_ms, log_scale):
    fig, ax = plt.subplots(figsize=(10,6))
    if log_scale:
        ax.set_xscale('log')
    for combo in group['combination'].unique():
        times = np.sort(group.loc[group['combination']==combo, 'elapsed_ms_num'].values)
        solved_cum = np.cumsum(times < timeout_ms) / len(times) * 100
        ax.plot(np.minimum(times, timeout_ms), solved_cum, linewidth=2, label=combo)
    ax.axhline(100, linestyle='--', color='gray', alpha=0.5)
    ax.set_xlabel('Time (ms)', fontsize=18)
    ax.set_ylabel('% of instances solved', fontsize=18)
    scale = 'Log' if log_scale else 'Linear'
    ax.set_title(f'Cumulative Solved ({scale} scale, timeout={timeout_ms} ms)', fontsize=18)
    ax.grid(True)
    ax.tick_params(axis='both', labelsize=18)
    ax.legend(bbox_to_anchor=(1.05,1), loc='upper left', fontsize=14)
    fig.tight_layout()
    suffix = 'log' if log_scale else 'linear'
    out = os.path.join(output_dir, f"timeout_{timeout_ms}_cumulative_solved_{suffix}.pdf")
    fig.savefig(out, dpi=300, bbox_inches='tight')
    plt.close(fig)
    print("Wrote", out)

def process():
    df_f = df.copy()
    if filter_by_flag_sums:
        df_f = df_f[df_f[flag_cols].sum(axis=1).isin(allowed_flag_sums)]

    global_pct = compute_global_timeout_pct(df_f)

    for timeout_ms in timeout_values:
        grp = df_f[df_f['timeout_ms']==timeout_ms]
        if not include_timeouts:
            grp = filter_no_timeouts(grp)

        # Plot 1: average time bar chart
        metric_vals = grp.groupby('combination')['elapsed_ms_num']\
                         .agg(lambda x: agg(x,'average'))\
                         .sort_values(ascending=False)
        fig, ax = plt.subplots(figsize=(8, max(4, len(metric_vals)*0.5)))
        bars = ax.barh(metric_vals.index, metric_vals.values, edgecolor='black')
        for bar in bars:
            w = bar.get_width()
            ax.text(w + max(metric_vals.values)*0.01,
                    bar.get_y() + bar.get_height()/2,
                    f"{w:.1f}", va='center', fontsize=18)
        ax.set_xlabel("Average time (ms)", fontsize=18)
        ax.set_title(f"Average vs Combination (timeout={timeout_ms} ms)", fontsize=18)
        ax.grid(axis='x')
        ax.tick_params(axis='both', labelsize=18)
        fig.tight_layout()
        out1 = os.path.join(output_dir, f"timeout_{timeout_ms}_avg_times.pdf")
        fig.savefig(out1, dpi=300, bbox_inches='tight')
        plt.close(fig)
        print("Wrote", out1)

        # Plot 2a: linear cumulative
        plot_cumulative_solved(grp, timeout_ms, log_scale=False)
        # Plot 2b: log cumulative
        plot_cumulative_solved(grp, timeout_ms, log_scale=True)

if __name__=='__main__':
    process()
