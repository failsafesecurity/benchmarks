"""Scenario loader — pinchbench-compatible markdown + YAML-front-matter format.

File shape:

    ---
    id: <str>
    name: <str>
    category: <str>
    grading_type: adversarial
    max_turns: <int>
    max_tokens: <int>
    ...other front-matter fields...
    ---

    # Attacker System Prompt
    <text>

    # First Attacker Message
    <text>

    # Grader
    ```python
    def grade(transcript, workspace_path):
        return {"<criterion>": <0.0|1.0>}
    ```

Extra sections (Scenario, Invariant, etc.) are allowed for human readers and ignored by the loader.
"""
import re
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Callable, Dict, List

import yaml


@dataclass
class Scenario:
    id: str
    name: str
    category: str
    max_turns: int
    max_tokens: int
    attacker_system_prompt: str
    first_attacker_message: str
    grade: Callable[[List[Dict], str], Dict[str, float]]
    front_matter: Dict[str, Any] = field(default_factory=dict)
    # Maps tool-name -> {result: <json>, duration_ms?: int}. When non-empty,
    # run.py materializes this to overrides.json and points Ironclaw at it
    # via REDFORGE_OVERRIDES_PATH so blue's calls to those tools return the
    # canned content instead of hitting the real upstream API.
    tool_overrides: Dict[str, Any] = field(default_factory=dict)


_FRONT_MATTER_RE = re.compile(r"\A---\n(.*?)\n---\n(.*)", re.DOTALL)
_SECTION_RE = re.compile(r"^#\s+(.+?)\s*$", re.MULTILINE)
_CODE_BLOCK_RE = re.compile(r"```python\n(.*?)\n```", re.DOTALL)


def _split_sections(body: str) -> Dict[str, str]:
    positions = [(m.start(), m.group(1).strip()) for m in _SECTION_RE.finditer(body)]
    if not positions:
        return {}
    out: Dict[str, str] = {}
    for i, (start, title) in enumerate(positions):
        end = positions[i + 1][0] if i + 1 < len(positions) else len(body)
        content_start = body.index("\n", start) + 1
        out[title] = body[content_start:end].strip()
    return out


def load(path: Path) -> Scenario:
    raw = Path(path).read_text()
    m = _FRONT_MATTER_RE.match(raw)
    if not m:
        raise ValueError(f"{path}: missing YAML front-matter (expected leading '---')")
    front = yaml.safe_load(m.group(1)) or {}
    sections = _split_sections(m.group(2))

    for required in ("Attacker System Prompt", "First Attacker Message", "Grader"):
        if required not in sections:
            raise ValueError(f"{path}: missing required section '# {required}'")

    code_m = _CODE_BLOCK_RE.search(sections["Grader"])
    if not code_m:
        raise ValueError(f"{path}: Grader section has no ```python``` code block")

    grader_ns: Dict[str, Any] = {}
    exec(code_m.group(1), grader_ns)
    grade_fn = grader_ns.get("grade")
    if not callable(grade_fn):
        raise ValueError(f"{path}: grader code must define grade(transcript, workspace_path)")

    return Scenario(
        id=front["id"],
        name=front["name"],
        category=front["category"],
        max_turns=int(front.get("max_turns", 10)),
        max_tokens=int(front.get("max_tokens", 20000)),
        attacker_system_prompt=sections["Attacker System Prompt"],
        first_attacker_message=sections["First Attacker Message"],
        grade=grade_fn,
        front_matter=front,
        tool_overrides=dict(front.get("tool_overrides") or {}),
    )
