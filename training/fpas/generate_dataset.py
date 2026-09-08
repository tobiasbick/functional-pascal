"""Build deterministic conversational JSONL data for Functional Pascal."""

from __future__ import annotations

import hashlib
import io
import json
import subprocess
import tarfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
OUTPUT = ROOT / "training" / "fpas"
VERSION = "fpas-sft-v1"
SOURCE_ROOTS = ("docs/pascal", "examples", "tests", "apps", "lib/Std")
MAX_SOURCE_CHARS = 12_000
SYSTEM_PROMPT = (
    "You are an expert assistant for Functional Pascal (FPAS), the distinct language "
    "implemented by the fpas compiler. Follow FPAS syntax and its Std.* library exactly; "
    "do not substitute Free Pascal, Delphi, or generic Pascal conventions. When asked for "
    "source code only, return valid FPAS source code without Markdown fences or commentary."
)


CURATED_TRAINING = (
    (
        "identity",
        "What language are you assisting with, and which Pascal assumptions must you avoid?",
        "I assist with Functional Pascal (FPAS), the language implemented by the fpas compiler. "
        "FPAS is not Free Pascal, Delphi, or merely a functional coding style for standard "
        "Pascal. I use the repository's FPAS grammar and Std.* APIs, write source files with "
        "the .fpas extension, and avoid importing syntax or library units from other Pascal "
        "dialects.",
    ),
    (
        "core-rules",
        "Summarize the FPAS rules for mutable locals, parameters, typed for-in loops, and console output.",
        "Local bindings declared with `var` are immutable; use `mutable var` when a later "
        "assignment is required. Parameters use `Name: Type`; do not add standard-Pascal "
        "`var` unless the FPAS API explicitly declares a mutable parameter. A for-in loop "
        "declares the element type, for example `for Value: integer in Values do`. Import "
        "`Std.Console` and call `WriteLn` for line output; there is no `Std.WriteLn` unit.",
    ),
    (
        "hello-world",
        "Write a complete Functional Pascal hello-world program. Return only valid source code.",
        """program HelloWorld;

uses Std.Console;

begin
  WriteLn('Hello, world!')
end.""",
    ),
    (
        "sum-positive",
        "Write a complete Functional Pascal program that defines a function named SumPositive. "
        "The function accepts an array of integers and returns the sum of all positive values. "
        "Call it with [-2, 4, 0, 7] and print the result. Use the correct standard-library unit "
        "and return only valid source code.",
        """program SumPositiveExample;

uses Std.Console;

function SumPositive(Values: array of integer): integer;
begin
  mutable var Total: integer := 0;
  for Value: integer in Values do
  begin
    if Value > 0 then
    begin
      Total := Total + Value
    end
  end;
  return Total
end;

begin
  WriteLn(SumPositive([-2, 4, 0, 7]))
end.""",
    ),
    (
        "count-even",
        "Write a complete FPAS program with a CountEven function for an integer array. "
        "Print the result for [1, 2, 4, 7]. Return source code only.",
        """program CountEvenExample;

uses Std.Console;

function CountEven(Values: array of integer): integer;
begin
  mutable var Count: integer := 0;
  for Value: integer in Values do
  begin
    if Value mod 2 = 0 then
    begin
      Count := Count + 1
    end
  end;
  return Count
end;

begin
  WriteLn(CountEven([1, 2, 4, 7]))
end.""",
    ),
    (
        "immutable-local-correction",
        "Correct this FPAS idea: a local total is declared with `var` and then reassigned in "
        "a loop. Explain only the required declaration change.",
        "Declare the accumulator with `mutable var`, for example "
        "`mutable var Total: integer := 0;`. An FPAS local declared with plain `var` is "
        "immutable and cannot be reassigned.",
    ),
    (
        "for-in-correction",
        "Correct the FPAS loop `for Value in Values do`. Return only the corrected loop header.",
        "for Value: integer in Values do",
    ),
    (
        "console-import-correction",
        "Which FPAS unit must a program import to call WriteLn?",
        "Import `Std.Console` and call `WriteLn(...)`. `Std.WriteLn` is not a unit.",
    ),
    (
        "tui-background-events",
        "How does an implemented FPAS Std.Tui application start owned background work and "
        "receive application messages safely?",
        "Create a bounded `channel of TMessage`, then run the application with "
        "`TuiApplication.RunWithBackground`, passing separate framework and application update "
        "callbacks. On `TuiMsg.Started`, call `Cmd.StartBackground(Id, Work)` for one-shot work "
        "or `Cmd.ReplaceSubscription(Id, Work)` for a replaceable long-lived source. The work "
        "receives a `CancellationToken` and returns `result of boolean, string`. It sends typed "
        "messages through the channel, normally with `SendWithCancellation`. Only the serial "
        "update callbacks modify application state; failures arrive as "
        "`TuiMsg.BackgroundFailed`.",
    ),
)


def git_output(*args: str) -> bytes:
    """Return bytes from a read-only Git command."""
    result = subprocess.run(
        ["git", *args],
        cwd=ROOT,
        check=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    return result.stdout


def git_text(*args: str) -> str:
    """Return UTF-8 text from a read-only Git command."""
    return git_output(*args).decode("utf-8")


def split_for(identity: str) -> str:
    """Assign a stable identity to a train/validation/test split."""
    bucket = int(hashlib.sha256(identity.encode("utf-8")).hexdigest()[:8], 16) % 100
    if bucket < 85:
        return "train"
    if bucket < 95:
        return "validation"
    return "test"


def prompt_for(path: str, content: str) -> str:
    """Describe the source without misclassifying application files as tests."""
    is_unit = content.lstrip().lower().startswith("unit ")
    if path.startswith("docs/pascal/"):
        return f"Explain the implemented Functional Pascal topic documented in `{path}`."
    if path.startswith("tests/"):
        if is_unit:
            return f"Provide the Functional Pascal regression support unit stored in `{path}`."
        return f"Provide the Functional Pascal regression program stored in `{path}`."
    if path.startswith("examples/"):
        if is_unit:
            return f"Provide the complete Functional Pascal example unit stored in `{path}`."
        return f"Provide the complete Functional Pascal example stored in `{path}`."
    if path.startswith("apps/"):
        if is_unit:
            return f"Provide the Functional Pascal application unit stored in `{path}`."
        return f"Provide the Functional Pascal application source stored in `{path}`."
    declaration = "unit" if is_unit else "source"
    return f"Provide the implemented Functional Pascal standard-library {declaration} stored in `{path}`."


def source_files(commit: str) -> tuple[list[tuple[str, str]], list[dict[str, str]]]:
    """Read eligible source files from HEAD, never from concurrent working-tree edits."""
    included: list[tuple[str, str]] = []
    excluded: list[dict[str, str]] = []
    archive = git_output("archive", "--format=tar", commit, "--", *SOURCE_ROOTS)
    with tarfile.open(fileobj=io.BytesIO(archive), mode="r:") as snapshot:
        files = {
            member.name: snapshot.extractfile(member).read().decode("utf-8")
            for member in snapshot.getmembers()
            if member.isfile()
        }
    for path, raw_content in sorted(files.items()):
        suffix = Path(path).suffix
        if suffix not in {".md", ".fpas"}:
            continue
        if suffix == ".md" and not path.startswith("docs/pascal/"):
            continue
        if suffix == ".fpas" and ("_compile_error.fpas" in path or path.startswith("tests/manual/")):
            excluded.append({"path": path, "reason": "not a positive valid-source example"})
            continue
        content = raw_content.strip()
        if not content:
            continue
        if len(content) > MAX_SOURCE_CHARS:
            excluded.append({"path": path, "reason": f"longer than {MAX_SOURCE_CHARS} characters"})
            continue
        included.append((path, content))
    return included, excluded


def conversation(user: str, assistant: str) -> dict[str, list[dict[str, str]]]:
    """Build one Hugging Face conversational record."""
    return {
        "messages": [
            {"role": "system", "content": SYSTEM_PROMPT},
            {"role": "user", "content": user},
            {"role": "assistant", "content": assistant},
        ]
    }


def main() -> None:
    source_commit = git_text("rev-parse", "HEAD").strip()
    records: dict[str, list[dict[str, list[dict[str, str]]]]] = {
        "train": [],
        "validation": [],
        "test": [],
    }
    for _, user, assistant in CURATED_TRAINING:
        records["train"].append(conversation(user, assistant))

    sources, excluded = source_files(source_commit)
    for path, content in sources:
        split = split_for(path)
        records[split].append(conversation(prompt_for(path, content), content))

    data_dir = OUTPUT / "data"
    manifest_dir = OUTPUT / "manifests"
    data_dir.mkdir(parents=True, exist_ok=True)
    manifest_dir.mkdir(parents=True, exist_ok=True)
    for split, entries in records.items():
        with (data_dir / f"{split}.jsonl").open("w", encoding="utf-8", newline="\n") as handle:
            for entry in entries:
                handle.write(json.dumps(entry, ensure_ascii=False, separators=(",", ":")) + "\n")

    manifest = {
        "dataset": VERSION,
        "format": "huggingface-conversational-messages",
        "generator": "training/fpas/generate_dataset.py",
        "source_commit": source_commit,
        "source_roots": list(SOURCE_ROOTS),
        "maximum_source_characters": MAX_SOURCE_CHARS,
        "curated_training_count": len(CURATED_TRAINING),
        "curated_training_ids": [identity for identity, _, _ in CURATED_TRAINING],
        "source_count": len(sources),
        "excluded_source_count": len(excluded),
        "split_counts": {split: len(entries) for split, entries in records.items()},
        "sources": [path for path, _ in sources],
        "excluded_sources": excluded,
    }
    (manifest_dir / "dataset-v1.json").write_text(
        json.dumps(manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    print(json.dumps(manifest["split_counts"], sort_keys=True))


if __name__ == "__main__":
    main()
