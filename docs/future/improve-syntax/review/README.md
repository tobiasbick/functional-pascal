# Completed syntax package review

Reviewed revision: `f8d1b0ddc7ca6e93a6fb8c3053058065810eb505`.
The checkout was clean before the audit. Only this report directory is added;
existing implementation, specifications, examples, tests and completion checkboxes
are unchanged. This is a current-state findings report, not a package-completion
index or a claim about which historical commit introduced every defect.

**Result: 22 confirmed findings or grouped migration defects: 1 P1, 16 P2 and
5 P3.** The strongest finding is a checked enum pattern that executes the wrong
comparison and can panic in a case accepted as exhaustive. The complete Rust
suite is green, but the full editor-host suite is red and contains additional
invalid fixtures hidden behind its first failed assertion.

## Reports

- [Compiler and semantics](compiler-and-semantics.md): six confirmed compiler,
  checker, capture and matching defects, with reproductions and regression gaps.
- [Tooling and tests](tooling-and-tests.md): diagnostic transport, completion,
  the failing host assertion and 20 invalid debugger programs in 19 source files.
- [Documentation and specifications](documentation-and-specs.md): incomplete
  migrations, invalid examples, formal grammar discrepancies and stale claims.
- [Rust standards](rust-standards.md): the separate architecture/standards axis;
  no independent hard violation confirmed, one recommendation tied to T03.

P1 means incorrect execution of an accepted program and should be addressed
first. P2 means a reproducible correctness, specification, migration or
verification defect requiring correction. P3 means narrower tooling, wording,
formatting or navigation defects. Grouped examples are counted once per shared
issue, with their affected locations enumerated in the detailed reports.

## Finding index

| Finding | Priority | Problem |
| --- | --- | --- |
| [C01](compiler-and-semantics.md#c01) | P1 | enum comparison patterns are lowered by spelling instead of resolved constant identity |
| [C02](compiler-and-semantics.md#c02) | P2 | sibling nested calls lose transitive reference captures and trigger an internal compiler error |
| [C03](compiler-and-semantics.md#c03) | P2 | pattern bindings discard callable capability metadata |
| [C04](compiler-and-semantics.md#c04) | P2 | record-derived compile-time constants do not contribute their values to pattern coverage |
| [C05](compiler-and-semantics.md#c05) | P2 | Finite construction checking expands a shared type graph exponentially |
| [C06](compiler-and-semantics.md#c06) | P2 | A valid program sharing its name with a declaration fails object compilation |
| [T01](tooling-and-tests.md#t01) | P2 | LSP project failures discard the stable diagnostic record |
| [T02](tooling-and-tests.md#t02) | P2 | Root-program import failures lose an available path and source span |
| [T03](tooling-and-tests.md#t03) | P3 | Native completion fails for an incomplete chain over nested Result success types |
| [H01](tooling-and-tests.md#h01) | P2 | The migrated workflow fixture retains the old diagnostic offset |
| [H02](tooling-and-tests.md#h02) | P2 | Debugger-host fixtures still use removed block and pattern syntax |
| [D01](documentation-and-specs.md#d01) | P2 | The AP13 documentation migration leaves invalid positive examples and corrections |
| [D02](documentation-and-specs.md#d02) | P2 | AP11 grouped declarations remain in current language and formatter documentation |
| [D03](documentation-and-specs.md#d03) | P2 | The formatter's complete control-flow example mutates a const binding |
| [D04](documentation-and-specs.md#d04) | P2 | The current formatter specification advertises unimplemented generic record types |
| [D05](documentation-and-specs.md#d05) | P3 | The documented long-import golden is not formatter output |
| [D06](documentation-and-specs.md#d06) | P2 | Two complete introductory examples omit required Console imports |
| [G01](documentation-and-specs.md#g01) | P2 | The formal keyword set omits reserved discard |
| [G02](documentation-and-specs.md#g02) | P2 | The formal task-call grammar excludes supported postfix targets |
| [P01](documentation-and-specs.md#p01) | P3 | Completed package result text describes features already removed by other completed packages |
| [T04](documentation-and-specs.md#t04) | P3 | Runtime string bounds hints still recommend a removed free-call spelling |
| [L01](documentation-and-specs.md#l01) | P3 | Cryptography index links outside the documentation tree |

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

## Verification

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

Known debugger limitations for named arguments, typed construction and reference
mutation already appear in [compiler follow-ups](../../compiler-panic-followups.md)
and are not counted as newly discovered defects. New production regression tests
were not added because this task is an audit; each finding states the missing
coverage required when repairing it. No language change, repair, commit or push
was performed.
