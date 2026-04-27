"""Shell-out wrapper for PR #14's nearai-bench Rust binary, with reasoning tap.

Per attempt:
- write the mutated scenario JSON into a temp dataset directory matching
  PR #14's expected layout (datasets/trajectory/v1/structured-data/<entity>/<scenario>.json)
- mark t0
- run `nearai-bench run --suite structured-data --task-ids <id> --results-dir <tmp>`
  with REDFORGE_DUMP_RAW_LLM=1 set so our patched ironclaw dumps reasoning
- mark t1
- read the JSONL trace produced under <tmp>/ironclaw/<run-uuid>/tasks.jsonl
- tail the docker logs / dump file between t0 and t1 for blue's reasoning
"""
import json
import os
import shutil
import subprocess
import time
from dataclasses import dataclass, field
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional

from .dump_tail import fetch_reasoning


@dataclass
class BlueRunResult:
    response: str
    tool_calls: List[Dict[str, Any]]      # each: {name, arguments?}
    reasoning: str = ""
    wall_time_ms: int = 0
    error: Optional[str] = None
    raw_task_result: Dict[str, Any] = field(default_factory=dict)


def materialize_dataset(
    scenario: Dict[str, Any], scenario_ref: str, dataset_root: Path
) -> Path:
    """Write the (mutated) scenario into a PR #14-shaped dataset tree.

    `scenario_ref` is a path like "scenarios/signals/query-high-confidence.json"
    (sidecar's pointer into our agentbench layout). PR #14 expects
    "datasets/trajectory/v1/structured-data/<entity>/<scenario>.json", so we
    rewrite the prefix.
    """
    parts = Path(scenario_ref).parts
    # Strip a leading "scenarios" wrapper if present.
    if parts and parts[0] == "scenarios":
        parts = parts[1:]
    rel = Path(*parts)
    out_dir = dataset_root / "datasets" / "trajectory" / "v1" / "structured-data"
    out_path = out_dir / rel
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(scenario, indent=2))
    return out_path


def run_blue(
    scenario: Dict[str, Any],
    scenario_ref: str,
    task_id: str,
    bench_binary: str,
    work_root: Path,
    bench_config: str,
    container: Optional[str] = None,
    timeout_secs: int = 180,
) -> BlueRunResult:
    """Run a single PR #14 task end-to-end and capture (response, tool_calls, reasoning)."""
    work_root = work_root.resolve()
    work_root.mkdir(parents=True, exist_ok=True)
    dataset_root = work_root / "dataset"
    if dataset_root.exists():
        shutil.rmtree(dataset_root)
    dataset_root.mkdir(parents=True)
    materialize_dataset(scenario, scenario_ref, dataset_root)

    results_dir = work_root / "results"
    if results_dir.exists():
        shutil.rmtree(results_dir)
    results_dir.mkdir(parents=True)

    env = os.environ.copy()
    env["REDFORGE_DUMP_RAW_LLM"] = "1"

    cmd = [
        bench_binary,
        "run",
        "--suite", "trajectory",
        "--config", str(bench_config),
        "--task-ids", task_id,
        "--results-dir", str(results_dir),
        "--timeout-secs", str(timeout_secs),
    ]

    t0 = datetime.now(timezone.utc)
    start = time.perf_counter()
    try:
        proc = subprocess.run(
            cmd,
            cwd=str(dataset_root),
            env=env,
            capture_output=True,
            text=True,
            timeout=timeout_secs + 30,
        )
    except subprocess.TimeoutExpired as e:
        wall_ms = int((time.perf_counter() - start) * 1000)
        return BlueRunResult(
            response="", tool_calls=[], wall_time_ms=wall_ms,
            error=f"bench binary timeout: {e}",
        )
    t1 = datetime.now(timezone.utc)
    wall_ms = int((time.perf_counter() - start) * 1000)

    if proc.returncode != 0:
        return BlueRunResult(
            response="", tool_calls=[], wall_time_ms=wall_ms,
            error=f"bench exit {proc.returncode}: {proc.stderr[-500:]}",
        )

    task_record = _read_task_record(results_dir, task_id)
    response = task_record.get("response", "") or ""
    tool_calls = task_record.get("trace", {}).get("tool_calls", []) or []

    # PR #14's JSONL trace records tool_calls as {name, duration_ms, success} only;
    # arguments are stripped. Recover them from the [redforge-dump] raw_response
    # stream, where each LLM call's tool_calls[].function.arguments is preserved.
    # We merge by ordinal — the dump order matches the trace order since both are
    # produced by the same agent loop.
    dump_tool_calls = _parse_tool_calls_from_stderr(proc.stderr or "")
    for i, tc in enumerate(tool_calls):
        if i < len(dump_tool_calls) and dump_tool_calls[i].get("name") == tc.get("name"):
            tc["arguments"] = dump_tool_calls[i].get("arguments")

    if container:
        reasoning = fetch_reasoning(container, t0, t1)
    else:
        reasoning = _parse_reasoning_from_stderr(proc.stderr or "")

    return BlueRunResult(
        response=response,
        tool_calls=tool_calls,
        reasoning=reasoning,
        wall_time_ms=wall_ms,
        error=None,
        raw_task_result=task_record,
    )


def _parse_tool_calls_from_stderr(stderr: str) -> List[Dict[str, Any]]:
    """Extract tool_calls (with arguments) from [redforge-dump] raw_response lines.

    Each LLM call's response includes choices[].message.tool_calls[] with shape
    {id, type, function: {name, arguments}}. Arguments are JSON-encoded strings;
    we decode them so invariant checks can inspect them as dicts.
    Returns a flat list in dump order.
    """
    out: List[Dict[str, Any]] = []
    for line in stderr.splitlines():
        if "[redforge-dump]" not in line:
            continue
        idx = line.find("raw_response=")
        if idx < 0:
            continue
        try:
            payload = json.loads(line[idx + len("raw_response="):])
        except json.JSONDecodeError:
            continue
        for choice in payload.get("choices", []):
            for tc in (choice.get("message") or {}).get("tool_calls") or []:
                fn = tc.get("function") or {}
                args_raw = fn.get("arguments")
                args: Any = args_raw
                if isinstance(args_raw, str):
                    try:
                        args = json.loads(args_raw)
                    except json.JSONDecodeError:
                        args = args_raw
                out.append({"name": fn.get("name"), "arguments": args})
    return out


def _parse_reasoning_from_stderr(stderr: str) -> str:
    """Extract reasoning_content from [redforge-dump] lines emitted by our patched
    rig adapter. Each line: `[redforge-dump] path=... model=... raw_response=<json>`.
    """
    chunks: List[str] = []
    for line in stderr.splitlines():
        if "[redforge-dump]" not in line:
            continue
        idx = line.find("raw_response=")
        if idx < 0:
            continue
        try:
            payload = json.loads(line[idx + len("raw_response="):])
        except json.JSONDecodeError:
            continue
        for choice in payload.get("choices", []):
            msg = choice.get("message") or {}
            rc = msg.get("reasoning_content")
            if rc:
                chunks.append(rc)
    return "\n---\n".join(chunks)


def _read_task_record(results_dir: Path, task_id: str) -> Dict[str, Any]:
    """Find tasks.jsonl under results_dir (any depth) and return the row for task_id.

    PR #14's bench writes either results_dir/<run-uuid>/tasks.jsonl or, depending on
    framework config, results_dir/<framework>/<run-uuid>/tasks.jsonl. Walk both shapes.
    """
    candidates = sorted(results_dir.rglob("tasks.jsonl"), key=lambda p: p.stat().st_mtime, reverse=True)
    for jsonl in candidates:
        for line in jsonl.read_text().splitlines():
            row = json.loads(line)
            if row.get("task_id") == task_id:
                return row
    return {}
