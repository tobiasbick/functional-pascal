"""Validate generated Functional Pascal JSONL dataset artifacts."""

from __future__ import annotations

import hashlib
import json
import re
import subprocess
from pathlib import Path

from dataset_rules import SOURCE_ROOTS, forbidden_reason


ROOT = Path(__file__).resolve().parents[2]
TRAINING = ROOT / "training" / "fpas"
DATA = TRAINING / "data"
MANIFEST = TRAINING / "manifests" / "dataset-v1.json"
SPLITS = ("train", "validation", "test")
ROLES = ["system", "user", "assistant"]
SOURCE_FILE = re.compile(r"^(?:program|unit)\s+", re.IGNORECASE)
FORBIDDEN_CODE = (
    (re.compile(r"\bStd\.WriteLn\b"), "Std.WriteLn is not an FPAS unit"),
    (re.compile(r"\.fpp\b", re.IGNORECASE), "FPAS source files use .fpas"),
    (
        re.compile(r"^\s*for\s+[A-Za-z_][A-Za-z0-9_]*\s+in\s+", re.IGNORECASE | re.MULTILINE),
        "FPAS for-in variables require a type",
    ),
)


def fail(path: Path, line_number: int, message: str) -> None:
    """Stop validation with a precise record location."""
    raise SystemExit(f"{path.name}:{line_number}: {message}")


def git_text(*args: str) -> str:
    """Return UTF-8 text from a read-only Git command."""
    return subprocess.run(
        ["git", *args],
        cwd=ROOT,
        check=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        encoding="utf-8",
    ).stdout


def current_source_tree_ids(source_roots: list[str]) -> dict[str, str]:
    """Return current Git tree identities for the declared dataset source roots."""
    return {root: git_text("rev-parse", f"HEAD:{root}").strip() for root in source_roots}


def validate() -> dict[str, int]:
    """Validate schema, FPAS identity, common syntax traps, and split isolation."""
    if not MANIFEST.is_file():
        raise SystemExit(f"missing dataset manifest: {MANIFEST.as_posix()}")
    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    source_roots = manifest.get("source_roots")
    if source_roots != list(SOURCE_ROOTS):
        raise SystemExit("manifest source_roots do not match the generator")
    if manifest.get("source_trees") != current_source_tree_ids(source_roots):
        raise SystemExit("dataset source trees differ from HEAD; regenerate the dataset")
    counts: dict[str, int] = {}
    seen: dict[str, tuple[str, int]] = {}
    for split in SPLITS:
        path = DATA / f"{split}.jsonl"
        if not path.is_file():
            raise SystemExit(f"missing dataset split: {path.as_posix()}")
        count = 0
        for line_number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
            try:
                record = json.loads(line)
            except json.JSONDecodeError as error:
                fail(path, line_number, f"invalid JSON: {error}")
            if set(record) != {"messages"}:
                fail(path, line_number, "record must contain only messages")
            messages = record.get("messages")
            if not isinstance(messages, list) or len(messages) != 3:
                fail(path, line_number, "expected exactly three messages")
            if [item.get("role") for item in messages] != ROLES:
                fail(path, line_number, "expected system/user/assistant roles")
            if any(not isinstance(item.get("content"), str) or not item["content"].strip() for item in messages):
                fail(path, line_number, "message content must be non-empty text")
            system, user, assistant = (item["content"] for item in messages)
            if "Functional Pascal" not in system or "Free Pascal" not in system or "Delphi" not in system:
                fail(path, line_number, "system message must identify FPAS and reject other Pascal dialects")
            if reason := forbidden_reason(assistant):
                fail(path, line_number, reason)
            digest = hashlib.sha256(line.encode("utf-8")).hexdigest()
            if digest in seen:
                previous_split, previous_line = seen[digest]
                fail(path, line_number, f"duplicate of {previous_split}.jsonl:{previous_line}")
            seen[digest] = (split, line_number)
            if SOURCE_FILE.search(assistant.lstrip()):
                for pattern, reason in FORBIDDEN_CODE:
                    if pattern.search(assistant):
                        fail(path, line_number, reason)
            count += 1
        if count == 0:
            raise SystemExit(f"empty dataset split: {path.name}")
        counts[split] = count

    if counts != manifest.get("split_counts"):
        raise SystemExit("manifest split counts do not match generated JSONL files")
    if manifest.get("curated_training_count", 0) < 10:
        raise SystemExit("dataset must retain the curated FPAS correction examples")
    if "reserved-keywords" not in manifest.get("curated_training_ids", []):
        raise SystemExit("dataset must retain the reserved-keywords correction example")
    return counts


if __name__ == "__main__":
    print(json.dumps(validate(), sort_keys=True))
