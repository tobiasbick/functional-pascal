# Crate review follow-ups (2026-08)

Verified implementation intake from a defect-first review of all 21 Rust workspace crates
(approximately 182,000 lines of Rust, including tests). The intake was checked against the
current checkout on 2026-08-19.

Start with [`how-to-implement.md`](how-to-implement.md), then take exactly one **open** task.
[`findings.md`](findings.md) is the evidence index, not an implementation checklist.

## Status meanings

- **open** — current code contradicts repository documentation, a protocol contract, or a
  state-safety invariant. The task is ready to implement.
- **decision required** — the desired behavior is absent from, or conflicts with, the current
  user-facing specification. Do not implement it until the user records a choice in the task.
- **coverage** — the implementation already enforces the rule; only regression coverage is
  missing.

Tasks 22, 29, 30, and 31 each own one concern. Tasks 32–37 contain the independent runtime, LSP,
formatter, coverage, transport, and console follow-ups.

## Recommended implementation order

Complete one row per implementation session unless the user explicitly groups tasks. Update the
task's progress record as described in [`how-to-implement.md`](how-to-implement.md).

| Order | Task | Difficulty | Why this slot |
|---:|---|---|---|
| 1 | [01 lexer exponent](tasks/01-lexer-exponent.md) | easy | Isolated lexer rule and existing numeric tests |
| 2 | [02 Slice/Substring/Delete overflow](tasks/02-std-range-overflow.md) | easy | Local checked range arithmetic |
| 3 | [03 FromChar / Pad / IntToHex limits](tasks/03-std-fromchar-pad.md) | easy | Shared allocation-limit helper |
| 4 | [04 Window / GotoXY errors](tasks/04-std-console-coords.md) | easy | Existing documentation already requires errors |
| 5 | [05 `uses` comments](tasks/05-fmt-uses-comments.md) | easy | Formatter comment emission only |
| 6 | [06 JSONL fatal exit status](tasks/06-debug-jsonl-fatal-exit.md) | medium | Distinguish clean and protocol-error termination |
| 7 | [07 DAP pagination](tasks/07-dap-pagination.md) | easy | Direct DAP contract mismatch |
| 8 | [08 empty parser sections](tasks/08-parser-empty-sections.md) | easy | Grammar is one-or-more |
| 9 | [10 retain recovered static AST](tasks/10-parser-static-ast.md) | easy | Parser recovery only |
| 10 | [30 formatter control-character codes](tasks/30-fmt-char-codes.md) | easy | Literal emission and round-trip tests |
| 11 | [31 remove no-op debug report flag](tasks/31-debug-report-option.md) | easy | Help/parser consistency |
| 12 | [34 skip globbed symlinks](tasks/34-fmt-glob-symlinks.md) | easy/medium | Output-path safety |
| 13 | [36 bound debugger input lines](tasks/36-debug-transport-limits.md) | medium | Bounded readers for two transports |
| 14 | [37 synchronize console dimensions](tasks/37-console-screen-dimensions.md) | medium | Getter/state consistency |
| 15 | [09 expression recovery](tasks/09-parser-expr-recovery.md) | medium | Shared synchronization-token policy |
| 16 | [12 closure initializers](tasks/12-compiler-closure-init.md) | medium | Declaration-expression discovery |
| 17 | [11 enum backing values](tasks/11-compiler-enum-alias.md) | medium | Existing interface field must reach semantic lowering |
| 18 | [15 contextual record literals](tasks/15-sema-record-literals.md) | medium | Establish before any record-identity decision |
| 19 | [16 imported `Color.Red`](tasks/16-sema-imported-enum.md) | medium | Imported ambiguity bookkeeping |
| 20 | [17 case exhaustiveness](tasks/17-sema-exhaustiveness.md) | medium | Use resolved labels rather than spelling |
| 21 | [22 restore failed IndexSet](tasks/22-vm-indexset-restore.md) | medium | One VM failure-path invariant |
| 22 | [32 clean up failed Application.Run](tasks/32-vm-graph-run-cleanup.md) | medium | Independent hosted-graph failure path |
| 23 | [27 atomic DAP breakpoints](tasks/27-dap-breakpoints.md) | medium | DAP replacement semantics |
| 24 | [33 bound LSP project discovery](tasks/33-lsp-discovery-boundary.md) | medium | Windows path identity and search boundary |

## Leave for a stronger model

These are ready but cross subsystem boundaries or have concurrency/identity failure paths. Read the
whole task and stop rather than landing a partial workaround.

| Task | Why hard |
|---|---|
| [13 generic compatibility](tasks/13-sema-generic-compat.md) | Strict body checks must not break call-site unification |
| [19 Wait shutdown](tasks/19-vm-wait-shutdown.md) | Concurrency test must prove bounded completion without hanging |
| [20 helped-task failure](tasks/20-vm-help-fail.md) | Must share the pool failure path and preserve task attribution |
| [24 linker layout identity](tasks/24-linker-layouts.md) | Per-object debug type IDs cannot be compared numerically |
| [28 DAP source identity](tasks/28-dap-source-paths.md) | Original and portable source identities must survive reloads |
| [29 LSP sibling I/O](tasks/29-lsp-sibling-io.md) | Analysis failure must still produce a current diagnostics publication |

## Decision queue — do not implement yet

| Task | Missing decision |
|---|---|
| [14 named record identity](tasks/14-sema-named-records.md) | Records documentation does not define nominal versus structural compatibility |
| [18 public API using a private type](tasks/18-sema-export-private-type.md) | Visibility docs do not define whether this is rejected or exported opaquely |
| [21 Sleep/Yield in synchronous callbacks](tasks/21-vm-callback-sleep.md) | Current scheduling docs promise cooperative spawned-task Sleep but do not define nested callback behavior |
| [23 overlapping consumer/library source](tasks/23-project-origin.md) | Project docs do not define ownership when one physical source belongs to both graphs |
| [26 test timeout policy](tasks/26-cli-test-timeout.md) | Current docs start explicit timeouts after worker readiness and define no default |

## Coverage-only follow-up

- [35 reject non-library dependencies](tasks/35-project-dependency-kind-tests.md) — implementation
  exists; add path and workspace regression tests.

## Out of scope

- Language or user-facing semantic decisions without the user's explicit agreement.
- Package registries, CI workflows, `unsafe`, or unrelated refactors.
- Marking a decision-required item open merely because an implementation seems convenient.
