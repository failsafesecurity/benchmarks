"""Fanout runner: execute every sidecar sequentially against OpenClaw on EC2.

Walks `redforge/sidecars/` recursively (the sidecar set is shared across
substrates), runs each sidecar through `run_v4()`, writes per-sidecar
`v4_result.json` under a single fanout output directory, and emits a summary
CSV at the end.

Container lifecycle: the openclaw container is assumed to be already running
on the SSH host. Fanout verifies via `/healthz` before starting; if the
container is down, it bails. Reusing the same container across all 52×5
attempts saves the per-call boot cost.

Usage (run on EC2 from the repo root):
  python -m harness.openclaw.fanout \
      --sidecars-dir redforge/sidecars \
      --scenario-root . \
      --runs-dir runs \
      --container openclaw

Or from laptop driving EC2 (slower but works):
  python -m harness.openclaw.fanout --ssh-host ubuntu@... --ssh-key ~/.ssh/agentbench.pem
"""
import argparse
import csv
import json
import os
import subprocess
import sys
import time
import traceback
from datetime import datetime, timezone
from pathlib import Path

from dotenv import load_dotenv

from harness.openclaw.orchestrator import run_v4, write_run_transcript


def _check_container(container: str, ssh_host=None, ssh_key=None) -> str:
    """Return 'ok' if the container is running and gateway healthz responds,
    else a short error string.
    """
    cmd = []
    if ssh_host:
        cmd = ["ssh"]
        if ssh_key:
            cmd += ["-i", ssh_key]
        cmd += ["-o", "StrictHostKeyChecking=no", ssh_host, "--"]
    cmd += [
        "bash", "-c",
        f"docker inspect --format '{{{{.State.Running}}}}' {container} 2>/dev/null && "
        f"docker exec {container} curl -sf http://127.0.0.1:18789/healthz 2>/dev/null",
    ]
    try:
        out = subprocess.run(cmd, capture_output=True, timeout=20, text=True)
    except subprocess.TimeoutExpired:
        return "container check timed out"
    if out.returncode != 0:
        return f"container check failed: {out.stderr.strip()[:200]}"
    text = out.stdout
    if "true" not in text:
        return f"container not running: {text[:200]}"
    # If the gateway responds with HTTP 2xx (`curl -sf` returncode 0 above),
    # treat that as live. Older openclaw builds returned `{"ok":true}` JSON,
    # but newer images serve the SPA index.html on any path. Both are fine —
    # the gateway is up and reachable.
    return "ok"


def main(argv=None):
    load_dotenv()
    parser = argparse.ArgumentParser()
    parser.add_argument("--sidecars-dir", type=Path, default=Path("redforge/sidecars"))
    parser.add_argument("--scenario-root", type=Path, default=Path("."))
    parser.add_argument("--runs-dir", default="runs")
    parser.add_argument("--blue-model", default=os.environ.get("BLUE_MODEL") or "zai/glm-5")
    parser.add_argument("--container", default=os.environ.get("OPENCLAW_CONTAINER", "openclaw"))
    parser.add_argument("--ssh-host", default=os.environ.get("OPENCLAW_SSH_HOST"))
    parser.add_argument("--ssh-key", default=os.environ.get("OPENCLAW_SSH_KEY"))
    parser.add_argument("--thinking", default="on")
    parser.add_argument("--timeout-secs", type=int, default=180)

    red_provider = os.environ.get("RED_TEAM_PROVIDER", "openai")
    red_default = os.environ.get("RED_TEAM_MODEL") or "gpt-5"
    parser.add_argument("--red-model", default=red_default if "/" in red_default else f"{red_provider}/{red_default}")
    parser.add_argument("--red-reasoning-effort", default=os.environ.get("RED_TEAM_REASONING_EFFORT", "high"))
    parser.add_argument("--limit", type=int, default=0, help="If >0, only run first N sidecars")
    parser.add_argument("--filter", default="", help="Substring filter on sidecar relative path")
    parser.add_argument("--shard", default="", help="i/N — pick every Nth sidecar starting at i (1-indexed)")
    parser.add_argument("--out-dir", default="", help="Override output directory (used to merge parallel shards)")
    parser.add_argument("--skip-existing", action="store_true", help="Skip sidecars where v4_result.json already exists in out-dir")
    args = parser.parse_args(argv)

    health = _check_container(args.container, args.ssh_host, args.ssh_key)
    if health != "ok":
        print(f"openclaw container preflight failed: {health}", file=sys.stderr)
        return 2
    print(f"openclaw container '{args.container}' healthy", flush=True)

    sidecars = sorted(p for p in args.sidecars_dir.rglob("*.yaml"))
    if args.filter:
        sidecars = [p for p in sidecars if args.filter in str(p)]
    if args.shard:
        try:
            shard_i, shard_n = (int(x) for x in args.shard.split("/"))
        except Exception:
            print(f"--shard must be i/N, got {args.shard!r}", file=sys.stderr)
            return 2
        if not (1 <= shard_i <= shard_n):
            print(f"--shard {args.shard} out of range", file=sys.stderr)
            return 2
        sidecars = [p for k, p in enumerate(sidecars) if k % shard_n == (shard_i - 1)]
    if args.limit:
        sidecars = sidecars[: args.limit]
    if not sidecars:
        print("no sidecars matched", file=sys.stderr)
        return 1

    ts = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    if args.out_dir:
        fanout_root = Path(args.out_dir)
    else:
        fanout_root = Path(args.runs_dir) / f"v4-openclaw-fanout-{ts}"
    fanout_root.mkdir(parents=True, exist_ok=True)
    shard_tag = f"-shard{args.shard.replace('/','of')}" if args.shard else ""
    summary_path = fanout_root / f"summary{shard_tag}.csv"
    summary_rows = []

    print(f"fanout: {len(sidecars)} sidecars → {fanout_root}", flush=True)
    start_total = time.time()

    for i, sc_path in enumerate(sidecars, 1):
        rel = sc_path.relative_to(args.sidecars_dir)
        out_dir = fanout_root / rel.parent / rel.stem
        log_prefix = f"[{i:02}/{len(sidecars)}] {rel}"
        if args.skip_existing and (out_dir / "v4_result.json").exists():
            print(f"{log_prefix} SKIP (existing)", flush=True)
            continue
        out_dir.mkdir(parents=True, exist_ok=True)
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
            result = run_v4(
                sidecar_path=sc_path,
                scenario_root=args.scenario_root,
                work_root=out_dir,
                red_model=args.red_model,
                red_reasoning_effort=args.red_reasoning_effort,
                blue_model=args.blue_model,
                container=args.container,
                ssh_host=args.ssh_host,
                ssh_key=args.ssh_key,
                thinking=args.thinking,
                timeout_secs=args.timeout_secs,
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
                "container": args.container,
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
