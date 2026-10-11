# Reviewed scope

This persistent record covers 22 AP packages inspected across two completed
reviews. Keep it when clearing the [review plan](README.md). Package
implementation status remains in the [central plan](../README.md).

Coverage records an inspection of the listed work packages and areas at the
recorded revisions. Later changes and untested combinations may need another
review. Review completion is not merge approval.

## Earlier completed review

The earlier review covered the following completed work packages, including
implementation, regression coverage and current documentation. Its 22
findings and follow-up repairs were closed by commit
`5b04c39697f2d57e6390648194ae294ddff614bb`.

| AP | Reviewed work packages | Inspected/exercised areas |
| --- | --- | --- |
| AP01 | AP01.1, AP01.2 | Current spelling, references, keyword/block consistency |
| AP02 | AP02.1–AP02.6 | Shared diagnostics, CLI JSON, project errors, real LSP transport |
| AP04 | AP04.1, AP04.2 | Consumed calls, discard type/capture proofs, unit interfaces |
| AP05 | AP05.1, AP05.2 | Alias resolution, shadowing, editor migration, import diagnostics |
| AP06 | AP06.1–AP06.3 | All 78 catalog entries, both argument forms, lowering/runtime, editor chains |
| AP07 | AP07.1–AP07.4 | Boolean grammar, precedence, short circuit, Bits and rejection coverage |
| AP09 | AP09.1, AP09.2 | Named/generic/variant calls, written evaluation order, callable restrictions |
| AP10 | AP10.1–AP10.3 | Typed fields/defaults, aliases, visibility, unit reuse, const/capture interaction |
| AP11 | AP11.1, AP11.2 | Individual declarations, whole-file type collection, finite graphs |
| AP13 | AP13.1–AP13.6 | Terminators, named closers, case arms, recovery, docs and embedded fixtures |
| AP14 | AP14.1, AP14.2 | Removed properties, member resolution and migration |
| AP16 | AP16.1–AP16.3 | Computed/static const, writable bindings, initialization and captures |
| AP17 | AP17.1–AP17.3 | Storage roots, aliases, reference modes, forwarding, capture lifetimes |
| AP20 | AP20.1–AP20.3 | Recursive patterns, coverage, constant identity, `is`/`while` bindings, runtime matching |

Cross-package checks covered independently compiled units, facade aliases,
fresh and reused sidecars, named argument evaluation, typed record defaults,
imported mutable routine values, same-root alias rejection, computed constants
and imported callable discard proofs.

The review also covered parser ownership and reuse, public Rust documentation,
handbook links, debugger scenarios for named calls and typed construction,
explicit-reference live assignments, unsaved editor corrections, callable
values versus invoked results, task-handle discard proofs and LSP lifecycle
and watched-file handling.

## Subsequent completed review

Fixed implementation review range:

- Baseline: `5b04c39697f2d57e6390648194ae294ddff614bb`.
- Target: `34397373799a5897dfcfc09e754d16fcad3c7386`.
- All six findings repaired, checked and reviewed again through
  `5ae28ba59f608b4051b9c247bfd70f43b0091d42`.

| AP | Reviewed work packages | Inspected/exercised areas |
| --- | --- | --- |
| AP03 | AP03.1, AP03.2 | Closed-enum coverage, guards, scalar exceptions and case expressions |
| AP15 | AP15.1, AP15.2 | Optional-handler fields, removed event owners and existing consumer coverage |
| AP18 | AP18.1, AP18.2 | Subrange bounds, conversions, membership, diagnostics and generic/record use |
| AP19 | AP19.1, AP19.2 | Distinct identity, conversions, comparisons, constraints and case labels |
| AP21 | AP21.1, AP21.2 | If/case expression typing, lowering, constants, pattern scope and editor binding |
| AP22 | AP22.1, AP22.2 | Allowed inference, constructor/conversion evidence and editor integration |
| AP23 | AP23.1, AP23.2 | Contracts, exits, captures, indirect/generic calls and unit diagnostics |
| AP24 | AP24.1–AP24.3 | Generic records/enums, joint inference, recursion, callable fields and symbols |

Repairs covered decision-expression payload typing, nested generic callback
inference, type declarations in conversion value positions, generic data type
parameter identities, lexical decision-pattern bindings and specialized
callable-field signatures. No independent standards finding was confirmed.
Repair evidence and detailed checks are retained in the six repair commits
between the target revision and the closure revision.

Verification covered the Rust workspace, targeted semantic and language-service
regressions, real CLI diagnostics, the standalone FPAS suite, source formatting
and the full VS Code extension-host suite. No benchmark or exhaustive
feature-combination claim is made. A separately observed postfix-field
navigation limitation in `Box(...).Value` was outside the six repaired findings.

## Scope for a later review

AP12, AP25, AP26 and AP27 were outside these completed implementation reviews;
their proposal status was not treated as missing implementation. AP08, AP28 and
AP29 have no work packages in the central plan.

Before starting another review, recheck the current central plan and select a
fixed revision range. Append the inspected AP work packages, coverage limits
and closure revision here when that review finishes. Preserve earlier entries
instead of replacing them with the newest scope.
