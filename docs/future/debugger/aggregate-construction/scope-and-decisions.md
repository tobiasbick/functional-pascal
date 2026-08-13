# Scope and decisions

## Baseline assumptions

- Complete replacement of mutable enum, `Result`, and `Option` targets already
  works through constructor expressions.
- Qualified textual mutation already constructs an inactive single-payload
  variant when the target names its exact payload field.
- The debugger already resolves mutable textual paths, evaluates expressions
  in a detached context, validates portable debug types, and commits one live
  root atomically.
- JSONL is the external and LLM-friendly protocol. DAP and VS Code map onto the
  same debugger session behavior.
- No FPAS language or language-specification change is authorized by this
  package.

## In scope

- Enum, `Result`, and `Option` targets with usable portable debug metadata.
- Mutable local, capture-cell, or global roots already accepted by textual
  mutation.
- Stored descendants below initialized records, arrays, dictionaries, and
  active payloads when the existing textual target resolver marks them
  writable.
- Uninitialized mutable local or global roots when a complete variant value can
  initialize the root in one commit.
- Fieldless, single-field, and multi-field variants.
- Read-only variant discovery and explicit complete construction through the
  VM session, JSONL, DAP, and VS Code.
- Deterministic diagnostics suitable for humans and machine clients.

## Out of scope

- New FPAS syntax, constructor semantics, default arguments, or hidden-field
  initialization rules.
- Creating outer storage for an uninitialized descendant, missing capture cell,
  or absent parameter (`DBG-D02`).
- Identity-bearing or otherwise debugger-unsafe field values (`DBG-D03`).
- Implicit switching caused by `setVariable` on an old or inactive child.
- Incremental construction that exposes a partially initialized value.
- Virtual inactive variants in the standard DAP Variables tree.
- Task control, attach, remote debugging, native debugging, reverse execution,
  and hot reload.

## Fixed decisions

### AGC-D01: Textual target ownership

Discovery and construction accept the same textual assignment target and
optional frame ID as `expression.set`. No new long-lived mutable child handle is
introduced.

### AGC-D02: Discovery precedes structured construction

The engine exposes a read-only description operation. It returns the exact
variant names, declared field names, and display type names required to build a
valid request. Clients are not expected to infer construction rules from
formatted variable values.

### AGC-D03: Complete field set

Construction requires exactly one expression for every declared field and no
unknown fields. A fieldless variant requires an empty field object. Missing,
extra, or ASCII-case-duplicate field names reject the request before evaluation.

### AGC-D04: Metadata order is evaluation order

Field expressions are evaluated once in declaration order, independent of JSON
object ordering. All selector and field evaluation shares one
`DebugEvaluationLimits` budget.

### AGC-D05: One atomic commit

Target resolution, variant resolution, expression parsing, detached evaluation,
type validation, and complete value construction finish before the live root is
written. Any failure leaves the target and debugger generation unchanged.

### AGC-D06: One engine, three surfaces

The VM session owns semantics. JSONL performs protocol parsing, DAP performs
naming translation, and VS Code performs prompting only. No surface may build a
variant value independently.

### AGC-D07: Naming

Enum variants use their fully qualified metadata name, for example
`Choice.Pair`. `Result` and `Option` use `Ok`, `Error`, `Some`, and `None`.
Variant and field matching is ASCII-case-insensitive; responses always return
canonical metadata spelling.

### AGC-D08: Stopped-state operation

Both discovery and construction require a stopped session. Construction also
requires the target to remain mutable and current at commit time.

### AGC-D09: No partial or implicit fallback

The operation never guesses a variant from a field name, never fills omitted
fields, and never converts an expired child reference into a root switch.

### AGC-D10: Existing storage boundary

An uninitialized mutable root may receive the complete constructed value.
Every descendant target still requires its outer storage to exist. This keeps
empty-storage construction in `DBG-D02`.

## Success criteria

- All variants discoverable from executable metadata can be represented without
  display-string parsing.
- Fieldless and multi-field variants can be constructed explicitly and
  atomically through JSONL, DAP, and VS Code.
- Existing complete constructor assignment and single-payload qualified
  transition tests remain green.
- Rejections identify the target, requested variant or field, and a concrete
  correction hint without exposing machine-specific data.
- No language page changes are required.
