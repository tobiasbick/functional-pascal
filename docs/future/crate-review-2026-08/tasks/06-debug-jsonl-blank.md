# Task 06 — Blank JSONL lines must not kill the debug session

Status: open
Severity: P1
Difficulty: easy
Language gate: no
Depends on: none

## Goal

Empty or whitespace-only JSONL lines are ignored. A later valid command still runs. `fpas debug --protocol jsonl --commands` must not exit 0 after dropping the rest of the file because of a blank line.

## Spec

Debugger JSONL docs under `docs/pascal/tools/` (`debugger-jsonl.md`). Blank lines in command files are normal for humans and agents.

## Bug

`crates/fpas-debug/src/jsonl/server.rs` — `handle_line` sends any non-object parse failure through `fatal_request`, which sets `ServerStatus::Terminated`. `serve_script` then returns `Ok(())`.

## Fix

If the trimmed line is empty, skip it. Real parse errors on non-empty lines can stay as `invalid_request` **without** terminating the session, unless the existing protocol explicitly says fatal for malformed JSON. Prefer: blank → ignore; malformed JSON object → recoverable error response, continue. Only keep fatal for true transport/session death.

Read how other invalid requests are handled in the same file and match that.

## Tests

Add a JSONL test that joins requests with `\n\n` (blank line in the middle) and asserts the second command still executes. Existing tests join with `\n` only — do not weaken them.

## Verify

```text
cargo test -p fpas-debug
cargo fmt
```

## Done when

- Blank lines are skipped.
- A typo on a non-empty line does not need to stay fatal if you found a recoverable path; do not make *every* error fatal.
- Docs: add one sentence to the JSONL command-file page if blank lines are now specified; otherwise unchanged if already implied.
