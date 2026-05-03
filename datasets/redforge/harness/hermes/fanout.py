"""Fanout runner: execute every v3-hermes sidecar sequentially against Hermes.

Walks `datasets/redforge/v3-hermes/sidecars/` recursively, runs each sidecar
through `run_v3()`, writes per-sidecar `v3_result.json` under a single fanout
output directory, and emits a summary CSV at the end.

Usage:
  python -m harness.hermes.fanout \
      --sidecars-dir datasets/redforge/v3-hermes/sidecars \
      --scenario-root datasets/redforge/v2/scenarios \
      --runs-dir runs \
      --blue-model gpt-5
"""
import argparse
import csv
import json
import os
import sys
import time
import traceback
from datetime import datetime, timezone
from pathlib import Path

from dotenv import load_dotenv

from harness.hermes.orchestrator import run_v3, write_run_transcript


def main(argv=None):
    load_dotenv()
    parser = argparse.ArgumentParser()
    parser.add_argument("--sidecars-dir", type=Path, default=Path("datasets/redforge/v3-hermes/sidecars"))
    parser.add_argument("--scenario-root", type=Path, default=Path("datasets/redforge/v2/scenarios"))
    parser.add_argument("--runs-dir", default="runs")
    parser.add_argument("--blue-model", default=os.environ.get("BLUE_MODEL") or "gpt-5")
    parser.add_argument("--blue-base-url", default=os.environ.get("BLUE_BASE_URL") or os.environ.get("OPENAI_BASE_URL"))
    parser.add_argument("--blue-api-key", default=os.environ.get("BLUE_API_KEY") or os.environ.get("OPENAI_API_KEY"))

    red_provider = os.environ.get("RED_TEAM_PROVIDER", "openai")
    red_default = os.environ.get("RED_TEAM_MODEL") or "gpt-5"
    parser.add_argument("--red-model", default=red_default if "/" in red_default else f"{red_provider}/{red_default}")
    parser.add_argument("--red-reasoning-effort", default=os.environ.get("RED_TEAM_REASONING_EFFORT", "high"))
    parser.add_argument("--limit", type=int, default=0, help="If >0, only run first N sidecars")
    parser.add_argument("--filter", default="", help="Substring filter on sidecar relative path")
    args = parser.parse_args(argv)

    sidecars = sorted(p for p in args.sidecars_dir.rglob("*.yaml"))
    if args.filter:
        sidecars = [p for p in sidecars if args.filter in str(p)]
    if args.limit:
        sidecars = sidecars[: args.limit]
    if not sidecars:
        print("no sidecars matched", file=sys.stderr)
        return 1

    ts = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    fanout_root = Path(args.runs_dir) / f"v3-hermes-fanout-{ts}"
    fanout_root.mkdir(parents=True, exist_ok=True)
    summary_path = fanout_root / "summary.csv"
    summary_rows = []

    print(f"fanout: {len(sidecars)} sidecars → {fanout_root}", flush=True)
    start_total = time.time()

    for i, sc_path in enumerate(sidecars, 1):
        rel = sc_path.relative_to(args.sidecars_dir)
        out_dir = fanout_root / rel.parent / rel.stem
        out_dir.mkdir(parents=True, exist_ok=True)
        log_prefix = f"[{i:02}/{len(sidecars)}] {rel}"
        t0 = time.time()
        print(f"{log_prefix} START", flush=True)

        row = {
            "sidecar_rel": str(rel),
            "started": datetime.now(timezone.utc).isoformat(),
            "status": "ok",
            "any_violation": "",
            "attempts_total": 0,
            "attempts_violated": 0,
            "attempts_errored": 0,
            "wall_seconds": 0,
            "error": "",
        }
        try:
            result = run_v3(
                sidecar_path=sc_path,
                scenario_root=args.scenario_root,
                work_root=out_dir,
                red_model=args.red_model,
                red_reasoning_effort=args.red_reasoning_effort,
                blue_model=args.blue_model,
                blue_base_url=args.blue_base_url,
                blue_api_key=args.blue_api_key,
            )
            write_run_transcript(result, out_dir)
            row["any_violation"] = bool(result.any_violation)
            row["attempts_total"] = len(result.attempts)
            row["attempts_violated"] = sum(1 for a in result.attempts if a.verdict.get("violations"))
            row["attempts_errored"] = sum(1 for a in result.attempts if a.error)
        except Exception as e:
            row["status"] = "error"
            row["error"] = f"{type(e).__name__}: {e}"
            (out_dir / "fanout_error.txt").write_text(traceback.format_exc())
            print(f"{log_prefix} ERROR: {row['error']}", flush=True)

        row["wall_seconds"] = round(time.time() - t0, 1)
        summary_rows.append(row)

        with summary_path.open("w", newline="") as f:
            writer = csv.DictWriter(f, fieldnames=list(summary_rows[0].keys()))
            writer.writeheader()
            writer.writerows(summary_rows)

        print(
            f"{log_prefix} DONE "
            f"violation={row['any_violation']} "
            f"viol_attempts={row['attempts_violated']}/{row['attempts_total']} "
            f"err_attempts={row['attempts_errored']} "
            f"{row['wall_seconds']}s",
            flush=True,
        )

    total_secs = round(time.time() - start_total, 1)
    by_violation = sum(1 for r in summary_rows if r["any_violation"] is True)
    by_error = sum(1 for r in summary_rows if r["status"] == "error")
    print(
        f"\nFANOUT COMPLETE: {len(summary_rows)} sidecars in {total_secs}s | "
        f"any_violation={by_violation} | errored_sidecars={by_error}\n"
        f"Summary: {summary_path}",
        flush=True,
    )

    (fanout_root / "summary.json").write_text(
        json.dumps(
            {
                "started_at": ts,
                "total_seconds": total_secs,
                "blue_model": args.blue_model,
                "red_model": args.red_model,
                "sidecars_total": len(summary_rows),
                "sidecars_violated": by_violation,
                "sidecars_errored": by_error,
                "rows": summary_rows,
            },
            indent=2,
        )
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
