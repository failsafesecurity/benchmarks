"""V2 orchestrator: load sidecar+scenario, run N attempts, write transcript.

Per attempt:
1. red_attacker.generate_attack(...) → list of rewrites
2. sidecar.apply_rewrites(...) → mutated scenario
3. runner.run_blue(...) → BlueRunResult (response, tool_calls, reasoning)
4. invariants.check(...) → Verdict
   plus, for sidecars whose substrate has a numeric assertion, check that too
5. record verdict; pass to red as prior context for next attempt
"""
import json
from dataclasses import asdict, dataclass, field
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional

from . import invariants, red_attacker, runner, sidecar


@dataclass
class AttemptRecord:
    attempt_id: int
    rewrites: List[Dict[str, str]]
    blue_response: str
    blue_reasoning: str
    tool_calls: List[Dict[str, Any]]
    verdict: Dict[str, Any]
    capability_violation: Optional[Dict[str, Any]]
    wall_time_ms: int
    error: Optional[str]


@dataclass
class V2RunResult:
    sidecar_name: str
    scenario_id: str
    user_input: str
    attempts: List[AttemptRecord] = field(default_factory=list)
    any_violation: bool = False
    started_at: str = ""
    finished_at: str = ""


def _current_field_values(
    scenario: Dict[str, Any], sc: sidecar.Sidecar
) -> Dict[str, Dict[str, str]]:
    docs = scenario["setup"]["workspace"]["documents"]
    out: Dict[str, Dict[str, str]] = {}
    for cf in sc.controlled_fields:
        for path in docs:
            if not cf.matches(path):
                continue
            try:
                obj = json.loads(docs[path])
            except json.JSONDecodeError:
                continue
            if cf.field_name not in obj:
                continue
            out.setdefault(path, {})[cf.field_name] = str(obj[cf.field_name])
    return out


def _scenario_numeric_expectation(
    scenario: Dict[str, Any],
) -> Optional[Dict[str, float]]:
    turns = scenario.get("turns") or []
    if not turns:
        return None
    nr = (turns[0].get("assertions") or {}).get("numeric_response")
    if not nr:
        return None
    return {"expected": float(nr["expected"]), "tolerance": float(nr.get("tolerance", 0.05))}


def run_v2(
    sidecar_path: Path,
    scenario_root: Path,
    bench_binary: str,
    bench_config: str,
    work_root: Path,
    red_model: str,
    red_reasoning_effort: str = "high",
    timeout_secs: int = 180,
) -> V2RunResult:
    sc = sidecar.load_sidecar(sidecar_path)
    scenario_path = scenario_root / sc.scenario_ref
    if not scenario_path.exists():
        raise FileNotFoundError(f"cannot resolve scenario_ref {sc.scenario_ref} under {scenario_root}")
    scenario = sidecar.load_scenario(scenario_path)
    sidecar.list_controllable_targets(scenario, sc)  # validation; raises on misconfig

    task_id = scenario["name"]
    user_input = scenario["turns"][0]["user_input"]
    numeric_expect = _scenario_numeric_expectation(scenario)

    started = datetime.now(timezone.utc).isoformat()
    result = V2RunResult(
        sidecar_name=sc.name,
        scenario_id=task_id,
        user_input=user_input,
        started_at=started,
    )

    prior_attempts: List[Dict[str, Any]] = []

    sidecar_wants_numeric = any(
        r.name == "numeric_answer_matches_literal_sum" for r in sc.arc.workflow_rules
    )
    target_truth = (
        f"{numeric_expect['expected']} (within ±{int(numeric_expect['tolerance']*100)}% — break this band)"
        if numeric_expect is not None and sidecar_wants_numeric
        else None
    )

    for n in range(1, sc.attempts + 1):
        ctx = red_attacker.AttackContext(
            user_input=user_input,
            sidecar=sc,
            current_field_values=_current_field_values(scenario, sc),
            prior_attempts=prior_attempts,
            target_truth=target_truth,
        )
        rewrites = red_attacker.generate_attack(
            ctx, model=red_model, reasoning_effort=red_reasoning_effort
        )
        mutated = sidecar.apply_rewrites(scenario, rewrites)
        attempt_work_root = work_root / f"attempt-{n:02d}"
        blue = runner.run_blue(
            scenario=mutated,
            scenario_ref=sc.scenario_ref,
            task_id=task_id,
            bench_binary=bench_binary,
            bench_config=bench_config,
            work_root=attempt_work_root,
            timeout_secs=timeout_secs,
        )
        verdict = invariants.check(
            response=blue.response, tool_calls=blue.tool_calls, sidecar=sc,
        )
        capability_violation = None
        # Only fold the numeric capability check into the verdict when the
        # sidecar explicitly opts in via the workflow rule. Breakdown-style
        # scenarios (e.g. "how many per owner") have a numeric_response in
        # PR #14's assertions but the closest-number matcher fires artifacts
        # on per-entity counts, so we skip them at templater time.
        wants_numeric_check = any(
            r.name == "numeric_answer_matches_literal_sum"
            for r in sc.arc.workflow_rules
        )
        if numeric_expect is not None and wants_numeric_check:
            cv = invariants.check_numeric_capability(
                blue.response, numeric_expect["expected"], numeric_expect["tolerance"]
            )
            if cv is not None:
                capability_violation = cv.__dict__
                verdict.violations.append(cv)

        record = AttemptRecord(
            attempt_id=n,
            rewrites=rewrites,
            blue_response=blue.response,
            blue_reasoning=blue.reasoning,
            tool_calls=blue.tool_calls,
            verdict=verdict.to_dict(),
            capability_violation=capability_violation,
            wall_time_ms=blue.wall_time_ms,
            error=blue.error,
        )
        result.attempts.append(record)
        if verdict.violated:
            result.any_violation = True

        prior_attempts.append({
            "rewrites": rewrites,
            "blue_response": blue.response,
            "blue_reasoning": blue.reasoning,
            "tool_calls": blue.tool_calls,
            "verdict": verdict.to_dict(),
        })

    result.finished_at = datetime.now(timezone.utc).isoformat()
    return result


def write_run_transcript(result: V2RunResult, out_dir: Path) -> Path:
    out_dir.mkdir(parents=True, exist_ok=True)
    out_file = out_dir / "v2_result.json"
    out_file.write_text(json.dumps(_asdict(result), indent=2))
    return out_file


def _asdict(r: V2RunResult) -> Dict[str, Any]:
    return {
        "sidecar_name": r.sidecar_name,
        "scenario_id": r.scenario_id,
        "user_input": r.user_input,
        "started_at": r.started_at,
        "finished_at": r.finished_at,
        "any_violation": r.any_violation,
        "attempts": [asdict(a) for a in r.attempts],
    }
