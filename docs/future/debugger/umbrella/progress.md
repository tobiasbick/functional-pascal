# Umbrella progress

## Current checkpoint

- Umbrella state: implementing `UMB-90`
- Active primary package: `UMB-90`
- Last completed item: `U80-50` after `U80-41`
- Next child: `U90-01` after `U90-00`
- Checkpoint: recoverable bounded recording capture without replay; `umb-80/`
  removed; `UMB-90` package is active
- Blocked child: `UMB-10B` requires `UMB-90`; `U10D-CELL` remains rejected
  (no capture-cell alias registry after `UMB-70A`)
- Branch: `codex/fpas-debugger`
- Implementation started: yes

The umbrella plan and completed `UMB-10A` implementation are committed at
`f1c991e2`. Completed `UMB-10C` is committed at `eed79928`; completed
`UMB-10D` is committed and available remotely at `87fd3b98`. Inspect the
current worktree before changing or staging anything.

## Package status

| ID | Status | Last evidence or next action |
|---|---|---|
| `UMB-00` | done | Current branch/worktree and focused baseline verified; CCRA is recoverable at `bed152a2` |
| `UMB-01` | done | Child contracts, dependencies, risks, and acceptance evidence frozen in this umbrella |
| `UMB-10` | blocked | `UMB-10A`, `UMB-10C`, and `UMB-10D` complete; remaining `UMB-10B` waits on `UMB-90` |
| `UMB-20` | done | Function breakpoints, exact runtime filters, policy ordering, adapters, docs, and full verification at `1198b1c6` |
| `UMB-30` | done | Entry completion, recovery, retained-result replacement, frame restart, initializer suppression, and instruction-change rejection at `c2a264d0` |
| `UMB-40` | done | All-stop quiescence, per-task pause/resume, cancel with `F4016`, create/restart rejection, and non-stop/history rejection at `6422489e` |
| `UMB-50` | done | Protocol/debuggee separation, queued Read/ReadLn, stopped TUI/graph ownership, and in-call pause rejection at `aee4f6a2` |
| `UMB-60` | done | Local attach, remote, and native inspection rejected at `eb0fbe64`; sessions stay launch-owned |
| `UMB-70` | done | Global write/change data breakpoints, location describe, and breakpoint-hit assign at `26b47a1d`; `umb-70/` removed |
| `UMB-80` | done | Bounded capture, `F4024` while capturing, recording-off unchanged, and replay rejected at `aa2af962`; `umb-80/` removed |
| `UMB-90` | active | Live-image hot reload; see [umb-90/progress.md](umb-90/progress.md) |
| `UMB-99` | pending | Final parity and plan removal |

Allowed statuses are `pending`, `active`, `blocked`, `rejected`, and `done`.
Only one primary package may be `active`.

## Known baseline evidence

The `U20-00` baseline established:

- Rust formatting, diff checks, workspace build, focused bytecode/VM/debugger
  tests, FPAS fixture formatting, and VS Code extension-host tests passed.
- The stale independent language-service reference-count oracle was corrected
  after a Notes fixture gained one real reference; its focused test and the
  full workspace suite now pass.
- Strict library Clippy for the changed VM and debugger libraries passed.
  Workspace/all-target Clippy is not a gate for this slice because existing
  test/example findings are outside its changed library scope.

Do not stage, commit, or push current `UMB-20` work without matching user
authorization.

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
- `U10D-CELL` rejected for this identity subset: capture cells have no alias
  registry (`unregistered_alias`). Task-bound writes stay rejected.
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
2026-08-16 | U20-00 | pending -> done | 87fd3b98 plus worktree | clean remote checkpoint; format, diff, Clippy, build, focused VM/JSONL/DAP, VS Code, and full workspace tests pass after correcting the stale Notes reference-count oracle | activate U20-01
2026-08-16 | U20-01 | pending -> active | worktree | matching, protocol, failure, policy, file-size, and limit ownership inventory started | add negative contracts before U20-10
2026-08-16 | UMB-20 | active -> active | worktree | function breakpoints, runtime filters, non-mutating policies, docs, focused tests, workspace suite, and VS Code host pass | checkpoint U20-50 before starting UMB-30
2026-08-16 | UMB-20 | active -> done | 1198b1c6 | recoverable checkpoint includes shared VM behavior, JSONL/DAP/VS Code parity, current docs, and green workspace verification | remove completed detail package and activate UMB-30
2026-08-16 | UMB-30 | pending -> active | 1198b1c6 base | context-loss-safe lifecycle package created from current runtime, scheduler, forced-return, protocol, and historical boundary evidence | execute U30-00
2026-08-16 | U30-00 | active -> done | 1198b1c6 plus docs | format, locked workspace build, 29 VM, 4 JSONL, and 3 DAP forced-return tests pass | execute U30-01
2026-08-16 | U30-10/11 | pending -> done | worktree | root/task entry completion, retained results, task events, JSONL/DAP/VS Code parity, docs, Clippy, build, and full workspace suite pass | execute U30-20
2026-08-16 | U30-20/21 | pending -> done | 48daa5cd | exact runtime-error recovery passes VM, scheduler, JSONL, DAP, VS Code, docs, and full workspace gates | execute U30-30
2026-08-16 | U30-30 | pending -> done | worktree | retained completed task results are typed, replaceable until consumption, available through JSONL/DAP/VS Code, and explicitly do not claim removed ordinary call frames | execute U30-40
2026-08-16 | U30-40/42 | active -> done/active | b7517403 | selected frame restart passes VM, JSONL, DAP, VS Code, documentation, formatting, and strict library Clippy gates | execute U30-41 and finish suppression adapters
2026-08-16 | U30-41/42 | active -> done | b7517403 plus worktree | portable exact initializer stores and live-frame suppression pass verifier, artifact, VM, forced-return, JSONL, DAP, VS Code, docs, Clippy, build, and workspace gates | wait before U30-50
2026-08-16 | U30-50 | pending -> done | c60e43ed plus worktree | instruction-change feasibility rejected; shared JSONL/DAP rejection, current docs, format, Clippy, workspace suite, and VS Code host pass | wait before U30-60
2026-08-16 | U30-60 | pending -> done | c60e43ed plus worktree | docs reconciled; `umb-30/` removed; cargo fmt --check, git diff --check, locked workspace build, strict library Clippy, cargo test --workspace --locked --no-fail-fast, and npm test (retry after an unrelated semantic-tools diagnostics timeout) pass | wait for checkpoint authorization before UMB-40
2026-08-16 | UMB-30 | active -> done | c2a264d0 | recoverable checkpoint includes instruction-change rejection, current docs, and removed `umb-30/` detail | activate UMB-40
2026-08-16 | UMB-40 | pending -> active | c2a264d0 base | context-loss-safe quiescence package created | execute U40-00
2026-08-16 | U40-00 | active -> done | c2a264d0 plus docs | format, locked workspace build, 13 VM behavior, 6 JSONL task, 2 DAP task, 10 DAP, and 1 scheduler test pass | freeze U40-01
2026-08-16 | U40-01 | pending -> done | c2a264d0 plus worktree | all-stop inspection cannot dispatch or admit tasks; frozen peers keep instruction windows | wait before U40-10
2026-08-16 | U40-10 | pending -> done | c2a264d0 plus worktree | stopped catalog no longer drains spawns; VM quiescence tests lock all-stop observation, shared globals, timers, and failed-handle expiry | wait before U40-11
2026-08-16 | U40-11 | pending -> done | c2a264d0 plus worktree | JSONL/DAP all-stop identity, session-wide continue, current docs, and VS Code host pass | wait before U40-20
2026-08-16 | U40-20 | pending -> done | c2a264d0 plus worktree | VM per-task pause/resume holds; paused peers are not dispatched; unknown/completed IDs reject atomically | wait before U40-21
2026-08-16 | U40-21 | pending -> done | c2a264d0 plus worktree | JSONL/DAP/VS Code per-task holds; `task_pause: true`; single-thread DAP remains false | wait before U40-30
2026-08-16 | U40-30 | pending -> done | c2a264d0 plus worktree | cancel stores F4016 without command-time dispatch; create/restart reject | map adapters
2026-08-16 | U40-31 | pending -> done | c2a264d0 plus worktree | JSONL/DAP/VS Code cancel; create/restart capabilities false; current docs | record U40-40
2026-08-16 | U40-40 | pending -> done | c2a264d0 plus worktree | non-stop, shortcuts, and unbounded history rejected | wait before U40-50
2026-08-16 | U40-50 | pending -> done | 6422489e plus worktree | docs reconciled; `umb-40/` removed; cargo fmt --check, git diff --check, locked workspace build, strict library Clippy, cargo test --workspace --locked --no-fail-fast, and npm test pass | wait for UMB-50
2026-08-16 | UMB-40 | active -> done | 6422489e | recoverable checkpoint includes all-stop task control, cancel, create/restart rejection, and removed `umb-40/` detail | activate UMB-50
2026-08-16 | UMB-50 | pending -> active | 6422489e base | context-loss-safe hosted-transport package created | execute U50-00
2026-08-16 | U50-00 | active -> done | 6422489e plus docs | format, locked workspace build, Clippy, workspace suite, and VS Code host pass | freeze U50-01
2026-08-16 | U50-01 | pending -> done | 6422489e plus worktree | raw protocol stdin rejected; structured output only | implement U50-10
2026-08-16 | U50-10 | pending -> done | 6422489e plus worktree | session debuggee channel connect/close; live input rejects without mutation | map adapters
2026-08-16 | U50-11 | pending -> done | 6422489e plus worktree | JSONL `io.input`, DAP `fpas/input`, VS Code structured output without a second console | wait before U50-20
2026-08-16 | U50-20 | pending -> done | 6422489e plus worktree | queued Read/ReadLn order, F4011, EOF, cancel, quota, disconnect | map adapters
2026-08-16 | U50-21 | pending -> done | 6422489e plus worktree | JSONL/DAP/VS Code live input; live_input true, live_terminal false | wait before U50-30
2026-08-16 | U50-30 | pending -> done | ac18d148 plus worktree | TUI/graph handlers wait until resume; debug KeyInput never polls OS | map adapters
2026-08-16 | U50-31 | pending -> done | ac18d148 plus worktree | JSONL/DAP event inject unsupported; no second editor event loop | wait before U50-40
2026-08-17 | U50-40 | pending -> done | aee4f6a2 plus worktree | in-call host interruption rejected; pause stays cooperative after the intrinsic; empty-queue ReadLn fails with F4011 | close U50-50
2026-08-17 | U50-50 | pending -> done | aee4f6a2 plus worktree | docs reconciled; `umb-50/` removed; cargo fmt --check, git diff --check, locked workspace build, strict library Clippy, cargo test --workspace --locked --no-fail-fast, and npm test pass | wait for UMB-60
2026-08-17 | UMB-50 | active -> done | aee4f6a2 | recoverable checkpoint includes debuggee channel, queued input, stopped event ownership, and in-call pause rejection | activate UMB-60
2026-08-17 | UMB-60 | pending -> active | aee4f6a2 base | context-loss-safe attach/remote package created from launch-owned JSONL/DAP and attach:false capabilities | execute U60-00
2026-08-17 | U60-00 | active -> done | aee4f6a2 plus docs | format, locked workspace build, Clippy, cargo test --workspace --locked --no-fail-fast, npm test, and git diff --check pass | freeze U60-01
2026-08-17 | U60-01 | pending -> done | fb91a7c7 plus worktree | JSONL/DAP attach reject without mutation; VS Code attach request rejected; current docs | wait before U60-10
2026-08-17 | U60-30 | pending -> done | fb91a7c7 plus worktree | native disassemble/memory/registers unsupported; second semantic engine forbidden | wait before U60-10
2026-08-17 | U60-10 | pending -> done | eb0fbe64 plus worktree | local attach rejected: fpas run has no listener; DebugSession constructs the VM; CLI flags stay unknown | close U60-20
2026-08-17 | U60-20 | pending -> done | eb0fbe64 plus worktree | remote sessions rejected with local attach; unauthenticated remote control forbidden | close U60-40
2026-08-17 | U60-40 | pending -> done | eb0fbe64 plus worktree | docs reconciled; `umb-60/` removed; cargo fmt --check, git diff --check, locked workspace build, strict library Clippy, cargo test --workspace --locked --no-fail-fast, and npm test pass | wait for UMB-70
2026-08-17 | UMB-60 | active -> done | eb0fbe64 | recoverable checkpoint includes attach/native rejection and launch-owned sessions | activate UMB-70
2026-08-17 | UMB-70 | pending -> active | eb0fbe64 base | context-loss-safe data-breakpoint package created from current source/function breakpoints and mutation identities | execute U70-00
2026-08-17 | U70-00 | active -> done | eb0fbe64 plus docs | format, locked workspace build, Clippy, cargo test --workspace --locked --no-fail-fast, npm test, and git diff --check pass | freeze U70-01
2026-08-17 | U70-01 | pending -> done | 7ab3e705 plus worktree | JSONL/DAP keep data breakpoints false; paired rejects do not resume; inspection IDs stay stop-scoped; format/build/Clippy/workspace tests/npm/diff check pass | wait for U70-10
2026-08-17 | U70-10 | pending -> done | 6f0b3b30 plus worktree | globals and live-frame identities; capture cells stay unregistered aliases; task-bound capture-cell destinations remain rejected | map U70-11
2026-08-17 | U70-11 | pending -> done | 6f0b3b30 plus worktree | JSONL location.describe and DAP fpas/locationDescribe; data breakpoints stay false; format/build/Clippy/workspace tests/npm/diff check pass | wait for U70-20
2026-08-17 | U70-20 | pending -> done | b65ecfc plus worktree | global write/change data breakpoints; read and frames unverified; atomic limit | map U70-21
2026-08-17 | U70-21 | pending -> done | b65ecfc plus worktree | JSONL/DAP data-breakpoint mapping; no extra VS Code watchpoint UX; format/build/Clippy/workspace tests/npm/diff check pass | wait for U70-30
2026-08-17 | U70-30 | pending -> done | da50b07b plus worktree | assign_data_location reuses commit_mutation; snapshots refresh once on success | map U70-31
2026-08-17 | U70-31 | pending -> done | 26b47a1d | JSONL/DAP assign mapping; function breakpoints reject assign; format/build/Clippy/workspace tests/npm/diff check pass | wait for U70-40
2026-08-17 | U70-40 | pending -> done | 26b47a1d plus worktree | docs reconciled; `umb-70/` removed; cargo fmt --check, git diff --check, locked workspace build, strict library Clippy, cargo test --workspace --locked --no-fail-fast, and npm test pass | wait for UMB-80
2026-08-17 | UMB-70 | active -> done | 26b47a1d | recoverable checkpoint includes global data breakpoints, location describe, breakpoint-hit assign, and removed `umb-70/` detail | activate UMB-80
2026-08-17 | UMB-80 | pending -> active | 26b47a1d base | context-loss-safe record/replay package created from launch-owned all-stop sessions with reverse_execution false | execute U80-00
2026-08-17 | U80-00 | active -> done | 26b47a1d plus docs | format, locked workspace build, Clippy, cargo test --workspace --locked --no-fail-fast, npm test, and git diff --check pass | freeze U80-01
2026-08-17 | U80-01 | pending -> done | worktree | named reverse/record rejects; record_replay false; paired JSONL/DAP tests; cargo fmt --check, locked workspace build, Clippy, cargo test --workspace --locked --no-fail-fast, npm test, and git diff --check pass | envelope U80-10
2026-08-17 | U80-10 | pending -> done | worktree | versioned envelope without host paths | map U80-11
2026-08-17 | U80-11 | pending -> done | worktree | JSONL/DAP recording.describe; cargo fmt --check, locked workspace build, Clippy, cargo test --workspace --locked --no-fail-fast, npm test, and git diff --check pass | capture U80-20
2026-08-17 | U80-20 | pending -> done | worktree | all-stop and queued-input capture; recording-off stays empty | map U80-21
2026-08-17 | U80-21 | pending -> done | worktree | JSONL/DAP record starts capture; replay stays rejected; cargo fmt --check, locked workspace build, Clippy, cargo test --workspace --locked --no-fail-fast, npm test, and git diff --check pass | bounds U80-30
2026-08-17 | U80-30 | pending -> done | worktree | advertised 4096 event ceiling; truncated; no disk or snapshot store | map U80-31
2026-08-17 | U80-31 | pending -> done | worktree | JSONL/DAP bound and truncated mapping; cargo fmt --check, locked workspace build, Clippy, cargo test --workspace --locked --no-fail-fast, npm test, and git diff --check pass | reject U80-40
2026-08-18 | U80-40 | pending -> done | worktree | capturing F4024 reject-before-dispatch; recording-off Random still terminates | map U80-41
2026-08-18 | U80-41 | pending -> done | worktree | JSONL/DAP F4024 and replayable false; cargo fmt --check, locked workspace build, Clippy, cargo test --workspace --locked --no-fail-fast, npm test, and git diff --check pass | close U80-50
2026-08-18 | U80-50 | pending -> done | aa2af962 plus worktree | docs reconciled; `umb-80/` removed; cargo fmt --check, git diff --check, locked workspace build, strict library Clippy, cargo test --workspace --locked --no-fail-fast, and npm test pass | wait for UMB-90
2026-08-18 | UMB-80 | active -> done | aa2af962 | recoverable checkpoint includes bounded capture, capturing F4024, recording-off unchanged, rejected replay, and removed `umb-80/` detail | activate UMB-90
2026-08-18 | UMB-90 | pending -> active | aa2af962 base | context-loss-safe hot-reload package created from launch-owned all-stop sessions with an immutable shared executable | execute U90-00
2026-08-18 | U90-00 | active -> done | aa2af962 plus docs | format, locked workspace build, Clippy, cargo test --workspace --locked --no-fail-fast, npm test, and git diff --check pass | freeze U90-01
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

The next pending implementation step is `U90-01` in
[umb-90/progress.md](umb-90/progress.md). `UMB-80` evidence remains in this
file, focused tests, and current debugger docs. Do not clean, reset, stage,
commit, merge, or push without matching user authorization.

## Evidence log format

Append one short entry when a package changes state:

```text
YYYY-MM-DD | UMB-ID | old -> new | commit or worktree | commands/results | next exact step
```

Do not record usernames, hostnames, absolute user paths, process IDs, or other
machine-identifying data.
