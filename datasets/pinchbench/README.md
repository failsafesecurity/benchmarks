# PinchBench Dataset

Task definitions and assets from [PinchBench](https://pinchbench.com) — a benchmarking system for evaluating LLM models as coding agents.

## Source

- **Repository**: https://github.com/pinchbench/skill
- **Website**: https://pinchbench.com
- **License**: MIT
- **Authors**: [Kilo Code](https://kilo.ai) — @olearycrew, @arpitg1991, @DJRHails, @evanjacobson, @iJaack

## Contents

- `v1/tasks/` — 23 task definitions as markdown files with YAML frontmatter
- `v1/assets/` — Supporting files (PDFs, spreadsheets, text) referenced by tasks

## Updating

To re-download tasks and assets from the latest upstream:

```bash
scripts/pinchbench_download.sh --force
```
