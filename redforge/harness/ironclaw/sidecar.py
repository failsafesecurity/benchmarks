"""Sidecar loader and field-rewrite engine for RedForge v2.

A sidecar is a YAML file declaring which fields of a PR #14 scenario are red-
controlled, what ARC invariants must hold, and the cross-cutting checks. It
sits next to (not inside) PR #14's scenario JSONs so PR #14 stays untouched.

Field rewrites mutate the scenario's `setup.workspace.documents` map in-place
on a fresh copy. Each document value is itself a JSON-encoded string (PR #14's
shape), so we parse → set the field → re-serialize.
"""
import fnmatch
import json
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

import yaml


@dataclass
class ControlledField:
    path_glob: str          # e.g. "collections/signals/sig-007-*.json"
    field_name: str         # e.g. "source_message"

    def matches(self, doc_path: str) -> bool:
        return fnmatch.fnmatchcase(doc_path, self.path_glob)


@dataclass
class WorkflowRule:
    name: str
    rule: str               # natural-language; checked by invariant code below


@dataclass
class ArcInvariants:
    tools_not_used: List[str] = field(default_factory=list)
    workflow_rules: List[WorkflowRule] = field(default_factory=list)
    no_writes_outside: List[str] = field(default_factory=list)


@dataclass
class CrossCutting:
    no_markdown_image_exfil: bool = False
    no_identity_writes: bool = False
    max_iterations: Optional[int] = None


@dataclass
class Sidecar:
    name: str
    description: str
    scenario_ref: str       # path relative to dataset root
    controlled_fields: List[ControlledField]
    arc: ArcInvariants
    cross_cutting: CrossCutting
    attempts: int
    max_fields_per_attempt: int


def load_sidecar(path: Path) -> Sidecar:
    raw = yaml.safe_load(Path(path).read_text())
    cf = [
        ControlledField(path_glob=e["path"], field_name=e["field"])
        for e in raw.get("red_controlled_fields", [])
    ]
    arc_raw = raw.get("arc_invariants", {}) or {}
    workflow_raw = arc_raw.get("workflow", []) or []
    workflow = [WorkflowRule(name=r["name"], rule=r["rule"]) for r in workflow_raw]
    arc = ArcInvariants(
        tools_not_used=list((arc_raw.get("tool", {}) or {}).get("tools_not_used", [])),
        workflow_rules=workflow,
        no_writes_outside=list(
            (arc_raw.get("permission", {}) or {}).get("no_writes_outside", [])
        ),
    )
    cc_raw = raw.get("cross_cutting", {}) or {}
    cc = CrossCutting(
        no_markdown_image_exfil=bool(cc_raw.get("no_markdown_image_exfil", False)),
        no_identity_writes=bool(cc_raw.get("no_identity_writes", False)),
        max_iterations=cc_raw.get("max_iterations"),
    )
    return Sidecar(
        name=raw["name"],
        description=raw.get("description", ""),
        scenario_ref=raw["scenario_ref"],
        controlled_fields=cf,
        arc=arc,
        cross_cutting=cc,
        attempts=int(raw.get("attempts", 3)),
        max_fields_per_attempt=int(raw.get("max_fields_per_attempt", 4)),
    )


def load_scenario(path: Path) -> Dict[str, Any]:
    return json.loads(Path(path).read_text())


def list_controllable_targets(
    scenario: Dict[str, Any], sidecar: Sidecar
) -> List[Tuple[str, str]]:
    """Return all (doc_path, field) tuples this sidecar can rewrite for the scenario.

    Validates that every glob in the sidecar resolves to at least one document.
    """
    docs: Dict[str, str] = scenario["setup"]["workspace"]["documents"]
    targets: List[Tuple[str, str]] = []
    for cf in sidecar.controlled_fields:
        matched = [p for p in docs if cf.matches(p)]
        if not matched:
            raise ValueError(
                f"sidecar field {cf.path_glob}.{cf.field_name} matches no documents"
            )
        for p in matched:
            targets.append((p, cf.field_name))
    return targets


def _id_prefix(path: str) -> Optional[str]:
    """Extract the `<dir>/<entity>-<num>-` prefix from a doc path, or None."""
    p = Path(path)
    name = p.name
    parts = name.split("-", 2)
    if len(parts) < 3:
        return None
    return f"{p.parent.as_posix()}/{parts[0]}-{parts[1]}-"


def apply_rewrites(
    scenario: Dict[str, Any], rewrites: List[Dict[str, str]]
) -> Dict[str, Any]:
    """Apply red's field rewrites to a scenario, returning a deep-copied mutated dict.

    Each rewrite: {"path": "<doc-path>", "field": "<json-key>", "value": <new-value>}
    The doc value in setup.workspace.documents is a JSON-encoded string; we
    decode, set the field, re-encode preserving 2-space indent (matches PR #14).

    Red occasionally hallucinates the slug suffix of a doc path (dash → underscore,
    truncation, etc.). When the exact path isn't a doc, fall back to id-prefix
    matching so a single fuzzy hit still applies the rewrite.
    """
    out = json.loads(json.dumps(scenario))  # deep copy via roundtrip
    docs: Dict[str, str] = out["setup"]["workspace"]["documents"]
    for rw in rewrites:
        path = rw["path"]
        if path not in docs:
            prefix = _id_prefix(path)
            candidates = [p for p in docs if prefix and p.startswith(prefix)] if prefix else []
            if len(candidates) == 1:
                path = candidates[0]
            else:
                raise ValueError(
                    f"rewrite targets unknown doc path: {rw['path']!r} "
                    f"(id-prefix candidates: {candidates})"
                )
        obj = json.loads(docs[path])
        if rw["field"] not in obj:
            raise ValueError(
                f"rewrite targets unknown field {rw['field']!r} in {path}"
            )
        obj[rw["field"]] = rw["value"]
        docs[path] = json.dumps(obj, indent=2)
    return out


def materialize_scenario(scenario: Dict[str, Any], out_path: Path) -> Path:
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(scenario, indent=2))
    return out_path
