# AP36.1: Argument conventions

Package: [AP36](README.md). Status: complete; decision work package.

## Scope and deliverable

The user selected the [routine-call contract](README.md#recorded-routine-call-decisions):
retain AP09's two complete call forms, reject mixtures, support named arguments
through visible static callable signatures, retain parameter names in callable
types, and preserve positional structural compatibility. Receiver/argument
evaluation remains in written order, and variadic calls remain entirely positional.

This decision covers free/imported routines, methods, bound methods, callable
fields/parameters, anonymous functions, fixed-signature native operations,
fixed-arity polymorphic builtins, and eligible `go` calls. Removing named routine
arguments or callable type parameter names is not part of the selected direction.

The user also selected the [construction and pattern contract](README.md#recorded-construction-and-pattern-decisions):
preserve positional-or-named user enum construction and AP10's named record
construction/defaults; support both complete styles in enum/Option/Result
patterns; require every pattern field exactly once; and use the single label
`Value` for named Some/Ok/Error construction and patterns. Styles are independent
at each nested field list. `None` and fieldless variants retain no parentheses.

All language decisions within AP36 are recorded. AP36.2 has delivered their
implementation, documentation, and regressions. This file retains the decision
record; the implementation result belongs to AP36.2.

## Prerequisites and affected areas

AP09/AP10/AP20 are the baseline. Inspect
`crates/fpas-sema/src/check/calls/named.rs`, callable type representation,
unit interfaces, and argument parsing. Existing `ParamTy.name`, type resolution,
and interface conversion retain names; AP36.2 removes the call-checking restriction
and reuses that metadata and existing argument-order lowering.
AP36.1 delivers planning records; compiler implementation belongs to AP36.2.
Do not reopen AP08's parameter separator decision or AP12's direct returned
callable syntax as incidental cleanup.

## Documentation and verification

AP09 remains complete. AP36.2 replaces only its callable-value restriction and
the fixed-arity polymorphic exceptions selected above, while preserving both
call styles, case-insensitive labels, required arguments, and written evaluation.
AP09.2's two complete enum construction forms and AP10's named record defaults
are preserved. AP36.2 replaces AP20's positional-only enum/Option/Result pattern
field rule and the positional-only Some/Ok/Error constructor restriction.
AP20 binding/comparison/coverage rules and AP35's nonbinding is semantics remain.
The language handbook and grammar describe these delivered rules.

The following matrix records the agreed call behavior. The source defines Double
with parameter `N` and an explicit `F: function(Value: integer): integer`.

| Case | Selected result |
|------|-----------------|
| `Double(3)` and `Double(N := 3)` | Both accepted. |
| `F(3)` and `F(Value := 3)` | Both accepted, using F's visible type. |
| `F(N := 3)` | Rejected; implementation names are not F's labels. |
| Assign Double to F despite `N` versus `Value` | Accepted under positional type/mode compatibility. |
| A call containing positional and named arguments | Rejected for every target. |
| A fully named call with reordered same-typed parameters | Labels determine slots; effects occur once in written order. |
| `Name := var Storage` versus positional `var Storage` | Both accepted for a var slot; existing reference checks apply. |
| Compatible callbacks with differently named parameters | Accepted; calls use the receiving callable signature. |
| Callable fields, bound methods, imported aliases, and eligible `go` | Same complete-call rule; Self stays implicit where applicable. |
| Fixed-arity polymorphic native calls | Support both complete forms with their declared labels. |
| Variadic calls or a named prefix plus a variadic tail | Entirely positional accepted; any named argument rejected. |
| Unknown, repeated, or missing supplied labels | Rejected with visible signature names. |

Implementation coverage must also confirm stable visible signatures under AP34
inference, specialization, decision expressions, and callable-valued aggregates;
ambiguous duplicate labels in written callable types cannot silently pick a slot.

The construction/pattern matrix is also selected, using a Rectangle variant with
Width and Height fields and appropriately typed Option/Result values:

| Case | Selected result |
|------|-----------------|
| `Shape.Rectangle(1.0, 2.0)` and `Shape.Rectangle(Width := 1.0, Height := 2.0)` | Both accepted. |
| Reordered named enum construction | Fields map by label; values evaluate once in written order. |
| `Point(X := 1, Y := 2)` and omission of fields with declared defaults | Existing named record construction accepted. |
| `Point(1, 2)` | Positional record construction remains rejected. |
| `Shape.Rectangle(const W, _)` and `Shape.Rectangle(Width := const W, Height := _)` | Equivalent complete patterns. |
| Reordered named pattern labels | Same declared field mapping, independent of binding names. |
| Missing, repeated, or unknown pattern fields | Rejected, including case-insensitive duplicate labels. |
| `Shape.Rectangle(const W, Height := _)` | Rejected mixture within one field list. |
| `Some(Value := Shape.Rectangle(const W, _))` | Valid nesting: each field list independently selects its style. |
| `Some(3)`, `Ok(3)`, `Error(E)` and their `Value := ...` forms | Both constructor styles accepted. |
| `Some(const N)`, `Ok(const N)`, `Error(const E)` and their `Value := ...` forms | Both case pattern styles accepted. |
| A payload name other than Value in a named Some/Ok/Error form | Rejected with the canonical Value label. |
| `None` and `Shape.Point` | Existing fieldless value spelling retained, without parentheses. |
| `Input is Some(Value := _)` | Accepted nonbinding test under AP35. |
| `Input is Some(Value := const N)` or any nested binding in is | Rejected under AP35. |

Decision verification covers both matrices, explicit preservation/replacement of
earlier contracts, and implementation ownership. No language decision remains
open in AP36's scope. Executable regressions, consumer updates, and
handbook/grammar verification are complete in AP36.2.
