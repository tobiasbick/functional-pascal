# AP06: Fixed dot-call targets

Status: agreed direction (Q05, revised). The standard-operation catalog, import
rules, and name-conflict rules are open and must be decided before
implementation (AP06.1). Effort: medium. Completion is tracked in the
[central README](../README.md); the process is in
[development-process.md](../development-process.md).

This package was previously titled "Remove automatic receivers". The revised
decision keeps dot chaining for record methods and a fixed set of standard
operations, and removes only the automatic lookup of arbitrary free functions.

## Goal

A dot call `Value.Name(Arguments)` has a fixed, predictable target: a record
method of the value's record type, or one standard-library operation assigned
to the value's built-in type. An imported or local free function can never
reinterpret a dot call.

Current behavior: `Value.Name(Args)` may resolve to any visible free routine or
callable value whose first parameter accepts the receiver, with layered lookup
rules ([receiver calls](../../../pascal/language/functions/fluent-calls.md),
[postfix chaining](../../../pascal/language/functions/postfix-chaining.md)).

## Decisions (Q05, revised)

- **Record methods keep dot chaining.** Instance methods declared in a record
  (with `Self`) stay callable as `Value.Method(Arguments)`. A method returns
  `Self`'s record type or a new value; the chain continues on the static type
  of that result.
- **Standard operations on built-in types.** For built-in types such as
  `string`, arrays, and dictionaries, matching standard-library operations
  such as `Trim`, `Map`, and `Filter` are assigned to dot notation in a fixed,
  unambiguous catalog. Each catalog entry maps one receiver type and one name
  to exactly one standard-library routine. A result may change type, and the
  chain continues on the new type.
- **No automatic free-function lookup.** Dot notation no longer searches
  visible free functions, procedures, or callable values by their first
  parameter. User-defined free functions and all other routines are called
  with ordinary call syntax, nested or through intermediate bindings.
- **Receiver passing stays.** The left value is passed as `Self` to a method,
  or as the first input argument of a catalog operation. The receiver is
  evaluated once, before the written arguments, which keep their order.
- **Unchanged from Q05:** no pipe operator. Dot notation remains available for
  actual record members (fields, callable fields, methods).

Draft, using agreed AP16 bindings:

```pascal
const Words: array of string := Input.Trim().Split(' ');   // catalog operations
const Sizes: array of integer := Words.Map(Std.Str.Length);   // array to array of integer
const Next: Point := Origin.Offset(1.0, 2.0).Normalize();  // record methods
const Clean: string := Normalize(Input);                    // own free function
const Bad: string := Input.Normalize();   // error: 'Normalize' is not a method
                                          // or standard operation of string
```

## Open decisions

These must be decided explicitly with the user in AP06.1, before any
implementation work package starts:

1. **Catalog contents.** The exact list of standard operations per receiver
   type for `string`, arrays, and dictionaries. Whether `Option`, `Result`,
   scalar types, channels, or task handles get catalog operations. Whether
   values of a generic type parameter have any.
2. **Import rules.** Whether a catalog dot call requires the owning `Std` unit
   to be imported (plainly or with an AP05 alias), or whether catalog
   operations are always available on their receiver type.
3. **Name conflicts.** How a catalog name relates to a local or imported free
   function with the same name; operations with the same name on different
   receiver types (for example `Length` on `string` and arrays); overloaded
   standard routines; a record field holding a callable value with the same
   name as a method.
4. **Mutating operations.** Whether caller-mutating operations such as `Push`
   and `Pop` belong to the catalog, and how AP17's call-site `var` marking
   applies to a dot receiver.

## Dependencies

- AP05 (the import model that the import rules build on).

AP12 and AP14 depend on this package.

## Order

AP06.1 records the open decisions. AP06.2 migrates calls that will lose their
dot form while the current lookup still accepts the ordinary form. AP06.3
switches resolution to methods and the catalog.

## Work packages

- [ ] [AP06.1: Catalog, import, and conflict decisions](01-catalog-and-rules-decision.md)
- [ ] [AP06.2: Migrate non-catalog receiver calls](02-migrate-non-catalog-calls.md)
- [ ] [AP06.3: Resolve dot calls through methods and the catalog](03-fixed-dot-resolution.md)

## Acceptance

An imported or local free function cannot reinterpret a dot call. Record-method
calls and catalog operations retain their meaning and allow type-changing
chains. Every former free-function receiver call is migrated to an ordinary
call with equivalent behavior.

## Reference

The reference branch `codex/syntax-changes` removed record methods and all
receiver calls. That direction is superseded by this decision and is not
adopted. Its owner map remains useful: sema `check/expr/calls/fluent.rs`,
`calls/methods.rs`, `check/expr/bound_method.rs`, and compiler
`lowering/calls/fluent.rs`.
