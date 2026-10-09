# Completed syntax package review

Reviewed revision: `f8d1b0ddc7ca6e93a6fb8c3053058065810eb505`.
The checkout was clean before the audit. The audit added only this report
directory and left existing implementation, specifications, examples, tests and
completion checkboxes unchanged. Repair status is recorded in the finding index.
This is a findings report, not a package-completion
index or a claim about which historical commit introduced every defect.

**Audit result at the reviewed revision: 22 confirmed findings or grouped
migration defects: 1 P1, 16 P2 and 5 P3.** The strongest finding was a checked
enum pattern that executed the wrong comparison and could panic in a case
accepted as exhaustive. The audit's complete Rust suite was green, but the full
editor-host suite was red and contained additional
invalid fixtures hidden behind its first failed assertion.

## Reports

- [Compiler and semantics](compiler-and-semantics.md): six confirmed compiler,
  checker, capture and matching defects, with reproductions and regression gaps.
- [Tooling and tests](tooling-and-tests.md): diagnostic transport, completion,
  and verified repairs to workflow positions and debugger-host sources.
- [Documentation and specifications](documentation-and-specs.md): incomplete
  migrations, invalid examples, formal grammar discrepancies and stale claims.
- [Rust standards](rust-standards.md): the separate architecture/standards axis;
  no independent hard violation confirmed, one recommendation tied to T03.

P1 means incorrect execution of an accepted program and should be addressed
first. P2 means a reproducible correctness, specification, migration or
verification defect requiring correction. P3 means narrower tooling, wording,
formatting or navigation defects. Grouped examples are counted once per shared
issue, with their affected locations enumerated in the detailed reports.

## Repair order

Follow this numbered order before starting another language package. Use the
Done column in the finding index to skip completed repairs within this order.
Mark a finding complete after its fix and regression coverage are verified, and
record required check results and independent validation blockers under Repair
verification:

1. [C01](compiler-and-semantics.md#c01): enum comparison patterns use resolved
   constant identity. Regressions cover `is`, nested `case`, shadowing, qualified
   and parenthesized constants, and actual enum members.
2. [C02](compiler-and-semantics.md#c02): sibling nested references preserve
   transitive captures and their declaration identity. Regressions cover valid
   call chains, shadowing, and rejection of escaping or spawned reference wrappers.
3. [C03](compiler-and-semantics.md#c03): pattern bindings preserve callable
   task restrictions and discard proofs. Regressions cover `is`, `case`, `while`,
   nested payloads, guards, captures, and scalar controls.
4. [H01](tooling-and-tests.md#h01) and [H02](tooling-and-tests.md#h02): correct
   workflow token ranges and debugger-host sources are verified by the complete
   editor-host suite. The
   [debugger argument diagnostic](../../compiler-panic-followups.md#reference-parameters-in-debugger-calls-and-assignments)
   rejects missing explicit `var` arguments before invocation, with Rust and
   DAP regressions proving that the callee does not execute.
5. [x] Full debugger support from the
   [compiler follow-ups](../../compiler-panic-followups.md#completed-debugger-repairs): named
   arguments, typed record construction, reference mutation, and
   `var` argument passing, with the coverage specified in those entries.

The initial repair sequence, including the debugger follow-ups, is complete.
The remaining findings retain their listed
priorities; this sequence does not close or reclassify them.

## Finding index

**Repair status: C01-C05 and H01/H02 are resolved; 15 findings remain open.
The debugger follow-ups are complete. The next open compiler finding is C06.** The Done column
tracks repairs, independently of package-completion checkboxes and the audit's
original severity counts.

| Done | Finding | Priority | Problem |
| --- | --- | --- | --- |
| [x] | [C01](compiler-and-semantics.md#c01) | P1 | resolved: enum comparison patterns use resolved constant identity |
| [x] | [C02](compiler-and-semantics.md#c02) | P2 | resolved: sibling nested calls preserve transitive captures and reference lifetimes |
| [x] | [C03](compiler-and-semantics.md#c03) | P2 | resolved: pattern bindings preserve callable task restrictions and discard proofs |
| [x] | [C04](compiler-and-semantics.md#c04) | P2 | resolved: static record fields and derived constants contribute evaluated pattern values across units |
| [x] | [C05](compiler-and-semantics.md#c05) | P2 | resolved: shared finite type graphs use one worklist solution with bounded dependency work |
| [ ] | [C06](compiler-and-semantics.md#c06) | P2 | A valid program sharing its name with a declaration fails object compilation |
| [ ] | [T01](tooling-and-tests.md#t01) | P2 | LSP project failures discard the stable diagnostic record |
| [ ] | [T02](tooling-and-tests.md#t02) | P2 | Root-program import failures lose an available path and source span |
| [ ] | [T03](tooling-and-tests.md#t03) | P3 | Native completion fails for an incomplete chain over nested Result success types |
| [x] | [H01](tooling-and-tests.md#h01) | P2 | resolved: workflow fixtures verify the exact diagnostic token range |
| [x] | [H02](tooling-and-tests.md#h02) | P2 | resolved: debugger-host fixtures use current syntax and pass the full host suite |
| [ ] | [D01](documentation-and-specs.md#d01) | P2 | The AP13 documentation migration leaves invalid positive examples and corrections |
| [ ] | [D02](documentation-and-specs.md#d02) | P2 | AP11 grouped declarations remain in current language and formatter documentation |
| [ ] | [D03](documentation-and-specs.md#d03) | P2 | The formatter's complete control-flow example mutates a const binding |
| [ ] | [D04](documentation-and-specs.md#d04) | P2 | The current formatter specification advertises unimplemented generic record types |
| [ ] | [D05](documentation-and-specs.md#d05) | P3 | The documented long-import golden is not formatter output |
| [ ] | [D06](documentation-and-specs.md#d06) | P2 | Two complete introductory examples omit required Console imports |
| [ ] | [G01](documentation-and-specs.md#g01) | P2 | The formal keyword set omits reserved discard |
| [ ] | [G02](documentation-and-specs.md#g02) | P2 | The formal task-call grammar excludes supported postfix targets |
| [ ] | [P01](documentation-and-specs.md#p01) | P3 | Completed package result text describes features already removed by other completed packages |
| [ ] | [T04](documentation-and-specs.md#t04) | P3 | Runtime string bounds hints still recommend a removed free-call spelling |
| [ ] | [L01](documentation-and-specs.md#l01) | P3 | Cryptography index links outside the documentation tree |

The architecture recommendation in the standards report is not an additional
finding. L01 is an incidental broken documentation link found in the wider sweep;
its origin is not attributed to the syntax work. C06 also lacks a historical
introduction attribution. Proposed/open packages are not counted as defects for
being unimplemented.

## Scope and package coverage

The reference point is the [central plan](../README.md), its completed package
READMEs/work files, the [development process](../development-process.md),
[shared constraints](../shared-constraints.md), current `docs/pascal/`, and
the [formal grammar](../../../specs/grammar.ebnf). Three independent reviewers
covered semantics/runtime, syntax/docs and standards/tooling; the primary
reviewer ran broad verification, checked links, reproduced principal failures
and reconciled the findings.

This table records audit coverage, not a second completion checklist.

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

Cross-package probes used independently compiled units, a facade type alias,
and both fresh and reused sidecars. They covered named argument evaluation,
typed record defaults, imported mutable routine values, same-root alias
rejection, computed constant rejection and imported callable discard proofs.
Those exercised paths passed; they do not cancel the distinct failures above.

## Repair verification

### C01

C01 is implemented and covered by five new compiler execution tests. The
pattern handbook explicitly describes enum-member shadowing by constants.

| Check | C01 repair result |
| --- | --- |
| `cargo fmt --all --check` | Passed |
| `cargo build --offline` | Passed |
| `cargo test --offline -p fpas-compiler enum_comparisons` | Passed: 5 regression tests |
| C01 runtime control through the built CLI | Check and run exit 0; output is `matched`, then `blue` |
| `fpas test --std-lib lib tests/suite.fpasprj` | Passed: 488 passed, 1 intentional skip, 0 failed |
| Rust workspace, serial verification | 3,776 passed and 1 failed across 202 result groups; the LSP watcher test was excluded after a separate timeout |
| Changed Markdown documents | All 114 local links and anchors resolve; whitespace check passed |

The complete Rust workspace is not green. These independent blockers remain
outside C01; their implementation and tests are unchanged:

- [LSP watcher registration](../../../../crates/fpas-lsp/tests/protocol.rs):
  `initialized_registers_source_and_manifest_file_watchers` hangs in the full
  run and in an isolated run bounded to 45 seconds. The final workspace command
  was `cargo test --offline --workspace --no-fail-fast -- --skip initialized_registers_source_and_manifest_file_watchers --test-threads=1`.
- [VM socket timeout assertion](../../../../crates/fpas-vm/src/vm/hosted/net/connections/cancellable_write/tests.rs):
  `write_deadline_is_not_reset_by_retries_and_preserves_socket_timeouts` expects
  exactly 30 ms but observes 32 ms in this environment.

Local network fixtures required network permission in this environment; their
initial sandbox failures are not compiler regressions. A transient JSONL debugger
failure passed both an isolated serial retry and the final serial workspace run.

### C02

C02 corrects implementation of the existing capture and reference-lifetime
contract. Five new compiler execution tests and five semantic tests cover
transitive storage, shadowing, escaping wrappers, and callable capabilities.
The closure and reference-parameter handbooks describe the implemented rules.

| Check | C02 repair result |
| --- | --- |
| `cargo fmt --all --check` | Passed |
| `cargo build --offline` | Passed |
| `cargo test --offline -p fpas-sema -p fpas-compiler transitive_captures --no-fail-fast` | Passed: 10 regression tests; also passed in the workspace run |
| C02 direct-call control through the built CLI | Check and run exit 0; runtime prints `1` |
| C02 returned-reference wrapper through the built CLI | Check and run reject it with FP3030, without an internal compiler error |
| `fpas test --std-lib lib tests/suite.fpasprj` | Passed: 488 passed, 1 intentional skip, 0 failed |
| Rust workspace, serial verification | 3,785 passed and 2 failed across 202 result groups; the LSP watcher test was excluded after a separate timeout |
| Changed Markdown documents | All local links and anchors resolve; whitespace check passed |

The workspace command was
`cargo test --offline --workspace --no-fail-fast -- --skip initialized_registers_source_and_manifest_file_watchers --test-threads=1`.
Compiler and semantic test groups are green. The unchanged LSP watcher test
again times out after 45 seconds. The unchanged VM socket-timeout assertion
again observes 32 ms instead of 30 ms. The other VM failure is
[`tcp_backpressure_can_time_out_and_cancel_without_closing`](../../../../crates/fpas-vm/src/vm/hosted/net/connections/cancellable_write/tests.rs):
its cancellation assertion observes `Ok(65536)` instead of
`Err("Network write cancelled")`, both in the workspace run and in an isolated
serial retry. These independent checks remain open; the complete Rust workspace
is not green.

### C03

C03 corrects implementation of the existing pattern, closure, and discard
contracts. Shared pattern binding preserves known capture metadata while
respecting scalar types and generic constraints. Closure capture identity and
debugger provenance distinguish multiple names from one pattern, and `is`
declaration spans agree between semantic analysis and lowering. The pattern,
closure, and discard handbooks describe the implemented rules.

| Check | C03 repair result |
| --- | --- |
| `cargo fmt --all --check` | Passed |
| `cargo build --offline` | Passed |
| `cargo test --offline -p fpas-sema -p fpas-compiler --no-fail-fast` | Passed: 835 tests, including 15 new C03 regression tests |
| C03 task-bound controls through the built CLI | `if`, `case`, and `while` controls reject checking and execution with FP3016 |
| C03 task-free control through the built CLI | Check and run exit 0; runtime prints `42` |
| `fpas test --std-lib lib tests/suite.fpasprj` | Passed: 488 passed, 1 intentional skip, 0 failed |
| Rust workspace, serial verification | 3,800 passed and 2 failed across 202 result groups; the known LSP watcher hang was excluded |
| Changed Markdown documents | All local links and anchors resolve; whitespace check passed |

The workspace command was
`cargo test --offline --workspace --no-fail-fast -- --skip initialized_registers_source_and_manifest_file_watchers --test-threads=1`.
All 15 C03 regressions, all 10 C02 regressions, and all five C01 regressions pass
in this run. Compiler, semantic, and debugger test groups are green. The same
two unchanged VM socket assertions described under [C02](#c02) fail with the
same observations. The LSP watcher remains excluded after its two previously
confirmed 45-second timeouts. These independent blockers remain open; the
complete Rust workspace is not green.

### H01/H02

H01/H02 correct tooling test fixtures and verify debugger argument-mode
diagnostics. Workflow checks derive both diagnostic range endpoints from the
fixture token, use native selection paths, and wait for newly created manifests
to appear in the editor index. Debugger sources preserve breakpoint line counts
while using current closers and explicit case patterns. The debugger handbooks
describe named calls, typed record construction, explicit reference calls, and
live reference assignment.

| Check | Current repair result |
| --- | --- |
| `cargo fmt --all --check` | Passed |
| `cargo build --offline` | Passed |
| `cargo test --offline -p fpas-debug` | Passed: 291 tests, including 29 call-evaluation tests |
| VM debugger tests | Passed: all 295 tests |
| Independently extracted debugger programs | All 33 positive programs in 31 source files pass `fpas check` |
| `node editors/vscode/scripts/run-tests.mjs` | Passed, exit 0: complete VS Code 1.137.0 host suite, including named calls, typed constructors, detached references, live reference writes, and 6 rejected reference-call controls |
| `fpas fmt --check lib apps examples tests` | Passed |
| `fpas test --std-lib lib tests/suite.fpasprj` | Passed: 488 passed, 1 intentional skip, 0 failed |
| Rust workspace, serial verification | 3,824 passed and 2 failed across 202 result groups; the known LSP watcher hang was excluded |
| Changed Markdown documents | All local links and anchors resolve; whitespace check passed |

The full host run includes external project-change watchers, formatting,
navigation, IntelliSense, semantic tools, workflow operations, debugger
execution, standard-library diagnostics, and language-client lifecycle.
Cloud verification used the official Microsoft package for VS Code 1.137.0,
checked its repository SHA-256, and ran it on a virtual display. `VSCODE_CLI=1`
preserved the test runner's configured PATH.

The Rust workspace command was
`cargo test --offline --workspace --no-fail-fast -- --skip initialized_registers_source_and_manifest_file_watchers --test-threads=1`.
All named-call, typed-construction, reference-call, and C01-C03 regressions pass.
The two unchanged VM socket assertions described under [C02](#c02) fail with
the same observations. The Rust LSP watcher test remains excluded after its
previously confirmed timeouts. The complete Rust workspace is not green.

### C04

C04 corrects the existing constant-expression and pattern-coverage contract.
Known record fields retain declaration-scope default values, survive copies and
updates, and project through nested fields or derived constants. Unit interfaces
preserve these values through aliases and facades while aggregate constants
remain immutable runtime globals. The constants and exhaustiveness handbooks,
AP10.1, AP16.1 and AP20.2 describe the implemented behavior.

| Check | C04 repair result |
| --- | --- |
| `cargo fmt --all --check` | Passed |
| `cargo build --offline` | Passed |
| `cargo test --offline -p fpas-sema -p fpas-unit -p fpas-compiler --no-fail-fast -- --test-threads=1` | Passed: 884 tests, including 13 new C04 regression tests |
| C04 control through the built CLI | Check and run exit 0 |
| Independent two-unit CLI project | Fresh and reused sidecar runs both exit 0 and print `matched`; both unit sidecars remain unchanged on reuse |
| `fpas fmt --check lib apps examples tests` | Passed |
| `fpas test --std-lib lib tests/suite.fpasprj` | Passed: 488 passed, 1 intentional skip, 0 failed |
| Rust workspace, serial verification | 3,838 passed and 1 failed across 202 result groups; the known LSP watcher hang was excluded |
| Changed Markdown documents | All local links and anchors resolve; whitespace check passed |

The workspace command was
`cargo test --offline --workspace --no-fail-fast -- --skip initialized_registers_source_and_manifest_file_watchers --test-threads=1`.
All C01-C04 regressions pass. The unchanged VM socket assertion
`write_deadline_is_not_reset_by_retries_and_preserves_socket_timeouts` observes
32 ms instead of the expected 30 ms, as recorded under [C02](#c02).
The previously failing backpressure control passes in this run. The LSP watcher
test remains excluded after its previously confirmed timeouts. The complete
Rust workspace is not green.

### C05

C05 replaces recursive re-expansion with a shared finite-construction graph and
one worklist solution across all declared types. Nominal definitions are expanded
once, finite dependencies are propagated once, and cycle witnesses follow only
non-finite paths. AP11.1 and the declaration-order handbook describe the
implementation; language rules and diagnostic codes are unchanged.

| Check | C05 repair result |
| --- | --- |
| `cargo fmt --all --check` | Passed |
| `cargo build --offline` | Passed |
| `cargo test --offline -p fpas-sema -p fpas-compiler --no-fail-fast -- --test-threads=1` | Passed: 853 tests, including 7 new C05 regression tests |
| Built CLI shared-record controls | Depths 12, 20, 22, 40 and 64 all pass `fpas check --std-lib lib` with exit 0 |
| Deterministic graph-work controls | Up to 256 nominal types; finite notifications and nominal expansions stay bounded by the graph, without timing assertions |
| `fpas fmt --check lib apps examples tests` | Passed |
| `fpas test --std-lib lib tests/suite.fpasprj` | Passed: 488 passed, 1 intentional skip, 0 failed |
| Rust workspace, serial verification | 3,844 passed and 2 failed across 202 result groups; the known LSP watcher hang was excluded |
| Changed Markdown documents | All local links and anchors resolve; whitespace check passed |

The workspace command was
`cargo test --offline --workspace --no-fail-fast -- --skip initialized_registers_source_and_manifest_file_watchers --test-threads=1`.
All C01-C05 regressions pass. The two unchanged VM network assertions described
under [C02](#c02) fail with the same observations: the cancelled backpressure
write completes with 65,536 bytes, and the socket timeout is 32 ms instead of
the expected 30 ms. The LSP watcher test remains excluded after its previously
confirmed timeouts. The complete Rust workspace is not green.

## Verification

The following results describe the original audit at the reviewed revision.

| Check | Result |
| --- | --- |
| `cargo build` | Passed |
| `cargo fmt --all --check` | Passed; non-mutating format check |
| `cargo test --workspace` | Passed: 3,776 Rust tests/doc-tests, 0 failed, 0 ignored across 202 result groups |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | Passed |
| `fpas fmt --check lib apps examples tests` | Passed |
| `fpas test --std-lib lib tests/suite.fpasprj` | Passed in a complete unrestricted rerun: 488 passed, 1 intentional skip, 0 failed (489 total) |
| `fpas check --std-lib lib` on every app/example manifest/workspace | All 23 passed |
| Native catalog execution | All 156 positional/named cases compiled/lowered; all 78 entries reached runtime, with valid-bound controls for 10 intentionally invalid fixture inputs |
| VS Code TypeScript compilation; grammar, contracts and manifest verifiers | All passed |
| `node editors/vscode/scripts/run-tests.mjs` | Failed twice, including an isolated retry: H01. Later host stages not reached |
| Extracted debugger-host source checks | 31 programs checked: 20 failed across 19 files, 11 passed (H02) |
| Handbook code fences | 446 screened; 56 complete fences checked: 41 passed, 11 expected context failures, 4 substantive failures |
| Markdown relative file links | 278 files / 1,490 links checked; one missing target (L01) |
| Literal Rust handbook paths | 887 references checked; all target files exist |
| New review documents | All 127 local links and 51 referenced source-line ranges resolve; 22 finding IDs and severity totals agree; whitespace/privacy checks passed |

Initial sandbox runs encountered filesystem permission errors, blocked loopback
sockets and an executable lock from simultaneous compiler use. Those are not
counted as product defects: affected checks were rerun outside the sandbox, and
probe executables were copied to avoid relinking conflicts. The first editor
run overlapped Clippy's copied-library refresh; the isolated retry reproduced
H01 without that transient noise. The report's conclusions use the isolated
result, not the transient missing-library message.

Logs and extra reproducers remain in ignored `.temp-data/syntax-review*`
directories. Essential source shapes, exact ownership, expected/actual behavior,
and regression recommendations are included in these Markdown reports so the
findings do not depend on preserving the raw logs. Commands assume repository
root and a CLI built from the reviewed revision; `--std-lib lib` makes library
selection explicit even when using a copied executable.

## Interpretation and limits

This is a broad review of all packages currently marked complete, backed by
source inspection, complete baseline suites where executable, and targeted
negative/boundary probes. It is not a proof that every program is correct.
Arbitrary generic substitutions, every parser-recovery input, every concurrency
schedule and other operating systems were not exhaustively tested. The finite
graph timings are bounded debug stress observations, not release benchmarks.
External URLs and all Markdown fragment anchors were not exhaustively checked.

Named debugger arguments, typed construction, and reference mutation are
complete in [compiler follow-ups](../../compiler-panic-followups.md) and do not
add to the review's finding count. Subsequent repairs and their regression
coverage are tracked in the finding index and Repair verification.
