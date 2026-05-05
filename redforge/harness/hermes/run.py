"""CLI: run a RedForge sidecar against Hermes.

Usage (run from the repo root):
  python -m harness.hermes.run \
      --sidecar redforge/sidecars/signals/query-high-confidence.yaml \
      --scenario-root . \
      --runs-dir runs \
      --blue-model openai/gpt-5

`--scenario-root` is interpreted as the prefix for each sidecar's
`scenario_ref`. Sidecars ship with repo-rooted refs (datasets/trajectory/...
or redforge/scenarios/math/...), so passing the repo root resolves them.
"""
import argparse
import os
import sys
from datetime import datetime, timezone
from pathlib import Path

from dotenv import load_dotenv

from harness.hermes.orchestrator import run_v3, write_run_transcript


def main(argv=None):
    load_dotenv()

    parser = argparse.ArgumentParser(description="Run a RedForge sidecar against Hermes.")
    parser.add_argument("--sidecar", required=True, type=Path)
    parser.add_argument(
        "--scenario-root",
        type=Path,
        default=Path("."),
        help="Repo root. Sidecar scenario_ref is interpreted relative to this.",
    )
    parser.add_argument("--runs-dir", default="runs")

    red_provider = os.environ.get("RED_TEAM_PROVIDER", "openai")
    red_default = os.environ.get("RED_TEAM_MODEL") or "gpt-5"
    parser.add_argument(
        "--red-model",
        default=red_default if "/" in red_default else f"{red_provider}/{red_default}",
    )
    parser.add_argument(
        "--red-reasoning-effort",
        default=os.environ.get("RED_TEAM_REASONING_EFFORT", "high"),
    )

    parser.add_argument(
        "--blue-model",
        default=os.environ.get("BLUE_MODEL") or os.environ.get("OPENAI_MODEL") or "gpt-5",
    )
    parser.add_argument(
        "--blue-base-url",
        default=os.environ.get("BLUE_BASE_URL") or os.environ.get("OPENAI_BASE_URL"),
    )
    parser.add_argument(
        "--blue-api-key",
        default=os.environ.get("BLUE_API_KEY") or os.environ.get("OPENAI_API_KEY"),
    )

    args = parser.parse_args(argv)

    ts = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    run_dir = Path(args.runs_dir) / f"v3-hermes-{args.sidecar.stem}-{ts}"

    result = run_v3(
        sidecar_path=args.sidecar,
        scenario_root=args.scenario_root,
        work_root=run_dir,
        red_model=args.red_model,
        red_reasoning_effort=args.red_reasoning_effort,
        blue_model=args.blue_model,
        blue_base_url=args.blue_base_url,
        blue_api_key=args.blue_api_key,
    )
    out = write_run_transcript(result, run_dir)
    print(f"v3 result: {out}")
    print(
        f"sidecar={result.sidecar_name} scenario={result.scenario_id} "
        f"any_violation={result.any_violation} attempts={len(result.attempts)}"
    )
    for a in result.attempts:
        viols = a.verdict.get("violations") or []
        verdict_str = "VIOLATED" if viols else "HELD"
        names = ",".join(v["name"] for v in viols) if viols else "-"
        print(f"  attempt {a.attempt_id}: {verdict_str} ({names}) {a.wall_time_ms}ms")
    return 0


if __name__ == "__main__":
    sys.exit(main())
