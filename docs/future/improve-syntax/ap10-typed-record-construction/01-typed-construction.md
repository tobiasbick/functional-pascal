# AP10.1: Typed construction

Package: [AP10: Typed record construction](README.md)

Status: complete.

## Result

Concrete record types and transparent type aliases support
`TypeName(Field := Value, ...)`, including imported types, qualified names,
import aliases, and construction inside generic routines. Aliases preserve
the original nominal identity, defaults, and declaring-unit visibility.

Call targets follow normal lexical and qualified lookup. A nearer value or
routine keeps its ordinary meaning; no hidden record type is used as a
fallback. Types and routines cannot share a case-insensitive name in one scope.

Fields are named only and matched case-insensitively. Unknown, duplicate,
missing required, and incorrectly typed fields are diagnosed. Methods,
properties, events, and `var` arguments are not constructor fields. Omitted
fields use declared defaults. Records with private stored fields construct
only inside their defining unit, even when those fields have defaults.

Supplied fields evaluate once in written order; omitted defaults follow in
field declaration order. Checked defaults retain expression identity and their
declaration environment, including nested constructors, named routine calls,
and callable defaults. Scalar defaults survive exported type aliases and
facades. Unit interfaces carry scalar constant defaults.

The formatter preserves constructor syntax and argument order. Editor field
completion inserts named labels and excludes supplied fields. Signature help
shows defaults and the named active field. Definition, hover, references, and
rename resolve constructor labels to the original field through type and
import aliases. Private records expose no constructor fields or signatures
outside the defining unit.

The contextual `record ... end` literal remains valid with its existing
field-declaration evaluation order. Positive consumers use typed construction
after AP10.2; AP10.3 owns removal of the literal form. Anonymous literal labels
remain outside editor field rename.

User-defined generic record types and constructor type-argument inference
belong to [AP24.2](../ap24-generic-data-structures/02-generic-records.md).

## Implementation locations

- `crates/fpas-sema/src/check/expr/record_construction/`: constructor and shared
  contextual field validation; call resolution and construction metadata.
- `crates/fpas-sema/src/check/decl/types/records/defaults.rs`: retained defaults;
  constant, discard, and task-bound classification use the checked fields.
- `crates/fpas-sema/src/check/decl/consts/records.rs`: known static record fields,
  including defaults evaluated in their declaration scope, support scalar
  projections and pattern coverage.
- `crates/fpas-sema/src/interface/export.rs`: scalar defaults in exported aliases.
- `crates/fpas-compiler/src/lowering/aggregates/record_construction.rs`: ordered
  field evaluation; `context/expressions.rs` retains default declaration scope.
- `crates/fpas-language-service/src/navigation/record_construction.rs` and
  `src/intellisense/record_construction.rs`: canonical field resolution and
  completion; symbol extraction and signature help expose constructors.

## Regression coverage

Sema, compiler, build, formatter, and language-service tests cover concrete,
empty, nested, aliased, imported, and generic-routine construction; lexical
shadowing and scope collisions; private fields inside and outside their unit;
required/defaulted fields and invalid field lists; written-order traces and
`try` exits; default binding and metadata; constant and callable capture
classification; interface reuse and facade aliases; field labels, signatures,
completion, and rename. The FPAS suite includes
`tests/stdlib/records/typed_construction_test.fpas`.

## Documentation

- [Records](../../../pascal/language/types/records.md).
- [Editor integration](../../../pascal/tools/editor-integration.md).
- `docs/specs/grammar.ebnf` (`typed_record_construction`).
