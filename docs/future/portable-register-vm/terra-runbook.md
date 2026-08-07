# Terra implementation runbook

This file is the operational handoff for a GPT-5.6 Terra implementation agent. The architecture is
already selected. Follow it; do not substitute a different VM, add Cranelift, or change FPAS.

## Start-of-turn checklist

At the beginning of every implementation turn:

1. Confirm the active branch is `codex/portable-register-vm-plan` or a user-approved implementation
   branch derived from it.
2. Read repository `AGENTS.md` completely.
3. Read every document in this directory in the order listed by `README.md`.
4. Read `.agents/skills/fpas-bench/SKILL.md`, `.agents/skills/fpas-change-checklist/SKILL.md`, and
   `.agents/skills/rust-best-practices/SKILL.md` plus its chapters relevant to that phase.
5. Run `git status --short --branch`. Preserve unrelated user work.
6. Locate current files with `rg --files` and symbols with `rg`; do not trust stale paths blindly.
7. Read the current phase, its dependencies, its tests, and neighboring modules completely enough to
   understand ownership.
8. Announce assumptions, success criteria, and exact file layout before editing.

Do not commit or push unless the user explicitly requests it. Branch creation is not commit authority.

## Non-negotiable scope guard

Stop and ask the user before any change to:

- FPAS tokens, grammar, syntax, accepted/rejected programs, evaluation order, type rules, visibility,
  numeric semantics, concurrency semantics, or standard-library API;
- normative language pages under `docs/pascal/language/` except correcting implementation-independent
  links after an approved change;
- CLI surface beyond updating artifact version diagnostics already required here;
- the meaning of `.fpasprj`, `.fpasworkspace`, `.fpascu`, or `.fpascp` as a user workflow.

Internal format incompatibility is approved. User-facing language incompatibility is not.

## Fixed technical choices

Do not ask the user to choose among these during implementation:

- safe Rust register interpreter first;
- packed `Instruction(u64)` with explicit codecs;
- fixed-width numeric IDs and deterministic tables;
- typed control-flow IR in `fpas-ir`;
- deterministic linear-scan register allocation;
- per-function register windows;
- numeric function/global/field/variant dispatch;
- sparse source maps resolved on errors;
- explicit little-endian sectioned artifact encoding;
- Cranelift deferred;
- old stack artifacts rejected and rebuilt, not migrated;
- one final compiler and VM path.

If a fixed choice conflicts with concrete current semantics, preserve semantics and report the conflict
with file/test evidence before changing architecture.

## Work-unit size

One work unit should be reviewable and independently verified. Good units:

- ID newtypes plus their conversion tests;
- one IR concern plus validation;
- instruction packing plus exhaustive form tests;
- one verifier concern;
- one lowering family plus differential tests;
- one runtime handler family plus existing regressions;
- one artifact section codec plus malformed inputs.

Bad units:

- "rewrite compiler and VM" in one edit;
- code, dependency upgrades, formatting cleanups, and unrelated refactors together;
- broad search-and-replace of `Chunk` without understanding each consumer;
- adding a compatibility abstraction intended to be deleted later without a named deletion phase.

## File discipline

- Keep one concern per file and prefer thematic subdirectories.
- When an existing file exceeds roughly 400 lines, plan a concern-based split before adding substantial
  logic.
- Do not create `utils.rs`, `helpers.rs`, `common.rs`, `legacy.rs`, `old.rs`, or `new.rs` as dumping
  grounds.
- Public modules, types, and functions receive concise `///` documentation.
- Comments explain invariants, portability, or measured performance reasons, not obvious mechanics.
- Avoid `TODO` comments. Put incomplete work in the phase ledger and finish it before phase completion.
- Remove dead imports, declarations, modules, dependencies, and test adapters exposed by the work.

## Rust discipline

- Use Rust edition 2024 conventions and existing workspace lints.
- Prefer borrowing and slices in hot paths. Clone only at a clear ownership/value-semantics boundary.
- Keep small ID/instruction types `Copy`; do not make `Value` `Copy`.
- Use `Result` and structured crate errors. No production `unwrap`/`expect` or panic for input/artifact
  failures.
- Use `try_from`, checked arithmetic, and explicit error fields for every narrowing/bounds calculation.
- Use static dispatch in interpreter hot paths. Do not introduce trait objects for hypothetical
  backends.
- Do not add unsafe code in this plan.
- Do not add dependencies until the current standard library/existing crates cannot solve the concern.
  Cranelift and JIT support are explicitly forbidden in this plan.
- Use `#[expect(clippy::...)]` narrowly with a reason only after understanding a genuine lint exception.

## Test-first sequence for each opcode family

1. Define IR semantic operation and validation.
2. Define checked bytecode constructor/accessors.
3. Add valid packing round-trip test.
4. Add malformed operand/verifier tests.
5. Add compiler lowering tests from real FPAS syntax.
6. Add direct interpreter fixture tests.
7. Add differential behavior tests against the old path while it exists.
8. Run existing regression families covering that operation.
9. Only then mark the operation mapped in `traceability.md`.

Never implement an opcode only in compiler or VM. The slice includes IR, bytecode, verifier, lowering,
runtime, diagnostics, artifact codec coverage, and tests.

## Performance sequence

For any optimization beyond the structural register cutover:

1. identify a current release profile hotspot;
2. save a same-machine baseline with `cargo bench-fpas`;
3. make one narrow change;
4. release rebuild;
5. compare at least three times;
6. inspect every selected row;
7. keep only repeatable material gains;
8. run the full suite before recording;
9. record only the settled result.

Do not optimize from intuition, debug timing, instruction count alone, or historical results from a
different checkout/machine.

## Required phase report

After each phase, report:

```text
Phase: Pn — name
Outcome: complete | blocked | in progress
Files: created / moved / modified / removed
Behavior: unchanged, or exact approved observable change
Docs: paths updated or unchanged with reason
Tests: added/updated plus exact commands and outcomes
Bench: baseline/compare deltas, or not applicable because production path is unchanged
Portability: evidence collected and remaining unverified hosts
Cleanup: old/dead code removed; temporary path still present only until phase Px
Next: one concrete phase
```

Never report "all tests pass" without exact commands. Never report a speedup without before/after
numbers and workload.

## Blocking conditions

Stop and present evidence when:

- preserving current behavior appears to require a syntax or semantic decision;
- a current test and current user-facing specification disagree;
- deterministic numeric layout cannot be derived from semantic metadata;
- a platform failure comes from an external crate and no in-scope portable alternative is evident;
- mandatory performance gates fail after profiling and at least one evidence-based attempt;
- unrelated user changes overlap the same lines and cannot be preserved safely;
- a required native host/device is unavailable for a claim the user asked to confirm.

Do not mark blocked merely because the rewrite is large. Continue within the current phase while useful
independent work remains.

## Cutover checklist

Before switching production CLI/build/test paths:

- [ ] every current stack opcode has a register successor or a documented removal reason;
- [ ] compiler, verifier, and VM exhaustive opcode inventories agree;
- [ ] all language/compiler/VM regression families pass on the new path;
- [ ] all intrinsics and hosted callbacks use the register ABI;
- [ ] task state uses register frames and passes stress tests;
- [ ] unit objects/linker produce the verified register executable;
- [ ] new `.fpascp` decoder is bounded and mutation-tested;
- [ ] old artifacts fail with actionable rebuild diagnostics;
- [ ] source-less run and bundles pass;
- [ ] full workspace and FPAS suites pass.

Immediately after cutover, delete the old path. Do not start performance polishing while two production
architectures remain.

## Final completion checklist

- [ ] `cargo fmt`
- [ ] `cargo build`
- [ ] `cargo test --workspace`
- [ ] `cargo clippy --all-targets --all-features --locked -- -D warnings`
- [ ] `fpas fmt --check examples/ tests/ apps/`
- [ ] `fpas test tests/`
- [ ] release full benchmark baseline comparison repeated and summarized
- [ ] `cargo bench-fpas record "after portable register VM"`
- [ ] deterministic artifact digest test passes
- [ ] available native platform/cross-artifact matrix executed
- [ ] old stack implementation and temporary adapters removed
- [ ] `docs/pascal/` and Rust doc links reconciled
- [ ] docs and tests classified per the change checklist
- [ ] `git diff --check`
- [ ] no host-identifying metadata in repository diff
- [ ] this completed future-plan directory removed after its durable information moves to code/current
      docs/tests

Completion means the outcome is genuinely achieved, not merely that the token/time budget ended.
