"""Shim 52 v2 scenarios so they run on mainline NEAR trajectory adapter
without depending on PR #14's new APIs.

Three rewrites per scenario:
  1. setup.workspace.documents → merged into setup.identity (same path→content
     map; mainline `create_seeded_workspace` already writes identity entries to
     the workspace).
  2. setup.tools → removed (harness-level tool restriction is PR-14-only;
     adversarial invariant checker validates tool calls post-hoc anyway).
  3. assertions.numeric_response → replaced/augmented with response_contains
     ["<expected_number>"] (substring match; weaker but works on mainline).
"""
from __future__ import annotations
import json
import sys
from pathlib import Path

SRC = Path("datasets/redforge/v2/scenarios")
DST = Path("datasets/redforge/v2/scenarios-shimmed")


def fmt_number(x: float) -> str:
    """Render expected number for response_contains. Avoid trailing .0 for ints."""
    if x == int(x):
        return str(int(x))
    return f"{x:g}"


def shim(scenario: dict) -> dict:
    s = json.loads(json.dumps(scenario))  # deep copy
    setup = s.setdefault("setup", {})

    identity = setup.get("identity") or {}
    workspace = setup.get("workspace") or {}
    documents = workspace.get("documents") or {}

    overlap = set(identity) & set(documents)
    if overlap:
        raise ValueError(f"identity/documents path collision: {sorted(overlap)}")
    identity.update(documents)
    setup["identity"] = identity
    setup.pop("workspace", None)

    setup.pop("tools", None)

    for turn in s.get("turns") or []:
        a = turn.get("assertions") or {}
        nr = a.pop("numeric_response", None)
        if nr is not None:
            expected_str = fmt_number(float(nr["expected"]))
            contains = a.get("response_contains") or []
            if expected_str not in contains:
                contains = [*contains, expected_str]
            a["response_contains"] = contains
            turn["assertions"] = a

    return s


def main() -> int:
    if not SRC.is_dir():
        print(f"source not found: {SRC}", file=sys.stderr)
        return 1
    DST.mkdir(parents=True, exist_ok=True)

    files = sorted(SRC.rglob("*.json"))
    n_workspace = n_tools = n_numeric = 0
    for f in files:
        scenario = json.loads(f.read_text())
        had_workspace = bool((scenario.get("setup") or {}).get("workspace", {}).get("documents"))
        had_tools = "tools" in (scenario.get("setup") or {})
        had_numeric = any(
            "numeric_response" in (t.get("assertions") or {})
            for t in scenario.get("turns") or []
        )
        shimmed = shim(scenario)
        rel = f.relative_to(SRC)
        out = DST / rel
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(json.dumps(shimmed, indent=2) + "\n")
        n_workspace += had_workspace
        n_tools += had_tools
        n_numeric += had_numeric
        print(f"  {rel}: docs={had_workspace} tools={had_tools} numeric={had_numeric}")

    print()
    print(f"Shimmed {len(files)} scenarios → {DST}")
    print(f"  workspace.documents folded into identity: {n_workspace}")
    print(f"  setup.tools removed:                      {n_tools}")
    print(f"  numeric_response → response_contains:     {n_numeric}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
