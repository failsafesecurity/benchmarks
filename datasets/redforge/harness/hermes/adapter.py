"""Hermes (NousResearch/hermes-agent) adapter for RedForge.

Replaces v2/runner.py's subprocess-to-ironclaw dance with a direct in-process
Hermes call. Per attempt:
- stage the (mutated) scenario's documents into a temp workspace dir
- override TERMINAL_CWD so Hermes' file toolset reads/writes inside that dir
- rewrite the scenario's identity prompt so memory_* references map onto
  Hermes' file toolset (read_file / write_file / search_files)
- launch AIAgent.run_conversation(user_input)
- read result["messages"] for tool calls + reasoning, normalize back to the
  v2 vocabulary (memory_read / memory_write / memory_search) so the existing
  invariant checker works unchanged
"""
import contextlib
import json
import os
import shutil
import tempfile
import time
import uuid
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Dict, List, Optional


@dataclass
class BlueRunResult:
    response: str
    tool_calls: List[Dict[str, Any]]
    reasoning: str = ""
    wall_time_ms: int = 0
    error: Optional[str] = None
    raw_task_result: Dict[str, Any] = field(default_factory=dict)


# Hermes "file" toolset → v2 ironclaw vocabulary the invariant checker uses.
_TOOL_NAME_TO_V2 = {
    "read_file": "memory_read",
    "write_file": "memory_write",
    "search_files": "memory_search",
    "patch": "memory_write",  # patch mutates a file → counts as a write
}


def _reset_hermes_terminal_state() -> None:
    """Drop Hermes' cached terminal env so the next call rebinds to TERMINAL_CWD.

    Hermes caches a single ShellFileOperations under task_id="default" on the
    local backend (_resolve_container_task_id collapses unique session_ids to
    "default"). Without this reset, attempt N+1 reuses attempt N's env — whose
    cwd points at the now-deleted previous tempdir, so every relative path
    lookup fails with ENOENT.
    """
    try:
        from tools.file_tools import clear_file_ops_cache
        from tools.terminal_tool import _active_environments, _env_lock, _last_activity
    except Exception:
        return
    clear_file_ops_cache()
    with _env_lock:
        for env in list(_active_environments.values()):
            with contextlib.suppress(Exception):
                close = getattr(env, "close", None) or getattr(env, "shutdown", None)
                if close:
                    close()
        _active_environments.clear()
        _last_activity.clear()


def _stage_workspace(documents: Dict[str, str], dest: Path) -> None:
    """Write each `path -> content` entry under dest, creating parents."""
    for rel_path, content in documents.items():
        out = dest / rel_path
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(content)


def _adapt_identity(identity_text: str) -> str:
    """Rewrite memory_* tool references in the agent identity prompt to the
    Hermes file-toolset names. The substrate prompt is authored against
    ironclaw's memory_* vocabulary; without this swap the agent emits
    `memory_search` / `memory_write` and Hermes rejects them as unknown tools.
    """
    swaps = [
        ("memory_search", "search_files"),
        ("memory_read", "read_file"),
        ("memory_write", "write_file"),
        ("memory_tree", "search_files (with target='files')"),
    ]
    out = identity_text
    for old, new in swaps:
        out = out.replace(old, new)
    return out


def _normalize_path(arg_path: str, workspace_root: Path) -> str:
    """Strip workspace_root prefix from an absolute path so invariant checks
    see `collections/...` regardless of where the workspace landed on disk.
    """
    if not arg_path:
        return arg_path
    p = Path(arg_path)
    if p.is_absolute():
        try:
            return str(p.resolve().relative_to(workspace_root.resolve()))
        except ValueError:
            return arg_path
    return arg_path


def _extract_tool_calls(
    messages: List[Dict[str, Any]], workspace_root: Path
) -> List[Dict[str, Any]]:
    """Pull tool_calls out of assistant messages, translate names to the v2
    vocabulary, and normalize path arguments to workspace-relative form.
    """
    out: List[Dict[str, Any]] = []
    for msg in messages:
        if msg.get("role") != "assistant":
            continue
        for tc in msg.get("tool_calls") or []:
            fn = tc.get("function") or tc
            name = fn.get("name") or tc.get("name")
            args_raw = fn.get("arguments")
            if isinstance(args_raw, str):
                try:
                    args = json.loads(args_raw)
                except json.JSONDecodeError:
                    args = {"_raw": args_raw}
            else:
                args = args_raw or {}
            if isinstance(args, dict) and "path" in args:
                args = {**args, "path": _normalize_path(args["path"], workspace_root)}
            v2_name = _TOOL_NAME_TO_V2.get(name, name)
            out.append({"name": v2_name, "arguments": args, "_hermes_name": name})
    return out


def _extract_reasoning(messages: List[Dict[str, Any]]) -> str:
    """Concatenate every assistant turn's reasoning content. Hermes stores
    reasoning under `reasoning`, `reasoning_content`, or both depending on
    the provider; we collect whatever is present.
    """
    chunks: List[str] = []
    for msg in messages:
        if msg.get("role") != "assistant":
            continue
        for key in ("reasoning", "reasoning_content"):
            val = msg.get(key)
            if val:
                chunks.append(val if isinstance(val, str) else json.dumps(val))
    return "\n---\n".join(chunks)


def _extract_final_response(messages: List[Dict[str, Any]]) -> str:
    """Last assistant message with non-empty text content."""
    for msg in reversed(messages):
        if msg.get("role") != "assistant":
            continue
        content = msg.get("content")
        if isinstance(content, str) and content.strip():
            return content
        if isinstance(content, list):
            text_parts = [c.get("text", "") for c in content if isinstance(c, dict) and c.get("type") == "text"]
            if any(text_parts):
                return "".join(text_parts)
    return ""


def run_blue(
    scenario: Dict[str, Any],
    sidecar_name: str,
    model: str,
    max_iterations: int = 30,
    base_url: Optional[str] = None,
    api_key: Optional[str] = None,
) -> BlueRunResult:
    """Run a single attempt against Hermes. `scenario` is the mutated v2
    scenario dict (`setup.identity.AGENTS.md`, `setup.workspace.documents`,
    `turns[0].user_input`).
    """
    from run_agent import AIAgent  # imported lazily so non-Hermes envs can import this module

    user_input = scenario["turns"][0]["user_input"]
    identity_raw = scenario["setup"]["identity"]["AGENTS.md"]
    identity = _adapt_identity(identity_raw)
    documents = scenario["setup"]["workspace"]["documents"]

    workspace = Path(tempfile.mkdtemp(prefix=f"redforge-v3-{sidecar_name}-"))
    _stage_workspace(documents, workspace)

    prior_cwd = os.environ.get("TERMINAL_CWD")
    os.environ["TERMINAL_CWD"] = str(workspace)
    _reset_hermes_terminal_state()

    session_id = uuid.uuid4().hex
    start = time.perf_counter()
    err: Optional[str] = None
    result: Dict[str, Any] = {}
    try:
        kwargs: Dict[str, Any] = {
            "model": model,
            "max_iterations": max_iterations,
            "enabled_toolsets": ["file"],
            "ephemeral_system_prompt": identity,
            "save_trajectories": False,
            "quiet_mode": True,
            "session_id": session_id,
            "skip_context_files": True,
            "skip_memory": True,
        }
        if base_url:
            kwargs["base_url"] = base_url
        if api_key:
            kwargs["api_key"] = api_key
        agent = AIAgent(**kwargs)
        result = agent.run_conversation(user_input)
    except Exception as e:
        err = f"{type(e).__name__}: {e}"
    wall_ms = int((time.perf_counter() - start) * 1000)

    if prior_cwd is None:
        os.environ.pop("TERMINAL_CWD", None)
    else:
        os.environ["TERMINAL_CWD"] = prior_cwd

    with contextlib.suppress(Exception):
        shutil.rmtree(workspace)

    if err:
        return BlueRunResult(response="", tool_calls=[], wall_time_ms=wall_ms, error=err)

    messages = result.get("messages") or []
    response = _extract_final_response(messages) or (result.get("final_response") or "")
    tool_calls = _extract_tool_calls(messages, workspace)
    reasoning = _extract_reasoning(messages)

    return BlueRunResult(
        response=response,
        tool_calls=tool_calls,
        reasoning=reasoning,
        wall_time_ms=wall_ms,
        error=None,
        raw_task_result={"session_id": session_id, "completed": result.get("completed")},
    )
