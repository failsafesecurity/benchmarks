#!/usr/bin/env python3
"""
Figure 4 — Which framework is safest, per model?

Grouped bar chart: 4 model groups × 3 framework bars each.
Framework ordering is consistent within each group (Ironclaw / Hermes / Openclaw),
so the eye can spot that the WINNING (shortest) bar lands in a different
position for each model — which is the unstable-ranking finding.
"""
import matplotlib.pyplot as plt
import numpy as np

FRAMEWORKS = ['Ironclaw', 'Hermes', 'Openclaw']
COLORS = {'Ironclaw': '#3261a8', 'Hermes': '#e07b3a', 'Openclaw': '#2a9d63'}

DATA = {
    'Sonnet 4.6': {'Ironclaw': 6,  'Hermes': 7,  'Openclaw': 13},
    'GLM-5':       {'Ironclaw': 7,  'Hermes': 18, 'Openclaw': 9},
    'Qwen-3.5':    {'Ironclaw': 13, 'Hermes': 11, 'Openclaw': 16},
    'Kimi K2.6':   {'Ironclaw': 17, 'Hermes': 22, 'Openclaw': 14},
}

models = list(DATA.keys())
x = np.arange(len(models))
bar_w = 0.26

fig, ax = plt.subplots(figsize=(12, 6.2))

# Draw bars
for i, fw in enumerate(FRAMEWORKS):
    vals = [DATA[m][fw] for m in models]
    bars = ax.bar(x + (i - 1) * bar_w, vals, bar_w,
                  label=fw, color=COLORS[fw], edgecolor='white', linewidth=1.2)
    for j, b in enumerate(bars):
        ax.text(b.get_x() + b.get_width() / 2, b.get_height() + 0.4,
                f'{int(b.get_height())}',
                ha='center', va='bottom', fontsize=10, color='#222', fontweight='bold')

# Crown the safest framework per model
for j, m in enumerate(models):
    safest_fw = min(DATA[m], key=DATA[m].get)
    safest_idx = FRAMEWORKS.index(safest_fw)
    safest_v = DATA[m][safest_fw]
    bx = x[j] + (safest_idx - 1) * bar_w
    ax.annotate('★\nsafest', (bx, safest_v),
                textcoords='offset points', xytext=(0, 22),
                ha='center', va='bottom', fontsize=11, color='#b8860b',
                fontweight='bold')

ax.set_xticks(x)
ax.set_xticklabels(models, fontsize=12, fontweight='bold')
ax.set_ylabel('Violations (out of 52 scenarios)', fontsize=11)
ax.set_ylim(0, 30)
ax.set_yticks(range(0, 26, 5))
ax.grid(axis='y', linestyle=':', alpha=0.4, zorder=0)
ax.spines['top'].set_visible(False)
ax.spines['right'].set_visible(False)
ax.set_axisbelow(True)

ax.legend(title='Framework', loc='upper left', fontsize=10, title_fontsize=11,
          frameon=True, edgecolor='#ddd', ncol=3)

ax.set_title("The safest framework is different for every model",
             fontsize=14, fontweight='bold', pad=16)

# subtitle
fig.text(0.5, 0.02,
         "Bars in consistent order within each group. The ★ moves across positions — "
         "the winning framework is different for every model.",
         ha='center', fontsize=10, color='#555', style='italic')

plt.tight_layout(rect=[0, 0.05, 1, 1])
plt.savefig('docs/paper/figures/figure_1_safest_per_model.png',
            dpi=180, bbox_inches='tight', facecolor='white')
print('wrote docs/paper/figures/figure_1_safest_per_model.png')
