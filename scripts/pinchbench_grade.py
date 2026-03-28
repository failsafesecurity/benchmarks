#!/usr/bin/env python3
"""
PinchBench grading runner.

Executes a Python grade() function extracted from a PinchBench task file
against a transcript and workspace directory.

Usage:
    uv run --no-project scripts/pinchbench_grade.py \
        --grade-code grade_func.py \
        --transcript transcript.json \
        --workspace /tmp/pinch-workspace

Output: JSON dict mapping criterion names to scores (0.0-1.0) on stdout.
"""

import argparse
import json
import sys
from pathlib import Path


def main():
    parser = argparse.ArgumentParser(description="PinchBench Python grader")
    parser.add_argument("--grade-code", required=True, help="Path to Python file containing grade() function")
    parser.add_argument("--transcript", required=True, help="Path to JSON transcript file")
    parser.add_argument("--workspace", required=True, help="Path to task workspace directory")
    args = parser.parse_args()

    # Load the grade function
    grade_code = Path(args.grade_code).read_text()
    namespace = {}
    try:
        exec(grade_code, namespace)  # noqa: S102
    except Exception as e:
        print(json.dumps({"_error": f"Failed to exec grade code: {e}"}))
        sys.exit(1)

    grade_func = namespace.get("grade")
    if not callable(grade_func):
        print(json.dumps({"_error": "No callable grade() function found"}))
        sys.exit(1)

    # Load transcript
    transcript = json.loads(Path(args.transcript).read_text())

    # Run grading
    try:
        scores = grade_func(transcript, args.workspace)
    except Exception as e:
        print(json.dumps({"_error": f"grade() raised: {e}"}))
        sys.exit(1)

    if not isinstance(scores, dict):
        print(json.dumps({"_error": f"grade() returned {type(scores).__name__}, expected dict"}))
        sys.exit(1)

    # Ensure all values are floats
    clean = {}
    for k, v in scores.items():
        try:
            clean[str(k)] = float(v)
        except (TypeError, ValueError):
            clean[str(k)] = 0.0

    print(json.dumps(clean))


if __name__ == "__main__":
    main()
