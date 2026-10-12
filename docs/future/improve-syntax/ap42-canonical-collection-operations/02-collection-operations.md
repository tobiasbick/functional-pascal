# AP42.2: Collection operations

Package: [AP42](README.md). Status: blocked by AP42.1.

## Scope and affected areas

Implement selected catalog removals/result changes and static sorting checks.
Inspect native signatures under `crates/fpas-sema/src/std_registry/builtins/`,
runtime collection operations, generated API declarations, contract validation,
diagnostics, and language-service native completion. Reuse existing membership
and ordering logic according to the chosen contract.

## Migration and documentation

Use resolved operations and temporary bindings where needed to retain
evaluation order. Replace -1 checks with explicit Option handling if approved.
Update native array/string/dictionary references, operators, contracts,
compact reference, API catalogs and consumers. Adjust
[shared catalog checks](../shared-constraints.md) if the approved entry set
changes; do not leave the old fixed count as a requirement.

## Verification

Test preserved argument effects, Unicode indexing, validation of repetition
inputs, present/absent searches, unsupported sorting at check time for both
empty and populated arrays, generics, and imported code. Contracts must retain
their approved membership capability without reopening AP25. Run required
compiler, Std, FPAS, formatter, and documentation checks.
