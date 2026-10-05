# AP25: Conservative purity

Status: retained; low priority; reassess after AP23 (Q18). Effort: very large.
Completion is tracked in the [central README](../README.md); the process is in
[development-process.md](../development-process.md).

An effect system is excluded from this plan; see the [central README](../README.md#excluded-and-deferred).

## Goal

The checker enforces an explicit, explainable boundary and cannot infer purity
merely from immutable variable names.

## Decisions (Q18)

- Retain this package at low priority. Reassess implementation after AP23
  contracts have been used in practice; their initial form does not require
  this checker.
- Support `pure function` only, not pure procedures. Pure functions have no
  `var` parameters and may call only other pure functions.
- Captures must be immutable. Local `var` bindings are allowed only when their
  mutation cannot be observed externally.
- Panics are allowed; purity does not imply termination or freedom from errors.
- Standard-library routines carry purity metadata for checked calls.
- The initial contract is conservative: no I/O, no observation of globally
  mutable state, and no externally observable mutation. Ordinary functions
  and procedures keep their side effects.

## Dependencies

- AP14 (no computed properties hiding calls).
- AP16 (binding keywords).
- AP17 (`var` parameters).

## Order

AP25.1 is the reassessment gate. AP25.2 delivers the checker for user code;
AP25.3 adds verified standard-library metadata so pure functions can call
standard routines.

## Work packages

- [ ] [AP25.1: Priority reassessment](01-priority-reassessment.md)
- [ ] [AP25.2: Pure function checker](02-pure-function-checker.md)
- [ ] [AP25.3: Standard-library purity metadata](03-std-purity-metadata.md)

## Acceptance

The checker enforces an explicit, explainable boundary and cannot infer purity
merely from immutable variable names.
