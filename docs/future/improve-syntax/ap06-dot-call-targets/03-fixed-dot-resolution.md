# AP06.3: Implement native type operations and remove duplicate call forms

Package: [AP06: Fixed dot-call targets](README.md)

Status: complete.

## Result

Dot calls select a declared record member or exactly one native operation by
static type and case-insensitive name. Free routines and callable values are
never searched by receiver; ordinary calls retain ordinary name resolution.

All 78 entries are import-free. Strings use Unicode-scalar `Slice`; `IsEmpty`
reuses length handling. `Join` belongs to arrays of string. The only factories
are `string.Chr` and `array.Fill`, with Fill inference from Value even at count
zero. Binding annotations remain required. Results can change type and continue
chains. Existing eager processing, ordering and value-returning semantics are
retained.

Fixed signatures allow positional or fully named explicit arguments. The
receiver is unnamed and evaluates once before arguments in written order.
`Format` remains positional and heterogeneous. Writable Push/Pop receivers use
AP17's shared storage, aliasing, lifetime, failure and task checks.

The catalog in `crates/fpas-sema/src/std_registry/native/` supplies checking,
lowering, completion, signature help and hover. API export and source-library
discovery exclude the five retired helper units. Native signatures are
[documented with their types](../../../pascal/language/types/README.md).

## Regression coverage

- Every catalog entry accepts its import-free positional/named signature.
- Automatic checks cover collisions, canonical names, comparable parameter
  roles and exactly one matching handbook signature per operation.
- Compiler and FPAS tests cover type-changing chains, factories, zero-count
  inference, Unicode, named argument traces, early failures/try, empty checks,
  native tasks, writable storage and copy-on-write.
- Sema and CLI tests reject free receiver lookup, retired imports/calls/
  references, invalid names/counts/types, overload selection and invalid receivers.
- Editor tests cover aliases, incomplete chains, signatures, hover,
  variadic Format, factory completion and absent retired-unit auto-imports.

No AP06-specific decision or implementation blocker remains.
