"""Build deterministic conversational JSONL data for Functional Pascal."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
OUTPUT = ROOT / "training" / "fpas"
VERSION = "fpas-sft-v1"


def split_for(path: str) -> str:
    """Assign a relative source path to a stable train/validation/test split."""
    bucket = int(hashlib.sha256(path.encode("utf-8")).hexdigest()[:8], 16) % 100
    if bucket < 85:
        return "train"
    if bucket < 95:
        return "validation"
    return "test"


def prompt_for(path: str) -> str:
    if path.startswith("docs/"):
        return f"Explain the Functional Pascal topic documented in `{path}`."
    if path.startswith("examples/"):
        return f"Write a Functional Pascal example for the topic represented by `{path}`."
    return f"Provide a Functional Pascal regression test corresponding to `{path}`."


def source_files() -> list[tuple[str, str]]:
    roots = ("docs/pascal", "examples", "tests", "apps")
    suffixes = {".md", ".fpas"}
    files: list[tuple[str, str]] = []
    for relative_root in roots:
        for path in (ROOT / relative_root).rglob("*"):
            if path.is_file() and path.suffix in suffixes:
                relative = path.relative_to(ROOT).as_posix()
                content = path.read_text(encoding="utf-8").strip()
                if content:
                    files.append((relative, content))
    return sorted(files)


def main() -> None:
    records = {"train": [], "validation": [], "test": []}
    for path, content in source_files():
        split = split_for(path)
        records[split].append(
            {
                "messages": [
                    {"role": "system", "content": "You are an expert Functional Pascal assistant."},
                    {"role": "user", "content": prompt_for(path)},
                    {"role": "assistant", "content": content},
                ],
            }
        )

    data_dir = OUTPUT / "data"
    manifest_dir = OUTPUT / "manifests"
    data_dir.mkdir(parents=True, exist_ok=True)
    manifest_dir.mkdir(parents=True, exist_ok=True)
    for split, entries in records.items():
        with (data_dir / f"{split}.jsonl").open("w", encoding="utf-8", newline="\n") as handle:
            for entry in entries:
                handle.write(json.dumps(entry, ensure_ascii=False, separators=(",", ":")) + "\n")

    paths = [path for path, _ in source_files()]
    manifest = {
        "dataset": VERSION,
        "format": "huggingface-conversational-messages",
        "generator": "training/fpas/generate_dataset.py",
        "source_roots": ["docs/pascal", "examples", "tests", "apps"],
        "source_count": len(paths),
        "split_counts": {split: len(entries) for split, entries in records.items()},
        "sources": paths,
    }
    (manifest_dir / "dataset-v1.json").write_text(
        json.dumps(manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    print(json.dumps(manifest["split_counts"], sort_keys=True))


if __name__ == "__main__":
    main()
