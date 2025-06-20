import os
import pandas as pd
import matplotlib.pyplot as plt
import numpy as np

# Hard-coded root directory containing benchmark subdirectories
ROOT_DIR = r"/home/guyamir/RustroverProjects/ser/out"  # <-- Update this path to your root experiments directory
OUTPUT_DIR = r"/home/guyamir/RustroverProjects/ser/optimization_experiments/pn_size"
SUMMARY_CSV = os.path.join(OUTPUT_DIR, "summary_petri_sizes.csv")
PLOT_PATH = os.path.join(OUTPUT_DIR, "petri_size_reduction.pdf")


def aggregate_csv_files(root_dir: str) -> pd.DataFrame:
    """
    Recursively search for 'petri_sizes.csv' files under root_dir,
    read each into a DataFrame, add a 'benchmark' column (the
    name of the subdirectory containing the CSV), and concatenate them.
    """
    records = []
    for dirpath, _, filenames in os.walk(root_dir):
        if "petri_sizes.csv" in filenames:
            csv_path = os.path.join(dirpath, "petri_sizes.csv")
            df = pd.read_csv(csv_path)
            benchmark_name = os.path.basename(dirpath)
            df.insert(0, "benchmark", benchmark_name)
            records.append(df)
    if not records:
        raise FileNotFoundError(f"No 'petri_sizes.csv' files found under {root_dir}")
    return pd.concat(records, ignore_index=True)


def generate_plot(df: pd.DataFrame, output_path: str):
    """
    Given a summary DataFrame with columns:
      ['benchmark','index','stage','num_places','num_transitions'],
    compute the average pre- and post-pruning sizes and plot them.
    """
    # Compute overall averages across all benchmarks
    agg = df.groupby("stage").agg({
        "num_places": "mean",
        "num_transitions": "mean"
    }).reindex(["pre_pruning", "post_pruning"])

    pre_places = agg.loc["pre_pruning", "num_places"]
    post_places = agg.loc["post_pruning", "num_places"]
    pre_trans = agg.loc["pre_pruning", "num_transitions"]
    post_trans = agg.loc["post_pruning", "num_transitions"]

    # Prepare bar positions
    categories = ["Places", "Transitions"]
    x = np.arange(len(categories))
    width = 0.35

    fig, ax = plt.subplots(figsize=(8, 5))

    # Plot places chunk with dark outlines
    bars_places_before = ax.bar(
        x[0] - width/2, pre_places, width,
        facecolor="forestgreen", edgecolor="black", linewidth=1
    )
    bars_places_after = ax.bar(
        x[0] + width/2, post_places, width,
        facecolor="lightgreen", edgecolor="black", linewidth=1
    )

    # Plot transitions chunk with dark outlines
    bars_trans_before = ax.bar(
        x[1] - width/2, pre_trans, width,
        facecolor="darkorange", edgecolor="black", linewidth=1
    )
    bars_trans_after = ax.bar(
        x[1] + width/2, post_trans, width,
        facecolor="peachpuff", edgecolor="black", linewidth=1
    )

    # Determine annotation offset
    max_height = max(pre_places, post_places, pre_trans, post_trans)
    y_offset = max_height * 0.02

    # Annotate each bar with "Before"/"After"
    for bar in bars_places_before:
        ax.text(
            bar.get_x() + bar.get_width() / 2,
            bar.get_height() + y_offset,
            "Before",
            ha="center", va="bottom"
        )
    for bar in bars_places_after:
        ax.text(
            bar.get_x() + bar.get_width() / 2,
            bar.get_height() + y_offset,
            "After",
            ha="center", va="bottom"
        )
    for bar in bars_trans_before:
        ax.text(
            bar.get_x() + bar.get_width() / 2,
            bar.get_height() + y_offset,
            "Before",
            ha="center", va="bottom"
        )
    for bar in bars_trans_after:
        ax.text(
            bar.get_x() + bar.get_width() / 2,
            bar.get_height() + y_offset,
            "After",
            ha="center", va="bottom"
        )

    # Labeling
    ax.set_xticks(x)
    ax.set_xticklabels(categories)
    ax.set_ylabel("Average Count")
    ax.set_title("Average Petri Net Size Before and After Pruning")

    # Set y-axis limit
    ax.set_ylim(0, 28)

    # Grid and layout
    ax.yaxis.grid(True, linestyle="--", linewidth=0.5, alpha=0.7)
    fig.tight_layout()

    # Save plot
    fig.savefig(output_path)
    plt.close(fig)


if __name__ == "__main__":
    # Aggregate all petri_sizes.csv into one summary CSV
    summary_df = aggregate_csv_files(ROOT_DIR)
    summary_df.to_csv(SUMMARY_CSV, index=False)
    print(f"Summary CSV saved to: {SUMMARY_CSV}")

    # Generate and save the plot
    generate_plot(summary_df, PLOT_PATH)
    print(f"Plot saved to: {PLOT_PATH}")


if __name__ == "__main__":
    # Aggregate all petri_sizes.csv into one summary CSV
    summary_df = aggregate_csv_files(ROOT_DIR)
    summary_df.to_csv(SUMMARY_CSV, index=False)
    print(f"Summary CSV saved to: {SUMMARY_CSV}")

    # Generate and save the plot
    generate_plot(summary_df, PLOT_PATH)
    print(f"Plot saved to: {PLOT_PATH}")
