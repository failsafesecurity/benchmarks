#!/usr/bin/env python3
from pathlib import Path
"""
Figure 3 — Parallel-coordinates view of the headline matrix.

Each line is one model. The three vertical axes are the three frameworks.
Points are positioned by violation count (out of 52 scenarios).
The lines crossing each other is the unstable-ranking finding made visual.
"""
import matplotlib.pyplot as plt
import numpy as np

FRAMEWORKS = ['Ironclaw', 'Hermes', 'Openclaw']
DATA = {
    'Sonnet 4.6': [6, 7, 13],
    'GLM-5':       [7, 18, 9],
    'Qwen-3.5':   [13, 11, 16],
    'Kimi K2.6':  [17, 22, 14],
}
COLORS = {
    'Sonnet 4.6': '#1f77b4',
    'GLM-5':      '#ff7f0e',
    'Qwen-3.5':   '#2ca02c',
    'Kimi K2.6':  '#d62728',
}

fig, ax = plt.subplots(figsize=(11, 6.5))

# Vertical axes
xs = np.arange(len(FRAMEWORKS))
for x in xs:
    ax.axvline(x, color='#bbb', linewidth=1, zorder=1)

# Lines + points
for model, vals in DATA.items():
    color = COLORS[model]
    ax.plot(xs, vals, color=color, linewidth=2.5, marker='o', markersize=10,
            markeredgecolor='white', markeredgewidth=1.5, label=model, zorder=3)
    # Annotate each point
    for x, v in zip(xs, vals):
        # Slight nudge to avoid overlapping labels at the same x
        nudge_y = 0.7 if v not in (7,) else -0.9  # generic small offset
        ax.annotate(str(v), (x, v), textcoords='offset points', xytext=(8, 6),
                    fontsize=10, color=color, fontweight='bold', zorder=4)

ax.set_xticks(xs)
ax.set_xticklabels(FRAMEWORKS, fontsize=12, fontweight='bold')
ax.set_ylabel('Violations (out of 52 scenarios)', fontsize=11)
ax.set_xlim(-0.4, len(FRAMEWORKS) - 0.6)
ax.set_ylim(0, 25)
ax.set_yticks(range(0, 26, 5))
ax.tick_params(axis='y', labelsize=10)
ax.grid(axis='y', linestyle=':', alpha=0.4, zorder=0)
ax.spines['top'].set_visible(False)
ax.spines['right'].set_visible(False)

ax.legend(title='Model', loc='upper left', fontsize=10, title_fontsize=11,
          frameon=True, edgecolor='#ddd')

ax.set_title('The same model, scored across three frameworks',
             fontsize=14, fontweight='bold', pad=14)

# Subtle subtitle
fig.text(0.5, 0.02,
         'Each line is one model. Lines crossing means rankings disagree across models — '
         'no single framework wins for everyone.',
         ha='center', fontsize=10, color='#555', style='italic')

plt.tight_layout(rect=[0, 0.04, 1, 1])
plt.savefig(str(Path(__file__).parent / 'figure_4_parallel_coords.png'),
            dpi=180, bbox_inches='tight', facecolor='white')
print(f"wrote {Path(__file__).parent / 'figure_4_parallel_coords.png'}")
