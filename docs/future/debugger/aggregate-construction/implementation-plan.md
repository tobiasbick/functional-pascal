# Implementation plan

Statuses: `PLANNED`, `IN_PROGRESS`, `BLOCKED`, `COMPLETE`.

All work packages are `COMPLETE`. The shared VM engine, JSONL, DAP, and VS Code
command are implemented. Remaining exclusions stay in
[`consciously-deferred.md`](consciously-deferred.md).

## Work ledger

| ID | Status | Depends on | Work | Exit gate |
|---|---|---|---|---|
| AGC-00 | COMPLETE | — | Establish scope, decisions, architecture, verification matrix, and resume checkpoint | All seven plan files exist and central roadmap links this package |
| AGC-01 | COMPLETE | AGC-00 | Introduce normalized enum/`Result`/`Option` variant descriptors and refactor qualified transition metadata lookup to reuse them | Existing transition behavior is unchanged; descriptor unit tests cover fieldless, single-field, multi-field, missing, and malformed metadata |
| AGC-02 | COMPLETE | AGC-01 | Add read-only target variant discovery in the VM debugger session | Discovery works for initialized descendants and uninitialized mutable roots; non-wrapper, unknown, immutable, expired, and running-state targets reject deterministically |
| AGC-03 | COMPLETE | AGC-01, AGC-02 | Add exact-field validation, ordered detached evaluation, complete value construction, and atomic commit | Every supported wrapper shape constructs correctly; every pre-commit failure preserves the original value and generation |
| AGC-04 | COMPLETE | AGC-03 | Stabilize session models, error kinds, limits, response summaries, and focused module boundaries | Public APIs have Rust docs, files remain thematic, and no display-string inference or duplicated constructor logic remains |
| AGC-05 | COMPLETE | AGC-04 | Expose `variant.describe` and `variant.construct` through JSONL | Machine-readable positive and rejection contracts pass, including parse offsets, canonical names, exact field sets, and unchanged-state failures |
| AGC-06 | COMPLETE | AGC-05 | Map `fpas/variantDescribe` and `fpas/variantConstruct` through DAP | DAP naming, response shape, stopped-state failures, and success-only negotiated invalidation match JSONL behavior |
| AGC-07 | COMPLETE | AGC-06 | Add and test the VS Code `Debug: Construct Variant` command | Interactive and programmatic command paths use discovery plus construct; Extension Host proves fieldless and multi-field continuation behavior |
| AGC-08 | COMPLETE | AGC-01 through AGC-07 | Reconcile current docs, run required gates, close `DBG-D01`, and preserve permanent exclusions as supported limits | Documentation describes only implemented behavior; required test/build/package gates pass; central backlog no longer contains DBG-D01 |

## Package details

### AGC-01 — Shared metadata model

- Define canonical wrapper and variant descriptors from executable debug
  metadata.
- Preserve variant and field declaration order.
- Carry debug type IDs and runtime enum layouts without exposing runtime values
  to protocol layers.
- Refactor `mutation/transition/suffix.rs` to consume descriptors.
- Run existing variant-transition and variant-replacement tests before advancing.

### AGC-02 — Discovery

- Resolve the declared type through the existing textual mutation-target path.
- Support declared metadata even when an allowed mutable root is uninitialized.
- Return only names and portable display types required by a client.
- Keep discovery read-only: no generation increment, handle invalidation, call,
  or live mutation.

### AGC-03 — Construction and commit

- Normalize request field names and reject unknown, missing, extra, or duplicate
  normalized names before evaluation.
- Parse all field expressions before evaluating any of them.
- Evaluate in declaration order with one shared call/value/depth/time budget.
- Construct fieldless, one-field, and multi-field enum payloads plus all
  `Result`/`Option` variants.
- Reuse existing type validation and atomic root commit.
- Prove rollback for parse, evaluation, effect-policy, limit, type, metadata,
  mutability, and expired-target failures.

### AGC-04 — Session boundary

- Keep new APIs in `session/variant.rs`.
- Add stable error kinds only where clients must branch mechanically; reuse
  existing mutation/evaluation kinds otherwise.
- Return the established value summary plus canonical variant name.
- Audit file sizes and split by concern before adding logic to files already
  near the project threshold.

### AGC-05 — JSONL

- Add strict argument decoding and command dispatch.
- Keep request and response keys snake_case.
- Include actionable `code`, `message`, and `help`; include parse offsets where
  available.
- Add protocol tests that continue execution after successful construction and
  demonstrate no mutation after every representative rejection.

### AGC-06 — DAP

- Keep custom request arguments camelCase where DAP uses an equivalent name.
- Translate to the JSONL core; do not call VM construction directly.
- Reuse mutation result translation and variables invalidation ordering.
- Test clients both with and without `supportsInvalidatedEvent`.

### AGC-07 — VS Code

- Add one focused command module and register it from the adapter.
- Use discovery output for Quick Pick and prompt order.
- Treat cancellation at any prompt as no operation.
- Surface adapter errors without losing their actionable hint.
- Verify packaged command contribution and real Extension Host forwarding.

### AGC-08 — Completion

- Update `docs/pascal/tools/debugger.md` only after behavior exists.
- Convert stable exclusions from future promises into current documented limits
  backed by rejection tests.
- Remove `DBG-D01` from `deferred.md`; do not add child backlog rows for rejected
  stale-handle, partial, or virtual-tree behavior.
- Run the commands listed in the verification matrix and record exact evidence.

## Stop conditions

Stop and request a new decision if implementation would require:

- a change to FPAS syntax or constructor semantics;
- synthesizing missing outer storage or suppressing a source initializer;
- constructing identity-bearing or debugger-unsafe field values;
- exposing partially initialized live values; or
- changing standard DAP `setVariable` stale-handle semantics.
