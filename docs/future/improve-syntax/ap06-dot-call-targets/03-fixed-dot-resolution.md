# AP06.3: Resolve dot calls through methods and the catalog

Package: [AP06: Fixed dot-call targets](README.md)

## Scope

Replace the automatic receiver lookup with fixed resolution: record members
first, then the catalog entry for the receiver's built-in type. Remove the
selection of free routines and callable values by receiver.

## Prerequisites

- AP06.2 (no non-catalog receiver calls remain).
- AP05.1, if the agreed import rule refers to aliased imports.

## Implementation

- Sema: for a record receiver, resolve declared members only. For a built-in
  receiver, resolve the catalog entry for its static type and name. Otherwise
  report an error.
- Represent the catalog once, next to the standard-library registry, and use
  the same data for checking, lowering, completion, and documentation tests.
- Remove the free-routine and callable-value receiver path and its
  first-parameter filtering.
- Pass the receiver as `Self` or as the first argument; evaluate it once,
  before the written arguments.
- Diagnostics: for a rejected dot call, name the receiver type and show the
  ordinary call form, for example `Normalize(Input)`; when a catalog entry
  with a similar name exists, mention it.
- Language service: completion after `.` lists fields, methods, and catalog
  operations only; signature help covers catalog operations.
- Apply the agreed rule for mutating operations; if they need AP17's `var`
  marking, leave their dot form unchanged here and record the follow-up in
  AP17.3.

## Affected areas

- `crates/fpas-sema/src/check/expr/calls/fluent.rs`, `calls/methods.rs`,
  `std_registry/`.
- `crates/fpas-compiler/src/lowering/calls/fluent.rs`.
- `crates/fpas-language-service/src/intellisense/` (completion, signature help).
- `crates/fpas-diagnostics/src/codes.rs`.

## Migration

None beyond AP06.2. Negative tests keep removed forms only as rejection cases.

## Documentation

- Rewrite `docs/pascal/language/functions/fluent-calls.md` as the dot-call page
  (methods and catalog), update `postfix-chaining.md`, `record-methods.md`,
  and mark dot-available operations on the `Std` pages for strings, arrays,
  and dictionaries.
- `docs/specs/grammar.ebnf` if the postfix description changes.
- Diagnostics reference entry.

## Verification

- Record-method chains returning `Self` and new values.
- Catalog chains on `string`, arrays, and dictionaries, including
  type-changing steps (for example array to integer).
- Rejection of free-function and callable-value receiver calls, with hint.
- A local or imported function named like a catalog operation behaves as the
  agreed conflict rules state; it never selects a different dot-call target.
- Receiver-before-argument evaluation order; `go` with a dot call.
- Completion and signature-help tests.
