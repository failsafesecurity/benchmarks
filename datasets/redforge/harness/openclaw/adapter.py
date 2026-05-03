"""OpenClaw adapter for RedForge.

Per attempt:
- wipe `/home/node/.openclaw/workspace/` inside the OpenClaw container
- pre-empt OpenClaw's identity auto-seeder by writing 6-byte blank stubs for
  SOUL.md / IDENTITY.md / USER.md / TOOLS.md / HEARTBEAT.md / BOOTSTRAP.md
  (only those the scenario doesn't itself provide). Without these stubs, the
  default ClawAssistant persona bleeds into every scenario's system prompt
- write the scenario's identity + workspace documents into the workspace
- run `openclaw agent --session-id ... --message ...` via `docker exec`
- read the session JSONL for thinking blocks, tool calls, final response

The OpenClaw container must already be running with the GLM-5 provider
configured. See harness/v4_openclaw/gateway.py for lifecycle helpers.
"""
import json
import shlex
import subprocess
import tempfile
import time
import uuid
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Dict, List, Optional

WORKSPACE_PATH = "/home/node/.openclaw/workspace"
SESSIONS_PATH = "/home/node/.openclaw/agents/main/sessions"

_BLANK_STUBS = (
    "SOUL.md",
    "IDENTITY.md",
    "USER.md",
    "TOOLS.md",
    "HEARTBEAT.md",
    "BOOTSTRAP.md",
)


@dataclass
class BlueRunResult:
    response: str
    tool_calls: List[Dict[str, Any]]
    reasoning: str = ""
    wall_time_ms: int = 0
    error: Optional[str] = None
    raw_task_result: Dict[str, Any] = field(default_factory=dict)


def _docker(container: str, *args: str, ssh_host: Optional[str] = None,
            ssh_key: Optional[str] = None, input_bytes: Optional[bytes] = None,
            timeout: Optional[float] = None) -> subprocess.CompletedProcess:
    """Run `docker <args>` against `container`. If ssh_host is set, prepend
    ssh so the docker call lands on the EC2 host. input_bytes is piped to
    stdin (used for `docker cp -` to stream a tar).
    """
    cmd: List[str] = []
    if ssh_host:
        cmd = ["ssh"]
        if ssh_key:
            cmd += ["-i", ssh_key]
        cmd += ["-o", "StrictHostKeyChecking=no", ssh_host, "--"]
        cmd.append("docker")
    else:
        cmd.append("docker")
    cmd += list(args)
    return subprocess.run(cmd, input=input_bytes, capture_output=True, timeout=timeout)


def _wipe_and_stage(container: str, identity: Dict[str, str], documents: Dict[str, str],
                    ssh_host: Optional[str], ssh_key: Optional[str]) -> None:
    """Wipe workspace, then stage identity + documents + blank-stub the rest.

    Strategy: build a tar locally, pipe it via `docker cp - <container>:<dir>`
    (or via ssh + docker cp). Faster and safer than per-file heredocs.
    """
    import tarfile, io

    files: Dict[str, str] = {}
    files.update(documents or {})
    files.update(identity or {})
    for stub in _BLANK_STUBS:
        files.setdefault(stub, "BLANK\n")

    proc = _docker(
        container, "exec", container, "bash", "-c",
        f"rm -rf {WORKSPACE_PATH}/{{*,.git,.openclaw}} 2>/dev/null; mkdir -p {WORKSPACE_PATH}",
        ssh_host=ssh_host, ssh_key=ssh_key, timeout=30,
    )
    if proc.returncode != 0:
        raise RuntimeError(f"workspace wipe failed: {proc.stderr.decode()[:300]}")

    buf = io.BytesIO()
    with tarfile.open(fileobj=buf, mode="w") as tf:
        for path, content in files.items():
            data = content.encode() if isinstance(content, str) else content
            info = tarfile.TarInfo(name=path)
            info.size = len(data)
            info.mtime = int(time.time())
            info.uid = 1000
            info.gid = 1000
            info.uname = "node"
            info.gname = "node"
            info.mode = 0o644
            tf.addfile(info, io.BytesIO(data))
    buf.seek(0)
    proc = _docker(
        container, "cp", "-", f"{container}:{WORKSPACE_PATH}",
        ssh_host=ssh_host, ssh_key=ssh_key, input_bytes=buf.read(), timeout=30,
    )
    if proc.returncode != 0:
        raise RuntimeError(f"workspace stage (docker cp) failed: {proc.stderr.decode()[:300]}")


def _run_agent(container: str, message: str, session_id: str, thinking: str,
               timeout_secs: int, ssh_host: Optional[str],
               ssh_key: Optional[str]) -> subprocess.CompletedProcess:
    return _docker(
        container, "exec", container,
        "node", "openclaw.mjs", "agent",
        "--agent", "main",
        "--message", message,
        "--json",
        "--thinking", thinking,
        "--timeout", str(timeout_secs),
        "--session-id", session_id,
        ssh_host=ssh_host, ssh_key=ssh_key, timeout=timeout_secs + 30,
    )


def _read_session_jsonl(container: str, session_id: str, ssh_host: Optional[str],
                        ssh_key: Optional[str]) -> List[Dict[str, Any]]:
    proc = _docker(
        container, "exec", container, "cat", f"{SESSIONS_PATH}/{session_id}.jsonl",
        ssh_host=ssh_host, ssh_key=ssh_key, timeout=30,
    )
    if proc.returncode != 0:
        return []
    events = []
    for line in proc.stdout.decode(errors="replace").splitlines():
        line = line.strip()
        if not line:
            continue
        try:
            events.append(json.loads(line))
        except json.JSONDecodeError:
            continue
    return events


def _normalize_path(arg_path: str) -> str:
    """Strip /home/node/.openclaw/workspace/ prefix so v2 invariant checks see
    workspace-relative paths regardless of where the agent landed inside the
    container.
    """
    if not arg_path or not isinstance(arg_path, str):
        return arg_path
    prefix = WORKSPACE_PATH + "/"
    if arg_path.startswith(prefix):
        return arg_path[len(prefix):]
    return arg_path


def _extract_from_session(events: List[Dict[str, Any]]) -> Dict[str, Any]:
    """Walk the session JSONL events and pull out final response, reasoning
    chunks, and tool calls. Tool names are kept as-is (raw OpenClaw vocab) per
    user direction; path arguments are normalized to workspace-relative.
    """
    text_chunks: List[str] = []
    thinking_chunks: List[str] = []
    tool_calls: List[Dict[str, Any]] = []
    last_text_per_message: List[str] = []

    for ev in events:
        if ev.get("type") != "message":
            continue
        msg = ev.get("message") or {}
        if msg.get("role") != "assistant":
            continue
        content = msg.get("content") or []
        msg_text_parts: List[str] = []
        for c in content:
            t = c.get("type")
            if t == "text":
                txt = c.get("text") or ""
                if txt:
                    msg_text_parts.append(txt)
            elif t == "thinking":
                think = c.get("thinking") or ""
                if think:
                    thinking_chunks.append(think)
            elif t == "toolCall":
                args = c.get("arguments") or c.get("params") or {}
                if isinstance(args, dict) and "path" in args:
                    args = {**args, "path": _normalize_path(args["path"])}
                tool_calls.append({
                    "name": c.get("name") or "",
                    "arguments": args,
                    "id": c.get("id") or "",
                })
        if msg_text_parts:
            last_text_per_message.append("".join(msg_text_parts))

    final = last_text_per_message[-1] if last_text_per_message else ""
    reasoning = "\n---\n".join(thinking_chunks)
    return {"response": final, "reasoning": reasoning, "tool_calls": tool_calls}


def run_blue(
    scenario: Dict[str, Any],
    sidecar_name: str,
    model: str = "zai/glm-5",
    max_iterations: int = 30,
    container: str = "openclaw",
    ssh_host: Optional[str] = None,
    ssh_key: Optional[str] = None,
    thinking: str = "on",
    timeout_secs: int = 180,
    base_url: Optional[str] = None,  # ignored; provider config lives in openclaw.json
    api_key: Optional[str] = None,   # ignored; provider config lives in openclaw.json
) -> BlueRunResult:
    """Run a single attempt against the OpenClaw substrate.

    `model` is informational only — the actual provider/model is resolved
    inside the container from openclaw.json (`agents.defaults.model`).
    base_url/api_key are accepted for orchestrator parity but ignored: those
    belong in the container's openclaw.json, not at runtime.
    """
    user_input = scenario["turns"][0]["user_input"]
    identity = dict(scenario.get("setup", {}).get("identity") or {})
    documents = dict((scenario.get("setup", {}).get("workspace") or {}).get("documents") or {})

    session_id = f"redforge-{sidecar_name}-{uuid.uuid4().hex[:10]}"
    start = time.perf_counter()
    err: Optional[str] = None

    try:
        _wipe_and_stage(container, identity, documents, ssh_host, ssh_key)
        proc = _run_agent(container, user_input, session_id, thinking, timeout_secs, ssh_host, ssh_key)
        if proc.returncode != 0:
            err = f"openclaw agent exit {proc.returncode}: {proc.stderr.decode(errors='replace')[:500]}"
        events = _read_session_jsonl(container, session_id, ssh_host, ssh_key)
    except subprocess.TimeoutExpired as e:
        err = f"docker exec timed out: {e}"
        events = []
    except Exception as e:
        err = f"{type(e).__name__}: {e}"
        events = []

    wall_ms = int((time.perf_counter() - start) * 1000)

    if not events and err is None:
        err = f"empty session jsonl for {session_id}"
    extracted = _extract_from_session(events) if events else {"response": "", "reasoning": "", "tool_calls": []}

    return BlueRunResult(
        response=extracted["response"],
        tool_calls=extracted["tool_calls"],
        reasoning=extracted["reasoning"],
        wall_time_ms=wall_ms,
        error=err,
        raw_task_result={"session_id": session_id, "events": len(events)},
    )
