"""CLI: run a scenario end-to-end and write the transcript."""
import argparse
import json
import os
import sys
from dataclasses import asdict
from datetime import datetime, timezone
from pathlib import Path

from dotenv import load_dotenv

from .attacker import run_scenario
from .ironclaw_client import IronclawClient
from .scenario import load as load_scenario


DEFAULT_TASKS_DIR = Path("datasets/redforge/v1/tasks")


def _resolve_scenario_path(spec: str) -> Path:
    p = Path(spec)
    if p.is_file():
        return p
    for candidate in (DEFAULT_TASKS_DIR / f"{spec}.md", DEFAULT_TASKS_DIR / spec):
        if candidate.is_file():
            return candidate
    raise FileNotFoundError(
        f"cannot find scenario {spec!r} (tried as file, as {DEFAULT_TASKS_DIR}/{spec}.md)"
    )


def main(argv=None):
    load_dotenv()

    parser = argparse.ArgumentParser(description="Run a RedForge scenario.")
    parser.add_argument(
        "scenario",
        help=f"task id, filename, or path under {DEFAULT_TASKS_DIR}/",
    )
    parser.add_argument("--runs-dir", default="runs")
    args = parser.parse_args(argv)

    scenario = load_scenario(_resolve_scenario_path(args.scenario))

    client = IronclawClient(
        url=os.environ["IRONCLAW_WEBHOOK_URL"],
        secret=os.environ["IRONCLAW_WEBHOOK_SECRET"],
    )

    provider = os.environ.get("RED_TEAM_PROVIDER", "openai")
    model = os.environ.get("RED_TEAM_MODEL") or "gpt-5"
    attacker_model = model if "/" in model else f"{provider}/{model}"

    # Env caps as ceilings — scenario values win when they're tighter.
    scenario.max_turns = min(scenario.max_turns, int(os.environ.get("MAX_ATTACKER_TURNS", 10)))
    scenario.max_tokens = min(scenario.max_tokens, int(os.environ.get("MAX_ATTACKER_TOKENS", 20000)))
    reasoning_effort = os.environ.get("RED_TEAM_REASONING_EFFORT", "low")

    ts = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    run_dir = Path(args.runs_dir) / f"{scenario.id}-{ts}"
    run_dir.mkdir(parents=True, exist_ok=True)

    # Materialize tool overrides to the path Ironclaw reads. The container
    # must be started with REDFORGE_OVERRIDES_PATH pointing here (see
    # docs/tool_substitution_design.md for the env-var contract). Atomic
    # write (temp + rename) so the dispatcher never reads a half-written file.
    if scenario.tool_overrides:
        overrides_path = Path(
            os.environ.get("REDFORGE_OVERRIDES_PATH", str(run_dir / "overrides.json"))
        )
        overrides_path.parent.mkdir(parents=True, exist_ok=True)
        payload = {"overrides": scenario.tool_overrides}
        tmp = overrides_path.with_suffix(overrides_path.suffix + ".tmp")
        tmp.write_text(json.dumps(payload, indent=2))
        tmp.replace(overrides_path)
        print(f"tool_overrides: wrote {len(scenario.tool_overrides)} entries to {overrides_path}")

    result = run_scenario(
        scenario, client, attacker_model, str(run_dir), reasoning_effort=reasoning_effort
    )
    (run_dir / "result.json").write_text(json.dumps(asdict(result), indent=2))

    verdict = "VIOLATED" if result.violated else "HELD"
    print(f"{scenario.id}: {verdict} — {result.stop_reason}")
    print(f"scores: {result.scores}")
    print(f"transcript: {run_dir}/result.json")
    return 0


if __name__ == "__main__":
    sys.exit(main())
