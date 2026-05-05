#!/usr/bin/env python3
from pathlib import Path
"""
Figure 2 — Two side-by-side heatmaps showing per-attack-family vulnerability:
  Left:  Framework × Attack family (aggregate across 4 models)
  Right: Model × Attack family (aggregate across 3 frameworks)

Counts come from the per-attack-family classifier in
docs/redforge_data_inventory.md (verdict-detail → family map).
"""
import matplotlib.pyplot as plt
import matplotlib.colors as mcolors
import numpy as np

ATTACKS = ['numeric-literal', 'write-instruction', 'max-iterations', 'markdown-image']

# Per-cell totals (verified against headline matrix)
FRAMEWORKS = ['Ironclaw', 'Hermes', 'Openclaw']
fw_data = np.array([
    [16, 1,  15, 10],   # Ironclaw
    [17, 25, 9,  7 ],   # Hermes
    [38, 0,  10, 4 ],   # Openclaw
])

MODELS = ['Sonnet 4.6', 'GLM-5', 'Qwen-3.5', 'Kimi K2.6']
m_data = np.array([
    [13, 0,  7,  6 ],   # Sonnet
    [15, 12, 6,  0 ],   # GLM-5
    [22, 5,  10, 3 ],   # Qwen
    [21, 9,  11, 12],   # Kimi
])

# Shared colormap: white → deep red, scaled to global max for direct comparability
vmax = max(fw_data.max(), m_data.max())
cmap = mcolors.LinearSegmentedColormap.from_list('reds', ['#fff7f3', '#fdbb84', '#e34a33', '#7f0000'])

fig, axes = plt.subplots(1, 2, figsize=(15, 5.5),
                         gridspec_kw={'width_ratios': [3, 4], 'wspace': 0.32})

def draw(ax, data, row_labels, title):
    im = ax.imshow(data, cmap=cmap, vmin=0, vmax=vmax, aspect='auto')
    ax.set_xticks(range(len(ATTACKS)))
    ax.set_xticklabels(ATTACKS, rotation=25, ha='right', fontsize=10)
    ax.set_yticks(range(len(row_labels)))
    ax.set_yticklabels(row_labels, fontsize=11)
    for i in range(data.shape[0]):
        for j in range(data.shape[1]):
            v = data[i, j]
            color = 'white' if v > vmax * 0.55 else '#222'
            ax.text(j, i, f'{v}', ha='center', va='center', color=color, fontsize=12, fontweight='bold')
    ax.set_title(title, fontsize=13, pad=12)
    ax.tick_params(top=False, bottom=True, left=True, right=False)
    return im

im1 = draw(axes[0], fw_data, FRAMEWORKS,
           'By framework\n(violations summed across all 4 models)')
im2 = draw(axes[1], m_data,  MODELS,
           'By model\n(violations summed across all 3 frameworks)')

cbar = fig.colorbar(im2, ax=axes, fraction=0.025, pad=0.04, shrink=0.85)
cbar.set_label('Violations across 52 scenarios × 5 attempts', fontsize=10)

fig.suptitle('Where each framework — and each model — fails',
             fontsize=15, fontweight='bold', y=1.02)
plt.savefig(str(Path(__file__).parent / 'figure_3_dual_heatmap.png'),
            dpi=180, bbox_inches='tight', facecolor='white')
print(f"wrote {Path(__file__).parent / 'figure_3_dual_heatmap.png'}")
