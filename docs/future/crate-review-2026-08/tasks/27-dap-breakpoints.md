# Task 27 — DAP `setBreakpoints` must be atomic per source

Status: open
Severity: P1
Difficulty: medium
Language gate: no
Depends on: none

## Goal

If any location in a `setBreakpoints` request fails, either:

- the previous breakpoint set for that source is unchanged, **or**
- the adapter still returns one `Breakpoint` per requested line (DAP) and records every ID that was installed,

and a following `setBreakpoints` can clear them. Do **not** return a request failure after clearing old breakpoints and leaking new ones.

## Spec

DAP `setBreakpoints`: replace all breakpoints for that source; respond with one Breakpoint per requested location (`verified: false` is ok).

## Bug

`crates/fpas-debug/src/dap/server/breakpoints.rs`: old breakpoints for that source are cleared first. If a later `breakpoint.set` returns `success: false` (e.g. `line: 0`), the handler fails the whole request without recording IDs already installed. Those breakpoints stay in the VM.

## Fix

Preferred: compute the new set, then commit (clear old + set new) only on full success; on failure restore the old set.

Alternative (DAP-faithful): never fail the request; return `verified: false` per bad line, keep successful lines, clear only the old IDs you replaced.

Pick one and test it. Do not mix “fail the request” with “partially installed”.

## Tests

DAP test: mixed valid + `line: 0` (or unresolved path). After the call, either old breakpoints still hit **or** the response lists both locations and a second `setBreakpoints` with `[]` clears the VM. Today there is no mixed-list test.

## Verify

```text
cargo test -p fpas-debug
cargo fmt
```

## Done when

- No leaked VM breakpoints after a failed/partial set.
- DAP clients get a legal response shape.
- Docs unchanged unless debugger.md describes setBreakpoints failures.
