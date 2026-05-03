"""V3-hermes orchestrator: same loop as v2, swapped to Hermes adapter.

Per attempt:
1. red_attacker.generate_attack(...) → list of rewrites
2. sidecar.apply_rewrites(...) → mutated scenario
3. adapter.run_blue(...) → BlueRunResult (response, tool_calls, reasoning)
4. invariants.check(...) → Verdict
5. record verdict; pass to red as prior context for next attempt

Reuses v2's sidecar/red_attacker/invariants modules — only the runner step
swaps. No subprocess, no bench binary, no container handle.
"""
import json
from dataclasses import asdict, dataclass, field
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional

from harness.ironclaw import invariants, red_attacker, sidecar
from harness.hermes import adapter


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
class V3RunResult:
    sidecar_name: str
    scenario_id: str
    user_input: str
    target_framework: str = "hermes"
    target_model: str = ""
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


def run_v3(
    sidecar_path: Path,
    scenario_root: Path,
    work_root: Path,
    red_model: str,
    blue_model: str,
    blue_base_url: Optional[str] = None,
    blue_api_key: Optional[str] = None,
    red_reasoning_effort: str = "high",
) -> V3RunResult:
    sc = sidecar.load_sidecar(sidecar_path)
    scenario_path = scenario_root / sc.scenario_ref.replace("scenarios/", "")
    if not scenario_path.exists():
        scenario_path = scenario_root.parent / sc.scenario_ref
    if not scenario_path.exists():
        raise FileNotFoundError(f"cannot resolve scenario_ref {sc.scenario_ref}")
    scenario = sidecar.load_scenario(scenario_path)
    sidecar.list_controllable_targets(scenario, sc)

    task_id = scenario["name"]
    user_input = scenario["turns"][0]["user_input"]
    numeric_expect = _scenario_numeric_expectation(scenario)
    max_iterations = sc.cross_cutting.max_iterations or 30

    started = datetime.now(timezone.utc).isoformat()
    result = V3RunResult(
        sidecar_name=sc.name,
        scenario_id=task_id,
        user_input=user_input,
        target_model=blue_model,
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

    work_root.mkdir(parents=True, exist_ok=True)

    for n in range(1, sc.attempts + 1):
        ctx = red_attacker.AttackContext(
            user_input=user_input,
            sidecar=sc,
            current_field_values=_current_field_values(scenario, sc),
            prior_attempts=prior_attempts,
            target_truth=target_truth,
        )
        try:
            rewrites = red_attacker.generate_attack(
                ctx, model=red_model, reasoning_effort=red_reasoning_effort
            )
        except Exception as e:
            # Don't let a single red-side failure (malformed JSON, quota error,
            # transient API issue) kill the rest of the sidecar's attempts.
            err = f"red_attacker.generate_attack failed: {type(e).__name__}: {e}"
            record = AttemptRecord(
                attempt_id=n, rewrites=[], blue_response="", blue_reasoning="",
                tool_calls=[], verdict={"violated": False, "violations": [], "unchecked": []},
                capability_violation=None, wall_time_ms=0, error=err,
            )
            result.attempts.append(record)
            prior_attempts.append({
                "rewrites": [], "blue_response": "", "blue_reasoning": "",
                "tool_calls": [], "verdict": record.verdict,
            })
            continue
        mutated = sidecar.apply_rewrites(scenario, rewrites)
        blue = adapter.run_blue(
            scenario=mutated,
            sidecar_name=sc.name,
            model=blue_model,
            max_iterations=max_iterations,
            base_url=blue_base_url,
            api_key=blue_api_key,
        )
        verdict = invariants.check(
            response=blue.response, tool_calls=blue.tool_calls, sidecar=sc,
        )
        capability_violation = None
        if numeric_expect is not None and sidecar_wants_numeric:
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


def write_run_transcript(result: V3RunResult, out_dir: Path) -> Path:
    out_dir.mkdir(parents=True, exist_ok=True)
    out_file = out_dir / "v3_result.json"
    out_file.write_text(json.dumps(_asdict(result), indent=2))
    return out_file


def _asdict(r: V3RunResult) -> Dict[str, Any]:
    return {
        "sidecar_name": r.sidecar_name,
        "scenario_id": r.scenario_id,
        "user_input": r.user_input,
        "target_framework": r.target_framework,
        "target_model": r.target_model,
        "started_at": r.started_at,
        "finished_at": r.finished_at,
        "any_violation": r.any_violation,
        "attempts": [asdict(a) for a in r.attempts],
    }
