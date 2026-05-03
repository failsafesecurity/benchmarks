"""CLI: run a sidecar against the nearai-bench binary on Ironclaw.

Usage:
  python -m harness.ironclaw.run \
      --sidecar datasets/redforge/sidecars/signals/query-high-confidence.yaml \
      --scenario-root datasets/redforge/scenarios \
      --bench-binary /path/to/nearai-bench \
      --runs-dir runs \
Reasoning capture goes through Ironclaw's `ironclaw::llm::reasoning`
tracing target (nearai/ironclaw#3129) and the bench-side
`tracing::Subscriber::Layer` in src/instrumented_llm.rs that writes it
into the structured tasks.jsonl row.
"""
import argparse
import os
import sys
from datetime import datetime, timezone
from pathlib import Path

from dotenv import load_dotenv

from .orchestrator import run_v2, write_run_transcript


def main(argv=None):
    load_dotenv()

    parser = argparse.ArgumentParser(description="Run a RedForge v2 sidecar.")
    parser.add_argument("--sidecar", required=True, type=Path)
    parser.add_argument(
        "--scenario-root",
        type=Path,
        default=Path("datasets/redforge/v2/scenarios"),
    )
    parser.add_argument("--bench-binary", required=True)
    parser.add_argument(
        "--bench-config",
        required=True,
        help="absolute path to suites/structured-data.toml",
    )
    parser.add_argument("--runs-dir", default="runs")
    parser.add_argument("--timeout-secs", type=int, default=180)

    provider = os.environ.get("RED_TEAM_PROVIDER", "openai")
    default_model = os.environ.get("RED_TEAM_MODEL") or "gpt-5"
    parser.add_argument(
        "--red-model",
        default=default_model if "/" in default_model else f"{provider}/{default_model}",
    )
    parser.add_argument(
        "--red-reasoning-effort",
        default=os.environ.get("RED_TEAM_REASONING_EFFORT", "high"),
    )
    args = parser.parse_args(argv)

    ts = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    run_dir = Path(args.runs_dir) / f"v2-{args.sidecar.stem}-{ts}"

    result = run_v2(
        sidecar_path=args.sidecar,
        scenario_root=args.scenario_root,
        bench_binary=args.bench_binary,
        bench_config=args.bench_config,
        work_root=run_dir,
        red_model=args.red_model,
        red_reasoning_effort=args.red_reasoning_effort,
        timeout_secs=args.timeout_secs,
    )
    out = write_run_transcript(result, run_dir)
    print(f"v2 result: {out}")
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
