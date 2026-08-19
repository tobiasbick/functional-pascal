# Task 06 — Fatal JSONL protocol errors must exit nonzero

Status: open
Severity: P1
Difficulty: medium
Language gate: no
Depends on: none

## Goal

`fpas debug --protocol jsonl`, including `--commands`, returns a nonzero process result after a fatal
protocol error. A clean `disconnect` or normal program termination remains successful.

## Spec

[`debugger-jsonl.md`](../../../pascal/tools/debugger-jsonl.md) defines UTF-8 JSON Lines as one
complete JSON object per line and includes fatal `protocol_error` in the state contract. A blank line
is therefore malformed input, not an ignorable separator.

## Bug

`crates/fpas-debug/src/jsonl/server.rs::fatal_request` sets `ServerStatus::Terminated`. Both
`jsonl::serve` and `serve_script` see only that state and return `Ok(())`, the same result used for a
clean termination. The CLI consequently exits 0 after malformed JSON, a blank line, a non-object,
or an invalid request ID, even though it emitted a fatal protocol error and discarded later input.

## Fix

Represent clean versus fatal termination explicitly in `JsonlServer`; do not infer it by reparsing
the emitted JSON records. After writing the fatal `protocol_error` record, live and scripted
transports return an actionable `InvalidData`/protocol error so the CLI exits nonzero. Preserve clean
success for explicit disconnect and normal session termination.

Do not make blank lines valid JSONL and do not duplicate request parsing in the transport.

## Tests

- Command script with a blank/malformed line emits a fatal protocol record, does not execute a later
  command, and returns `Err`.
- Live transport has the same fatal result.
- Explicit disconnect and normal program completion still return `Ok(())`.
- CLI integration test asserts nonzero for a malformed `--commands` file.

## Verify

```text
cargo test -p fpas-debug
cargo test -p fpas-cli
cargo build
cargo test --workspace
cargo fmt
```

## Done when

- Fatal protocol termination cannot be reported as process success.
- Clean termination remains success.
- Input strictness remains one JSON object per line; docs unchanged.
