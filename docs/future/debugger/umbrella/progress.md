# Umbrella progress

## Current checkpoint

- Umbrella state: between implementation packages
- Active primary package: none
- Last completed item: `UMB-10D`
- Next child: `UMB-20` / `U20-00` (checkpoint and baseline); plan ready, not
  started
- Checkpoint: completed `UMB-10C` is recoverable at `eed79928`; `UMB-10D`
  identity-boundary decisions are in the worktree
- Blocked child: `UMB-10B` requires `UMB-90`; `U10D-CELL` is reclassified to
  `UMB-70A`
- Branch: `codex/fpas-debugger`
- Implementation started: yes

The umbrella plan and completed `UMB-10A` implementation are committed at
`f1c991e2`. Completed `UMB-10C` is committed at `eed79928`. The current
worktree contains the completed `UMB-10D` decisions and focused rejection
tests. Inspect the current worktree before changing or staging anything.

## Package status

| ID | Status | Last evidence or next action |
|---|---|---|
| `UMB-00` | done | Current branch/worktree and focused baseline verified; CCRA is recoverable at `bed152a2` |
| `UMB-01` | done | Child contracts, dependencies, risks, and acceptance evidence frozen in this umbrella |
| `UMB-10` | blocked | `UMB-10A`, `UMB-10C`, and `UMB-10D` complete; remaining `UMB-10B` waits on `UMB-90` |
| `UMB-20` | pending | Execution plan ready in [`umb-20/`](umb-20/README.md); perform `U20-00` before activation |
| `UMB-30` | pending | Scheduler/waiter design review before code |
| `UMB-40` | pending | Quiescence protocol first |
| `UMB-50` | pending | Transport separation design after `UMB-40A` |
| `UMB-60` | pending | Local attach before remote; native is go/no-go |
| `UMB-70` | pending | Stable data identities before watchpoints; `U10D-CELL` depends on `UMB-70A` |
| `UMB-80` | pending | Recording format and nondeterminism inventory first |
| `UMB-90` | pending | Requires version/snapshot model from `UMB-80`; unblocks `UMB-10B` |
| `UMB-99` | pending | Final parity and plan removal |

Allowed statuses are `pending`, `active`, `blocked`, `rejected`, and `done`.
Only one primary package may be `active`.

## Known baseline evidence

The immediately preceding debugger review established:

- Rust formatting, diff checks, workspace build, focused bytecode/VM/debugger
  tests, FPAS fixture formatting, and VS Code extension-host tests passed.
- The full workspace test completed with one independently known
  language-service failure:
  `repository_references_find_notes_update_in_the_consuming_program` reports
  23 references while its assertion expects 22.
- Strict library Clippy for the changed VM and debugger libraries passed.
  Workspace/all-target Clippy is not a gate for this slice because existing
  test/example findings are outside its changed library scope.

Do not stage or commit the UMB-10D decisions, or push `eed79928`, without
matching user authorization.

## Child evidence

### `UMB-10A` — done in worktree

- Exact `SharedFunction` identity and existing cell handles are preserved.
- Copy is limited to a mutable local or parameter register in the selected
  owner task and selected frame.
- Foreign owner, global, descendant, capture-cell, stale-frame, spawn, and
  foreign invocation cases are rejected without a partial write.
- JSONL and DAP use the shared VM/session implementation; the VS Code extension
  uses standard Variables/Watch mutation and emits negotiated invalidation.
- No compiler, bytecode metadata, FPAS syntax, semantics, or language page was
  changed.

Current evidence:

```text
cargo fmt --check                                           PASS
fpas fmt --check (changed debugger fixture)                 PASS
cargo build --workspace --locked                            PASS
cargo clippy -p fpas-vm -p fpas-debug --lib -- -D warnings PASS
focused VM function/cell-capture tests                      PASS (18 + 17)
focused JSONL/DAP cell-capture tests                        PASS (2 + 2)
VS Code extension-host suite                                PASS
cargo test --workspace --locked --no-fail-fast              BASELINE
  only repository_context reference-count assertion fails: actual 23, expected 22
git diff --check                                            PASS
```

### `UMB-10C` — done at `eed79928`

- Portable compiler metadata maps each record method name to its exact
  canonical routine through bytecode, `.fpascu`, linker, and `.fpascp`
  artifacts.
- Assignment of `Receiver.Method` evaluates the receiver once, validates its
  exact record layout and visible function signature, and creates a function
  value that retains that receiver graph.
- Normal VM calls, hosted callbacks, controlled calls, and task spawning
  prepend the retained receiver while preserving user-visible arity.
- Receiver graphs containing task identities or otherwise unsupported mutable
  identity are rejected before commit. Missing, duplicate, and mismatched
  method metadata likewise leave the target and invalidation generation
  unchanged.
- JSONL and DAP exercise the shared VM/session implementation. VS Code uses
  the existing standard `setExpression`/Variables flow; no editor-specific
  behavior or FPAS syntax/semantics was added.

Current evidence:

```text
cargo fmt --all -- --check                                  PASS
fpas fmt --check (changed debugger fixture)                 PASS
git diff --check                                            PASS
cargo build --workspace --locked                            PASS
strict Clippy for changed libraries                         PASS
bytecode/compiler/unit/linker/program/bundle metadata tests PASS
focused VM bound-receiver and callback tests                PASS
focused JSONL/DAP assignment tests                          PASS
VS Code extension test suite                                PASS
cargo test --workspace --locked --no-fail-fast              BASELINE
  related bundle golden was updated and passes;
  only repository_context reference-count assertion fails: actual 23, expected 22
```

### `UMB-10D` — done in worktree

- `U10D-DYN` rejected: `DebugType::Dynamic` is generic type erasure, not a
  first-class callable endpoint. Current source and destination rejections
  stay.
- `U10D-CELL` blocked: capture-cell destinations have no owner-task or alias
  registry. Task-bound writes remain rejected until `UMB-70A`.
- `U10D-OPAQUE` rejected: `OpaqueHandle` is a raw integer; `SavedRegion` is a
  one-shot host map entry without typed identity on the value.
- `U10D-EDIT` rejected: synthetic `receiver` and `capture[i]` children stay
  non-assignable. Complete `setVariable`/`setExpression` replacement remains
  the write model. Code/`FunctionId`/signature stay on `UMB-90`.
- JSONL and DAP map the shared rejections. No new command. No language change.
  Compact VS Code function-value and capturing-routine host tests stop on
  source breakpoints instead of `stepIn` loops.

Current evidence:

```text
cargo fmt --all -- --check                                  PASS
fpas fmt --check (function_value_assignment fixture)        PASS
git diff --check                                            PASS
cargo clippy -p fpas-vm -p fpas-debug --lib -- -D warnings PASS
cargo build --workspace --locked                            PASS
focused VM function_value_assignment                        PASS (23)
focused JSONL function_value_assignment                     PASS (3)
focused DAP function_value_assignment                       PASS (4)
VS Code extension test suite                                PASS
cargo test --workspace --locked --no-fail-fast              BASELINE
  only repository_context reference-count assertion fails: actual 23, expected 22
```

Evidence log:

```text
2026-08-15 | UMB-00 | pending -> done | worktree | branch, scope, format, focused tests, build, and independent workspace baseline verified | freeze umbrella contracts
2026-08-15 | UMB-01 | pending -> done | worktree | child IDs, dependencies, acceptance matrix, risks, and consciously deferred scope recorded | start UMB-10A
2026-08-15 | UMB-10A | pending -> done | worktree | VM, JSONL, DAP, VS Code, format, build, Clippy, and workspace baseline evidence recorded | prepare UMB-10B contract tests
2026-08-15 | UMB-10B | pending -> blocked | f1c991e2 base | new bodies require a verified FunctionId in the immutable shared executable; source matching or a second interpreter is forbidden | resume after UMB-90 versioned live-image support
2026-08-15 | UMB-10C | pending -> active | worktree | bound methods can reuse existing method code if exact record-method metadata and receiver-aware invocation are added | implement portable metadata round trip first
2026-08-15 | UMB-10C | active -> done | eed79928 | portable metadata round trips, receiver-aware VM calls, JSONL/DAP assignment, editor tests, format, build, Clippy, and workspace baseline verified | prepare UMB-10D feasibility decisions
2026-08-15 | UMB-10D | pending -> pending | worktree | execution-ready plan splits Dynamic, capture-cell, opaque-resource, and callable-editing decisions with exact gates and evidence | refresh the U10D-00 baseline, then start U10D-01
2026-08-15 | UMB-10D | pending -> done | worktree | DYN rejected, CELL blocked by UMB-70A, OPAQUE rejected, EDIT rejected; focused rejection tests, current-doc limitations, format/build/Clippy/npm, and workspace BASELINE recorded | start UMB-20A
2026-08-15 | UMB-20 | pending -> pending | worktree | context-loss-safe contracts, work IDs, file layout, negative gates, adapter parity, and verification matrix recorded; UMB-10D detail plan deleted after review | perform U20-00 after checkpoint authorization
```

## Resume commands

Run from the repository root:

```powershell
git status --short --branch
git diff --stat
Get-Content docs/future/debugger/umbrella/progress.md
Get-Content docs/future/debugger/umbrella/implementation-plan.md
cargo fmt --check
cargo build --workspace --locked
```

The next implementation step is `U20-00` in
[`umb-20/implementation-plan.md`](umb-20/implementation-plan.md). `UMB-10D`
evidence remains in this file, focused tests, and current debugger docs. Do not
clean, reset, stage, commit, merge, or push without matching user authorization.

## Evidence log format

Append one short entry when a package changes state:

```text
YYYY-MM-DD | UMB-ID | old -> new | commit or worktree | commands/results | next exact step
```

Do not record usernames, hostnames, absolute user paths, process IDs, or other
machine-identifying data.
