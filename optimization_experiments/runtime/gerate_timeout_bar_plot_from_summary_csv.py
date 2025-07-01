import pandas as pd
import matplotlib.pyplot as plt
import numpy as np

# Constants
SER_EXPERIMENTS = 41
NON_SER_EXPERIMENTS = 22

# Paths
CSV_PATH = '/home/guyamir/RustroverProjects/ser/optimization_experiments/runtime/summary_timeout_stats_final_timeout_120_seconds.csv'
OUTPUT_PLOT_PATH = '/home/guyamir/RustroverProjects/ser/optimization_experiments/runtime/solved_percentage_barplot.pdf'

# Read the summary CSV as comma-separated
df = pd.read_csv(CSV_PATH, sep=',', header=0)
df.columns = df.columns.str.strip()

# Configurations
configs = {
    (1, 1, 1, 1): 'all',
    (0, 1, 1, 1): 'without bidirectional',
    (1, 0, 1, 1): 'without remove-redundant',
    (1, 1, 0, 1): 'without generate-less',
    (1, 1, 1, 0): 'without ordering',
    (0, 0, 0, 0): 'none'
}

# Prepare data
labels = []
ser_pct = []
non_ser_pct = []

for combo, label in configs.items():
    bidir, remove, genless, order = combo
    subset = df[
        (df['bidirectional_pruning ON'] == bidir) &
        (df['remove_redundant ON']     == remove) &
        (df['generate_less ON']        == genless) &
        (df['smart_order ON']          == order)
    ]
    if subset.empty:
        s, n = 0, 0
    else:
        row = subset.iloc[0]
        s = row['terminated_ser']
        n = row['terminated_non_ser']
    labels.append(label)
    ser_pct.append(s / SER_EXPERIMENTS * 100)
    non_ser_pct.append(n / NON_SER_EXPERIMENTS * 100)

# Reorder so that 'all' is first, then the rest by descending SER%
idx_all = labels.index('all')
rest = [i for i in range(len(labels)) if i != idx_all]
rest_sorted = sorted(rest, key=lambda i: ser_pct[i], reverse=True)
new_order = [idx_all] + rest_sorted

# Apply the new order
labels = [labels[i] for i in new_order]
ser_pct = [ser_pct[i] for i in new_order]
non_ser_pct = [non_ser_pct[i] for i in new_order]

# Now reverse the entire list so that 'all' ends up at the top
labels = labels[::-1]
ser_pct = ser_pct[::-1]
non_ser_pct = non_ser_pct[::-1]

# ─── swap 'without ordering' ↔ 'without remove-redundant' ───
i_remove = labels.index('without remove-redundant')
i_order  = labels.index('without ordering')
# swap labels
labels[i_remove], labels[i_order] = labels[i_order], labels[i_remove]
# swap SER %
ser_pct[i_remove], ser_pct[i_order] = ser_pct[i_order], ser_pct[i_remove]
# swap non-SER %
non_ser_pct[i_remove], non_ser_pct[i_order] = non_ser_pct[i_order], non_ser_pct[i_remove]

# Plotting
plt.rcParams.update({'font.size': 20})

y = np.arange(len(labels))
height = 0.35

fig, ax = plt.subplots(figsize=(12, 7), constrained_layout=True)

# Draw Non-SER at bottom of each pair, SER at top
bars_non = ax.barh(
    y - height/2, non_ser_pct, height,
    label='Non-SER',
    facecolor='lightcoral',
    edgecolor='black',
    linewidth=1.5
)
bars_ser = ax.barh(
    y + height/2, ser_pct, height,
    label='SER',
    facecolor='lightblue',
    edgecolor='black',
    linewidth=1.5
)

# Annotate with percentages
for bar in bars_non + bars_ser:
    w = bar.get_width()
    ax.annotate(f'{w:.0f}%',
                xy=(w, bar.get_y() + bar.get_height()/2),
                xytext=(5, 0),
                textcoords='offset points',
                ha='left', va='center',
                fontsize=20)

# Labels, title
ax.set_xlabel('% solved instances', fontsize=20)
ax.set_yticks(y)
ax.set_yticklabels(labels)

# Legend outside at bottom-left, with SER first
ax.legend(
    handles=[bars_ser, bars_non],
    labels=['ser', 'non-ser'],
    fontsize=20,
    loc='lower left',
    bbox_to_anchor=(-0.5, -0.20)
)

# Expand bottom margin for legend
plt.subplots_adjust(bottom=0.30)

plt.tight_layout()
plt.savefig(OUTPUT_PLOT_PATH)
plt.close()

print(f'Bar plot saved to {OUTPUT_PLOT_PATH}')
