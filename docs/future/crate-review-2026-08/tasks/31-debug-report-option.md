# Task 31 — Remove the no-op debug `--report jsonl` option

Status: open
Severity: P3
Difficulty: easy
Language gate: no
Depends on: none

## Goal

Debugger help and accepted options match behavior. JSONL protocol output remains on stdout without a
redundant report-format flag.

## Verified cause

`cli_input/options.rs` accepts and validates `--report jsonl` in debug mode but stores no value.
`fpas debug --protocol jsonl` already writes JSONL protocol records; there is no separate debug
report renderer. Help and debugger docs include the discarded flag only in examples.

## Fix

Remove debug-mode `--report` parsing, accepted-option classification, help examples, and debugger
documentation examples. A supplied flag must fail fast as unknown with a correct invocation using
only `--protocol jsonl --commands ...`.

Do not affect `fpas test --report json` or `fpas init --report json`; those are real structured
reports owned by other subcommands.

## Tests

- Debug CLI parsing rejects `--report jsonl` with an actionable example.
- `--commands` still emits JSONL records on stdout.
- Test/init report parsing remains green.

## Verify

```text
cargo test -p fpas-cli
cargo build
cargo test --workspace
cargo fmt
```

## Done when

- No debug docs/help advertise the flag.
- No debug parser branch accepts and discards it.
- Existing real report options are unchanged.
