import os
import pandas as pd
import matplotlib.pyplot as plt
import numpy as np

# Directory where the new summary CSV lives
OUTPUT_DIR = r"/home/guyamir/RustroverProjects/ser/optimization_experiments/petri_size"
SUMMARY_CSV = os.path.join(OUTPUT_DIR, "petri_size_stats.csv")
PLOT_PATH = os.path.join(OUTPUT_DIR, "petri_size_reduction_plot.pdf")


def generate_plot(df: pd.DataFrame, output_path: str):
    """
    Given a filtered DataFrame with columns:
      ['bidirectional_pruning ON','remove_redundant ON',
       'generate_less ON','smart_order ON',
       'benchmark','index','stage','num_places','num_transitions'],
    compute the average pre- and post-pruning sizes and plot them.
    """
    agg = df.groupby("stage").agg({
        "num_places": "mean",
        "num_transitions": "mean"
    }).reindex(["pre_pruning", "post_pruning"])

    pre_places = agg.loc["pre_pruning", "num_places"]
    post_places = agg.loc["post_pruning", "num_places"]
    pre_trans = agg.loc["pre_pruning", "num_transitions"]
    post_trans = agg.loc["post_pruning", "num_transitions"]

    categories = ["Places", "Transitions"]
    x = np.arange(len(categories))
    width = 0.35

    fig, ax = plt.subplots(figsize=(8, 5))

    # Plot bars
    ax.bar(x[0] - width/2, pre_places, width, label="Before", facecolor="forestgreen",edgecolor="black", linewidth=1)
    ax.bar(x[0] + width/2, post_places, width, label="After", facecolor="lightgreen",edgecolor="black", linewidth=1)
    ax.bar(x[1] - width/2, pre_trans, width, facecolor="darkorange", edgecolor="black", linewidth=1)
    ax.bar(x[1] + width/2, post_trans, width, facecolor="peachpuff", edgecolor="black", linewidth=1)

    # Annotate
    max_height = max(pre_places, post_places, pre_trans, post_trans)
    y_offset = max_height * 0.02
    for i, height in enumerate([pre_places, post_places]):
        x_pos = x[0] + (i*2-1)*width/2
        ax.text(x_pos, height + y_offset, ["Before", "After"][i], ha="center")
    for i, height in enumerate([pre_trans, post_trans]):
        x_pos = x[1] + (i*2-1)*width/2
        ax.text(x_pos, height + y_offset, ["Before", "After"][i], ha="center")

    ax.set_xticks(x)
    ax.set_xticklabels(categories)
    ax.set_ylabel("Average Count")
    ax.set_title("Average Petri Net Size Before and After Pruning")
    ax.set_ylim(0, max_height * 1.1)
    ax.yaxis.grid(True, linestyle="--", linewidth=0.5, alpha=0.7)
    fig.tight_layout()

    fig.savefig(output_path)
    plt.close(fig)


if __name__ == "__main__":
    # Read the already-aggregated summary CSV
    df = pd.read_csv(SUMMARY_CSV)

    # Keep only rows where bidirectional pruning was ON
    df = df[df["bidirectional_pruning ON"] == 1]

    # Remove duplicate entries for the same benchmark, index, and stage
    df = df.drop_duplicates(subset=["benchmark", "index", "stage"])

    # Save the filtered summary back to CSV (optional)
    # df.to_csv(SUMMARY_CSV, index=False)
    # print(f"Filtered summary saved to: {SUMMARY_CSV}")

    # Generate and save the plot
    generate_plot(df, PLOT_PATH)
    print(f"Plot saved to: {PLOT_PATH}")
