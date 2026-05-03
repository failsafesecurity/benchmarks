#!/usr/bin/env python3
"""
Select the publishable slice of run artifacts for the matrix.

Given a source tree of raw run artifacts (typically the EC2 host directories
where the matrix was actually executed), produce the canonical
runs/matrix-<date>/ tree by:

  1. Walking each (model, framework) cell.
  2. For cells the paper cites verbatim (configured below) — copy the full
     v*_result.json files.
  3. For all other cells — emit a compact verdicts.jsonl.gz with one record
     per scenario.
  4. Aggregate into summary.csv (per cell) and per-attack-family.csv (per
     cell × attack family).

Usage:
    python select_run_artifacts.py \
        --src-inventory ../docs/inventory.md \
        --out runs/matrix-2026-05-02/

The --src-inventory file is the human-readable inventory mapping cells to
their actual on-disk paths (across multiple hosts / runs); this script
respects it as the source of truth.
"""

import argparse
import gzip
import json
import os
import re
import shutil
import sys
from collections import defaultdict
from pathlib import Path

# ── cells the paper cites verbatim — keep full reasoning traces ─────────────
VERBATIM_CELLS = {
    ("kimi-k2.6", "hermes"):     ["commitments/crud-resolve"],
    ("claude-sonnet-4-6", "hermes"): ["commitments/crud-resolve"],
}

# ── attack-family classifier (verdict-detail → family) ──────────────────────
def classify(detail: str) -> str:
    d = (detail or "").lower()
    if "tool_calls" in d and "cap" in d:
        return "max-iterations"
    if "no numeric value" in d:
        return "max-iterations"
    if "iterations" in d or "step budget" in d:
        return "max-iterations"
    if "closest number" in d or "literal" in d or "numeric_answer" in d:
        return "numeric-literal"
    if any(s in d for s in (
        "forbidden tool", "memory_write", "disallowed path",
        "agents.md", "identity/", "outside",
    )):
        return "write-instruction"
    if any(s in d for s in ("markdown", "image", "banner", "exfil", "![")):
        return "markdown-image"
    if "tools_not_used" in d:
        return "sidecar-config-artifact"  # excluded from analysis
    return "other"


def parse_manifest(
    manifest_path: Path, staging_root: Path,
) -> tuple[dict[tuple[str, str], list[Path]], dict[tuple[str, str], list[str]]]:
    """
    Parse runs/manifest.yaml and return:
        cells:    {(model, framework): [Path, ...]}    -- staged source paths
        verbatim: {(model, framework): [scenario, ...]}

    The manifest references remote hosts (old-ec2, box-a, box-b, laptop).
    This parser assumes data has already been rsynced into a local staging
    directory at:
        <staging_root>/<host>/<original/path>

    The caller is responsible for the rsync from each host. See
    scripts/stage_run_artifacts.sh.
    """
    import yaml
    cells: dict = defaultdict(list)
    verbatim: dict = defaultdict(list)
    data = yaml.safe_load(manifest_path.read_text())

    for cell in data.get("cells", []):
        key = (cell["model"], cell["framework"])
        for src in cell["sources"]:
            host = src["host"]
            remote = src["path"].lstrip("/")
            local = staging_root / host / remote
            if src.get("glob"):
                cells[key].extend(p for p in staging_root.glob(f"{host}/{remote}") if p.is_dir())
            else:
                cells[key].append(local)

    for v in data.get("verbatim", []):
        key = (v["model"], v["framework"])
        verbatim[key].extend(v["scenarios"])

    return dict(cells), dict(verbatim)


def compact_cell(cell_paths: list[Path], out_dir: Path) -> dict:
    """Walk a cell's raw artifacts and emit a compact verdicts.jsonl.gz."""
    out_dir.mkdir(parents=True, exist_ok=True)
    counts = defaultdict(int)
    seen = set()
    out_records = []
    for root in cell_paths:
        for result_path in root.rglob("v*_result.json"):
            try:
                d = json.loads(result_path.read_text())
            except json.JSONDecodeError:
                continue
            sid = d.get("scenario_id") or result_path.parent.name
            if sid in seen or "math" in sid:
                continue
            seen.add(sid)
            verdict_summary = {
                "scenario_id": sid,
                "any_violation": d.get("any_violation"),
                "violations": [],
                "first_violating_attempt": None,
            }
            for att in d.get("attempts", []):
                v = att.get("verdict") or {}
                if v.get("violated"):
                    detail = (v.get("violations") or [{}])[0].get("detail", "")
                    verdict_summary["violations"].append({
                        "attempt": att.get("attempt_id"),
                        "detail": detail,
                        "family": classify(detail),
                    })
                    if verdict_summary["first_violating_attempt"] is None:
                        verdict_summary["first_violating_attempt"] = att.get("attempt_id")
                        family = classify(detail)
                        if family != "sidecar-config-artifact":
                            counts[family] += 1
            out_records.append(verdict_summary)
    out_records.sort(key=lambda r: r["scenario_id"])

    # write compact gzip
    with gzip.open(out_dir / "verdicts.jsonl.gz", "wt") as f:
        for r in out_records:
            f.write(json.dumps(r) + "\n")

    # write per-cell summary.csv
    with (out_dir / "summary.csv").open("w") as f:
        f.write("scenario_id,any_violation,first_violating_attempt,family\n")
        for r in out_records:
            family = r["violations"][0]["family"] if r["violations"] else ""
            f.write(f"{r['scenario_id']},{r['any_violation']},"
                    f"{r['first_violating_attempt'] or ''},{family}\n")

    return dict(counts)


def copy_verbatim(cell_paths: list[Path], scenario: str, out_dir: Path):
    """Copy a single scenario's full run artifact into the verbatim slot."""
    target = out_dir / scenario.replace("/", "-")
    target.mkdir(parents=True, exist_ok=True)
    for root in cell_paths:
        for result_path in root.rglob("v*_result.json"):
            try:
                d = json.loads(result_path.read_text())
            except json.JSONDecodeError:
                continue
            if d.get("scenario_id") == scenario.split("/")[-1]:
                shutil.copy2(result_path, target / "full_run.json")
                return
    print(f"WARN: verbatim scenario {scenario} not found in {cell_paths}",
          file=sys.stderr)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--src-inventory", required=True, type=Path,
                    help="Path to docs/inventory.md (or a YAML manifest)")
    ap.add_argument("--out", required=True, type=Path,
                    help="Output dir, e.g. runs/matrix-2026-05-02/")
    args = ap.parse_args()

    args.out.mkdir(parents=True, exist_ok=True)
    cells = parse_inventory(args.src_inventory)
    summary_rows = []
    family_rows = []

    for (model, framework), paths in cells.items():
        cell_id = f"{model}-{framework}"
        cell_dir = args.out / "per-cell" / cell_id
        counts = compact_cell(paths, cell_dir)
        for scenario in VERBATIM_CELLS.get((model, framework), []):
            copy_verbatim(paths, scenario, cell_dir)

        total = sum(counts.values())
        summary_rows.append((model, framework, total))
        for fam, n in counts.items():
            family_rows.append((model, framework, fam, n))

    # write top-level summary.csv
    with (args.out / "summary.csv").open("w") as f:
        f.write("model,framework,violations\n")
        for r in sorted(summary_rows):
            f.write(",".join(map(str, r)) + "\n")

    # write per-attack-family.csv
    with (args.out / "per-attack-family.csv").open("w") as f:
        f.write("model,framework,attack_family,violations\n")
        for r in sorted(family_rows):
            f.write(",".join(map(str, r)) + "\n")

    print(f"wrote {args.out}/summary.csv")
    print(f"wrote {args.out}/per-attack-family.csv")
    print(f"wrote {len(summary_rows)} per-cell summaries under {args.out}/per-cell/")


if __name__ == "__main__":
    main()
