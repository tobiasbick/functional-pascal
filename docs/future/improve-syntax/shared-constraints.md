# Shared constraints and regression checks

See the [steering document](README.md) for package completion and language
gates, and [development-process.md](development-process.md) for the required
checks per work package. This document supplies cross-package checks, not a
second implementation backlog.

The regression checks below are applied when a package is marked complete, in
addition to the verification list of each of its work packages.

## Per-package regression checks

| Area | Test case | Required result |
|------|-------|-----------------|
| Arguments | Two same-typed path parameters | Visible role mapping; invalid names rejected |
| Mutation | Local parameter copy versus caller variable | Only explicitly authorized caller mutation escapes |
| Enums | Add a closed-enum variant | Missing cases identified explicitly |
| Imports | Introduce a colliding helper | Qualified calls retain their target |
| Dot calls | Declare a free function named like a standard operation | The dot call keeps its catalog or member target |
| Native type operations | Use the complete built-in catalog without imports | Operations remain available; the five former helper units provide no parallel public API |
| Catalog coverage | Check all 78 native entries against the catalog | Names, receiver shapes, signatures, constraints, errors, and evaluation rules match |
| Booleans | `X > 0 and Y > 0`; `A and B or C` | First parses as two comparisons; second asks for parentheses |
| Discard | Unused `Result` from a cleanup call | Rejected without `discard`; accepted with it |
| Blocks | Nested conditionals and endings | Unambiguous branch ownership and useful recovery |
| Domain types | Swap UserId and OrderId | Type mismatch rejected |
| Subranges | Convert dynamic input to Percent | Out-of-range input cannot bypass checks |
| Contracts | Clamp with inverted bounds | Precondition violation detected |
| Scopes | Scoped child task and detached task | Ownership and completion remain explicit |
| Properties | Sort idempotence and encoding round-trip | Existing test mechanisms express the property |

Do not invent native test syntax for these tests.

## Cross-package rules and remaining details

| Concern | Required distinction | Open details |
|---------|----------------------|--------------|
| Bindings | Immutable name versus mutable/shared value | Binding, aggregate-write, and capture rules are delivered in [AP16](ap16-immutable-and-mutable-bindings/README.md); shared handles retain their resource semantics |
| Constants | Computed initialization versus compile-time constant | Case-label rules are delivered in [AP16.1](ap16-immutable-and-mutable-bindings/01-computed-const-bindings.md); subrange bounds follow in [AP18.1](ap18-subrange-types/01-subrange-declarations.md) |
| Parameters | Read-only parameter versus `var` caller mutation | See [AP17](ap17-visible-caller-mutation/README.md) |
| Patterns | Explicit binding versus literal/constant comparison | Scalar guard bindings, shadowing, constructor lookup, and recursive coverage are delivered in [AP20](ap20-nested-patterns-and-explicit-bindings/README.md#decisions); closed-enum `else` rejection remains in [AP03](ap03-explicit-closed-enum-cases/README.md) |
| Types | One `of` application form for built-in and user generic types | None; see [AP24](ap24-generic-data-structures/README.md) |
| Arguments | Fully positional versus fully named | None; see [AP09](ap09-named-arguments/README.md) |
| Records | Typed structural construction versus factory | None; see [AP10](ap10-typed-record-construction/README.md) |
| Domain types | Subrange restriction versus distinct identity | Decided in [AP18](ap18-subrange-types/README.md) and [AP19](ap19-distinct-domain-types/README.md) (Q10–Q13) |
| Contracts | Contract violation versus expected domain error (`Result`) | Q15–Q17 are recorded in [AP23](ap23-preconditions-and-postconditions/README.md); postconditions on `try` error returns remain open |
| Scopes | Owned child task versus detached work | Q19–Q20 are recorded in [AP26](ap26-structured-task-scopes/README.md); closure-capture limits remain open |

## Integrated target example

This uncompiled draft combines [AP03](ap03-explicit-closed-enum-cases/README.md),
[AP13](ap13-explicit-block-boundaries/README.md),
[AP20](ap20-nested-patterns-and-explicit-bindings/README.md),
[AP21](ap21-decision-expressions/README.md),
[AP24](ap24-generic-data-structures/README.md), and
[AP25](ap25-conservative-purity/README.md).
AP03 closed-enum cases, AP13 block boundaries, AP20 pattern bindings, AP21
decision expressions, and AP24 generic data declarations are implemented.
Purity remains a draft owned by AP25, so this example is not a runnable
program and does not request a combined implementation.

```pascal
type Lookup of T = enum
  Found(Value: T);
  Missing;
  Failed(Message: string);
end enum;

pure function Describe(Response: Lookup of string): string;
begin
  return case Response of
    when Lookup.Found(const Text): 'Found: ' + Text;
    when Lookup.Missing: 'Not found';
    when Lookup.Failed(const Message): 'Error: ' + Message;
  end case;
end function;
```

There is no automatic free-function receiver lookup, extra lambda syntax, or
closed-enum catch-all. Statement and block endings follow the agreed AP13
rules; the `case` expression syntax follows AP21.

## Design inspiration, not API commitments

| Inspiration | Design idea |
|-------------|------------------|
| Pascal | Familiar binding words, `of` type application, subranges, explicit assignment |
| Delphi | Distinct type declarations (`type X = type Y`), conditional expressions |
| Ada | Named block endings, restricted ranges, named argument associations |
| Ada and Swift | Visible mutable arguments at the call site |
| Eiffel, Dafny, and SPARK | Simple `requires`/`ensures` contracts |
| Haskell | Generic algebraic data, distinct domain types, nested patterns |
| Lisp/Scheme | Value-producing decisions, function values, structured editing |
| Modula | Clear public module contracts |
| Oberon and Gleam | Fewer language rules and exceptions |
| Elm | Diagnostics that support concrete corrections |
| Swift and Kotlin | Structured concurrency scopes for child tasks |

These are design influences, not claims of identical grammar or semantics.
Before implementation, verify relevant primary references for the selected
idea and the current FPAS checkout.
