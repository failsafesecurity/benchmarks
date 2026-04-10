# PinchBench Dataset

Task definitions and assets from [PinchBench](https://pinchbench.com) — a benchmarking system for evaluating LLM models as coding agents.

## Source

- **Repository**: https://github.com/pinchbench/skill
- **Website**: https://pinchbench.com
- **License**: MIT
- **Authors**: [Kilo Code](https://kilo.ai) — @olearycrew, @arpitg1991, @DJRHails, @evanjacobson, @iJaack

## Contents

- `v1/tasks/` — task definitions as markdown files with YAML frontmatter
- `v1/assets/` — supporting files (PDFs, spreadsheets, text) referenced by tasks, stored in Git LFS

## Setup

Assets are stored in Git LFS. After cloning, either pull them via LFS:

```bash
git lfs install
git lfs pull
```

Or re-download directly from upstream (does not require LFS):

```bash
scripts/pinchbench_download.sh
```

## Updating

To re-download assets from the latest upstream:

```bash
scripts/pinchbench_download.sh --force
```
