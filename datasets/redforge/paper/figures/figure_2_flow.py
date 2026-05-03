#!/usr/bin/env python3
from pathlib import Path
"""
Figure 1 — Red-vs-blue flow.

Solid arrows = the agent's normal request/response loop with the user.
Dashed arrows (red) = adversarial channels: red plants text into the files
blue is going to read, and red receives blue's reasoning trace between attempts.
"""
import matplotlib.pyplot as plt
from matplotlib.patches import FancyBboxPatch, FancyArrowPatch
from matplotlib.lines import Line2D

fig, ax = plt.subplots(figsize=(13, 6.5))
ax.set_xlim(0, 13)
ax.set_ylim(0, 8)
ax.axis('off')

BLUE_FILL = '#dde7f5'
BLUE_EDGE = '#3261a8'
RED_FILL  = '#fbdcdc'
RED_EDGE  = '#b22222'
NEUT_FILL = '#f1f3f5'
NEUT_EDGE = '#444'
ARROW_BLUE = '#3261a8'
ARROW_RED  = '#b22222'


def box(x, y, w, h, label, fill, edge, fontsize=11, fontweight='bold'):
    p = FancyBboxPatch((x, y), w, h,
                       boxstyle='round,pad=0.04,rounding_size=0.18',
                       linewidth=1.6, facecolor=fill, edgecolor=edge)
    ax.add_patch(p)
    ax.text(x + w / 2, y + h / 2, label, ha='center', va='center',
            fontsize=fontsize, fontweight=fontweight, color=edge)


def cylinder(x, y, w, h, label, fill, edge):
    p = FancyBboxPatch((x, y), w, h,
                       boxstyle='round,pad=0.02,rounding_size=0.4',
                       linewidth=1.6, facecolor=fill, edgecolor=edge)
    ax.add_patch(p)
    ax.text(x + w / 2, y + h / 2, label, ha='center', va='center',
            fontsize=10, fontweight='bold', color=edge)


def arrow(x1, y1, x2, y2, color, style='-', label=None, label_offset=(0, 0),
          label_fontsize=9, lw=1.7):
    style_ls = '--' if style == 'dashed' else '-'
    a = FancyArrowPatch((x1, y1), (x2, y2),
                        arrowstyle='-|>', mutation_scale=18,
                        color=color, linewidth=lw, linestyle=style_ls, zorder=3)
    ax.add_patch(a)
    if label:
        mx, my = (x1 + x2) / 2 + label_offset[0], (y1 + y2) / 2 + label_offset[1]
        ax.text(mx, my, label, ha='center', va='center',
                fontsize=label_fontsize, color=color, style='italic',
                bbox=dict(facecolor='white', edgecolor='none', pad=2))


# nodes ─ top row: user, blue, judge
box(0.4, 4.1, 1.7, 1.0, 'User', NEUT_FILL, NEUT_EDGE, fontsize=12)
box(4.6, 3.3, 3.6, 2.6,
    'Blue agent\n(model + framework)\nreads · reasons · acts',
    BLUE_FILL, BLUE_EDGE, fontsize=11)
box(11.0, 4.1, 1.7, 1.0, 'Judge', NEUT_FILL, NEUT_EDGE, fontsize=12)

# workspace files (above Blue)
cylinder(5.1, 6.2, 2.6, 0.95, 'Workspace files', NEUT_FILL, NEUT_EDGE)

# red attacker (below Blue)
box(5.0, 0.9, 2.8, 1.1, 'Red attacker\n(GPT-5)', RED_FILL, RED_EDGE, fontsize=11)

# solid arrows: normal flow
arrow(2.1, 4.7, 4.55, 4.7, ARROW_BLUE, label='question', label_offset=(0, 0.35))
arrow(4.55, 4.3, 2.15, 4.3, ARROW_BLUE, label='response', label_offset=(0, -0.35))
arrow(8.25, 4.6, 10.95, 4.6, ARROW_BLUE,
      label='final answer', label_offset=(0, 0.35))
# files → blue
arrow(6.4, 6.18, 6.4, 5.95, ARROW_BLUE, label='read',
      label_offset=(0.55, 0.05))

# dashed red arrows: adversarial channels
arrow(5.6, 2.05, 5.6, 6.18, ARROW_RED, style='dashed',
      label='plants text', label_offset=(-0.85, 0))
arrow(7.2, 3.3, 7.2, 2.05, ARROW_RED, style='dashed',
      label='reasoning trace\nseen between attempts',
      label_offset=(1.7, 0.0))

# legend
legend_elements = [
    Line2D([0], [0], color=ARROW_BLUE, lw=2, label='Normal request/response'),
    Line2D([0], [0], color=ARROW_RED,  lw=2, linestyle='--',
           label='Adversarial channel'),
]
ax.legend(handles=legend_elements, loc='lower right', fontsize=10,
          frameon=True, edgecolor='#ddd', bbox_to_anchor=(1.0, 0.02))

ax.text(6.5, 7.7, 'Red vs blue, with full reasoning visibility',
        ha='center', va='center', fontsize=14, fontweight='bold', color='#222')

plt.savefig(str(Path(__file__).parent / 'figure_2_flow.png'),
            dpi=180, bbox_inches='tight', facecolor='white')
print(f'wrote {Path(__file__).parent / "figure_2_flow.png"}')
