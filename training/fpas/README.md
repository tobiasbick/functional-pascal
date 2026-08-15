# Functional Pascal training dataset

This directory contains a reproducible, local-first supervised fine-tuning
dataset for Functional Pascal coding assistants. It is derived from the
repository's examples, regression tests, and implemented documentation. The
FPAS sources remain authoritative; the JSONL files are generated artifacts.

## Files

- `generate_dataset.py` scans the repository and writes deterministic splits.
- `validate_dataset.py` checks the JSONL schema and split invariants.
- `data/train.jsonl`, `data/validation.jsonl`, and `data/test.jsonl` contain
  conversational `messages` records accepted by Hugging Face TRL and Unsloth.
- `manifests/dataset-v1.json` records the generator version, source counts,
  split counts, and relative source paths without machine-specific metadata.

Regenerate the artifacts from the repository root:

```text
python training/fpas/generate_dataset.py
python training/fpas/validate_dataset.py
```

The generator uses only the Python standard library. It does not call an API,
upload files, or include absolute paths. The split is deterministic and based
on a stable digest of each relative source path. Test data is held out from
training and is intended for compiler-backed evaluation.

## Unsloth

In Unsloth Studio, select the generated local dataset and fine-tune an
instruction model with LoRA or QLoRA. The records use the Hugging Face
conversational format:

```json
{"messages":[{"role":"user","content":"..."},{"role":"assistant","content":"..."}]}
```

Keep the `test` split out of training. Evaluate generated FPAS with `fpas
check`, `fpas build`, or the relevant `fpas test` command after training.

The source material is licensed under the repository license. Any publication
to a model or dataset hub must be reviewed separately; this repository does
not upload anything automatically.
