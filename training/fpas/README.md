# Functional Pascal training dataset

This directory contains a reproducible, local-first supervised fine-tuning
dataset for Functional Pascal coding assistants. It combines short curated
instruction examples with implemented documentation, examples, regression
programs, application sources, and selected `Std.*` units.

The curated examples explicitly teach that FPAS is the language implemented by
the `fpas` compiler, not Free Pascal, Delphi, or generic Pascal written in a
functional style. They also cover common small-model errors: `.fpas` file names,
`Std.Console`, immutable `var` bindings, `mutable var` accumulators, typed
`for-in` loops, and the owned `Std.Tui` background-work API.

## Files

- `generate_dataset.py` reads committed sources from `HEAD` and writes the
  deterministic splits. Uncommitted working-tree changes are never ingested.
- `validate_dataset.py` checks the JSONL schema, FPAS identity, duplicate split
  records, manifest counts, and common foreign-Pascal output patterns.
- `data/train.jsonl`, `data/validation.jsonl`, and `data/test.jsonl` are the only
  dataset split files. Regeneration replaces them; it does not create a v2 copy.
- `manifests/dataset-v1.json` records the exact source commit, limits, curated
  examples, source selection, exclusions, and split counts without local paths.

Regenerate the artifacts from the repository root:

```text
python training/fpas/generate_dataset.py
python training/fpas/validate_dataset.py
```

The generator uses only committed files from `docs/pascal`, `examples`,
`tests`, `apps`, and `lib/Std`. It excludes deliberate compile-error fixtures,
manual failure demos, and individual sources longer than 12,000 characters.
This keeps invalid syntax and oversized whole-file completions out of positive
instruction examples. The split is deterministic and based on each relative
source path. The manifest's `source_commit` makes every generated snapshot
traceable.

## Unsloth

Upload `data/train.jsonl` as the training dataset and
`data/validation.jsonl` as the evaluation dataset. Keep `data/test.jsonl` out
of training for final compiler-backed checks. The files use the Hugging Face
conversational format accepted by Unsloth:

```json
{"messages":[{"role":"system","content":"..."},{"role":"user","content":"..."},{"role":"assistant","content":"..."}]}
```

After training, test both ordinary and reasoning-enabled inference. Generated
FPAS should still be checked with `fpas check`, `fpas build`, or the relevant
`fpas test` command; a falling evaluation loss does not prove that generated
programs compile.

The source material is licensed under the repository license. Publishing a
model or dataset to a hub remains a separate, manual decision; these scripts do
not upload anything.
