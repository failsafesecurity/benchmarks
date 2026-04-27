"""Pull blue's reasoning_content out of docker logs.

Ironclaw's rig adapter dumps each LLM response to stderr tagged
`[redforge-dump] path=<...> model=<...> raw_response=<json>` when
`REDFORGE_DUMP_RAW_LLM` is set on the container. We tail those lines
between two timestamps and extract `choices[].message.reasoning_content`.

No-ops gracefully if `docker` isn't available (e.g. running locally
without the EC2 setup).
"""
import json
import subprocess
from datetime import datetime, timezone


def _fmt(t: datetime) -> str:
    return t.astimezone(timezone.utc).strftime("%Y-%m-%dT%H:%M:%S.%fZ")


def fetch_reasoning(
    container: str, since: datetime, until: datetime, timeout_s: float = 10.0
) -> str:
    try:
        proc = subprocess.run(
            ["docker", "logs", "--since", _fmt(since), "--until", _fmt(until), container],
            capture_output=True,
            text=True,
            timeout=timeout_s,
        )
    except (FileNotFoundError, subprocess.TimeoutExpired):
        return ""
    text = (proc.stdout or "") + (proc.stderr or "")
    chunks = []
    for line in text.splitlines():
        if "[redforge-dump]" not in line:
            continue
        idx = line.find("raw_response=")
        if idx < 0:
            continue
        try:
            data = json.loads(line[idx + len("raw_response="):])
        except json.JSONDecodeError:
            continue
        for choice in data.get("choices") or []:
            rc = (choice.get("message") or {}).get("reasoning_content")
            if rc:
                chunks.append(rc)
    return "\n\n".join(chunks)
