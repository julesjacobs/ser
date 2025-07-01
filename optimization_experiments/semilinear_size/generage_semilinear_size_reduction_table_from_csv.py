import pandas as pd


def compute_stats(df, combo):
    """
    Compute mean and max statistics for a given DataFrame filtered by an optimization combo.
    """
    df_filtered = df.copy()
    for col, val in combo.items():
        df_filtered = df_filtered[df_filtered[col] == val]

    return {
        'mean_num_components': df_filtered['num_components'].mean(),
        'max_num_components': df_filtered['num_components'].max(),
        'mean_periods_per_component': df_filtered['avg_periods_per_component'].mean(),
        'max_periods_per_component': df_filtered['max_periods_per_component'].max(),
    }


def main():
    # Hard-coded paths for the input CSV and the summary output CSV
    input_csv = r'/home/guyamir/RustroverProjects/ser/optimization_experiments/semilinear_size/semilinear_size_stats_final_timeout_120_seconds.csv'
    output_csv = r'/home/guyamir/RustroverProjects/ser/optimization_experiments/semilinear_size/summary_semilinear_size_stats_final_timeout_120_seconds.csv'
    print(f"Loading data from {input_csv}...")

    # Read the CSV into a DataFrame
    df = pd.read_csv(input_csv)

    # Optimization flag columns with ' ON' suffix
    opt_cols = [
        'bidirectional_pruning ON',
        'remove_redundant ON',
        'generate_less ON',
        'smart_order ON'
    ]

    # Determine total unique combos
    combos = df[opt_cols].drop_duplicates()
    total_combos = len(combos)

    # Create a combo identifier per row
    df['combo'] = df[opt_cols].apply(tuple, axis=1)

    # Keep only benchmarks present under all combos
    combo_counts = df.groupby('benchmark')['combo'].nunique().reset_index(name='n_combo')
    complete_benchmarks = combo_counts[combo_counts['n_combo'] == total_combos]['benchmark']
    df_full = df[df['benchmark'].isin(complete_benchmarks)].drop(columns='combo')

    # Define target scenarios (with renamed baseline label)
    scenarios = {
        'baseline (all ON)':       {'bidirectional_pruning ON':1, 'remove_redundant ON':1, 'generate_less ON':1, 'smart_order ON':1},
        'no_remove_redundant':     {'bidirectional_pruning ON':1, 'remove_redundant ON':0, 'generate_less ON':1, 'smart_order ON':1},
        'no_generate_less':        {'bidirectional_pruning ON':1, 'remove_redundant ON':1, 'generate_less ON':0, 'smart_order ON':1},
        'no_smart_order':          {'bidirectional_pruning ON':1, 'remove_redundant ON':1, 'generate_less ON':1, 'smart_order ON':0}
    }

    # Compute stats
    results = []
    for name, combo in scenarios.items():
        stats = compute_stats(df_full, combo)
        stats['scenario'] = name
        results.append(stats)

    res_df = pd.DataFrame(results)

    # Compute deltas from baseline for each metric
    baseline = res_df[res_df['scenario'] == 'baseline (all ON)'].iloc[0]
    metrics = [col for col in res_df.columns if col != 'scenario']
    for col in metrics:
        res_df[col] = res_df[col].apply(
            lambda v: f"{v:.2f} ({(v - baseline[col]):+.2f})"
        )

    # Ensure 'scenario' is leftmost
    cols = ['scenario'] + metrics
    res_df = res_df[cols]

    # Print and save
    print(res_df.to_string(index=False))
    res_df.to_csv(output_csv, index=False)
    print(f"Summary statistics written to {output_csv}.")


if __name__ == '__main__':
    main()
