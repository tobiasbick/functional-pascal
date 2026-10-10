# Reviewed scope

This is the coverage record for completed reviews. The
[next review](README.md) starts with an empty scope and finding list.
Package implementation status remains in the [central plan](../README.md).
Reviewed coverage records an inspection of the listed areas, not a guarantee
that later changes need no further review.

## Reviewed packages

The completed review covered the following packages and their completed work
packages, including implementation, regression coverage and current documentation.

| Completed package | Inspected/exercised areas | Related findings |
| --- | --- | --- |
| AP01 | Current spelling, references, keyword/block consistency | D01, G01 |
| AP02 | Shared diagnostics, CLI JSON, project errors, real LSP transport | T01, T02, T04, D01 |
| AP04 | Consumed calls, discard type/capture proofs, unit interfaces | C03, G01 |
| AP05 | Alias resolution, shadowing, editor migration, import diagnostics | T02; no new alias semantic defect confirmed |
| AP06 | All 78 catalog entries, both argument forms, lowering/runtime, editor chains | T03, T04, P01 |
| AP07 | Boolean grammar, precedence, short circuit, Bits and rejection coverage | No new functional defect confirmed in inspected paths |
| AP09 | Named/generic/variant calls, written evaluation order, callable restrictions | Cross-unit positive/negative probes passed |
| AP10 | Typed fields/defaults, aliases, visibility, unit reuse, const/capture interaction | C04, P01 |
| AP11 | Individual declarations, whole-file type collection, finite graphs | C05, D02 |
| AP13 | Terminators, named closers, case arms, recovery, docs and embedded fixtures | D01, H02 |
| AP14 | Removed properties, member resolution and migration | P01; no new removal defect confirmed |
| AP16 | Computed/static const, writable bindings, initialization and captures | C03, C04, D03, H01 |
| AP17 | Storage roots, aliases, reference modes, forwarding, capture lifetimes | C02 |
| AP20 | Recursive patterns, coverage, constant identity, is/while bindings, runtime matching | C01, C03, C04, H02 |

Cross-package checks covered independently compiled units, facade aliases,
fresh and reused sidecars, named argument evaluation, typed record defaults,
imported mutable routine values, same-root alias rejection, computed constants
and imported callable discard proofs.

## Completed review points

All 22 findings from the completed syntax review are resolved.

| Reviewed | Finding | Priority | Resolved point |
| --- | --- | --- | --- |
| [x] | C01 | P1 | enum comparison patterns use resolved constant identity |
| [x] | C02 | P2 | sibling nested calls preserve transitive captures and reference lifetimes |
| [x] | C03 | P2 | pattern bindings preserve callable task restrictions and discard proofs |
| [x] | C04 | P2 | static record fields and derived constants contribute evaluated pattern values across units |
| [x] | C05 | P2 | shared finite type graphs use one worklist solution with bounded dependency work |
| [x] | C06 | P2 | generated entry symbols are isolated from source declarations while debugger names retain program identity |
| [x] | T01 | P2 | LSP project failures retain original records, source attribution and absent positions |
| [x] | T02 | P2 | root-program import failures retain the authoritative path and unit-name span |
| [x] | T03 | P3 | incomplete native chains preserve recursive types, declaration-local aliases and callback results |
| [x] | H01 | P2 | workflow fixtures verify the exact diagnostic token range |
| [x] | H02 | P2 | debugger-host fixtures use current syntax and pass the full host suite |
| [x] | D01 | P2 | positive examples and diagnostic corrections use implemented AP13 syntax |
| [x] | D02 | P2 | current handbook snippets repeat the keyword for each type declaration |
| [x] | D03 | P2 | the formatter control-flow example uses a mutable loop binding |
| [x] | D04 | P2 | the formatter documents implemented built-in generic forms |
| [x] | D05 | P3 | the long-import example matches canonical formatter output |
| [x] | D06 | P2 | complete introductory examples import Std.Console |
| [x] | G01 | P2 | the formal keyword set matches the implemented reserved words |
| [x] | G02 | P2 | the formal task-call grammar includes supported postfix targets |
| [x] | P01 | P3 | completed package descriptions agree with the final implemented forms |
| [x] | T04 | P3 | string bounds hints use native calls and account for empty strings |
| [x] | L01 | P3 | the cryptography index links to the existing future-work page |

The review also covered Rust module ownership, reuse of the shared parser,
public documentation and handbook links. No independent hard Rust architecture
finding was confirmed; the parser reuse recommendation is included in T03.

The debugger follow-ups are reviewed and complete: named calls, typed record
construction, explicit reference calls and live reference assignment. Their
implemented rules are documented in the
[debugger handbook](../../../pascal/tools/debugger.md).

## Completed implementation verification

The follow-up checked the completed repair implementations and resolved:

- Recursive capture metadata, reference lifetimes, task boundaries and discard
  proofs, including shadowing and shared mutable cells (C02/C03).
- Unsaved dependency correction and recovery without stale disk diagnostics (T01).
- Callable values versus invoked results in native completion (T03).
- Detached postfix procedure calls and reference-argument rejection (G02).
- Calls through array elements and dictionary entries, including evaluation order
  and retained/detached tasks.
- LSP shutdown with an unanswered file-watch registration request.

Fourteen regression tests cover these repairs. The latest verification passed
format checking and the build. The complete Rust workspace run had 3,970 passed,
2 failed and 0 ignored tests across 203 result groups, with no exclusions.
The two unchanged VM socket-test failures were
`tcp_backpressure_can_time_out_and_cancel_without_closing` (a successful write
instead of cancellation) and
`write_deadline_is_not_reset_by_retries_and_preserves_socket_timeouts`
(32 ms instead of 30 ms). The watcher and shutdown regressions passed.
The earlier reconciliation also recorded an independent Clippy blocker in
`crates/fpas-unit/src/object/mod.rs`; Clippy was not rerun in the follow-up.
