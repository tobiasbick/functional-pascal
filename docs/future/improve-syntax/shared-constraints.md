# Shared constraints and regression checks

See the [steering document](README.md) for package completion and language gates.
This document supplies checks, not a second implementation backlog.

## Per-package regression checks

| Area | Test case | Required result |
|------|-------|-----------------|
| Arguments | Two same-typed path parameters | Visible role mapping; invalid names rejected |
| Mutation | Local parameter copy versus caller variable | Only explicitly authorized caller mutation escapes |
| Enums | Add a closed-enum variant | Missing cases identified explicitly |
| Imports | Introduce a colliding helper | Qualified calls retain their target |
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
| Bindings | Immutable name versus mutable/shared value | Deep immutability of handles |
| Constants | Computed initialization versus compile-time constant | Compile-time-only contexts |
| Parameters | Read-only parameter versus `var` caller mutation | None; see AP17 |
| Patterns | Explicit binding versus literal/constant comparison | Shadowing and constructor lookup |
| Types | One `of` application form for built-in and user generic types | None; see AP24 |
| Arguments | Fully positional versus fully named | None; see AP09 |
| Records | Typed structural construction versus factory | None; see AP10 |
| Domain types | Subrange restriction versus distinct identity | Decided in AP18 and AP19 (Q10–Q13) |
| Contracts | Contract violation versus expected domain error (`Result`) | Decided in AP23 (Q15–Q17) |
| Scopes | Owned child task versus detached work | Decided in AP26 (Q19–Q20) |

## Integrated target example

This uncompiled draft combines AP13, AP20, AP21, AP24, and AP25.
It is not a request to implement them in one pass. Settle their rules first.

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

There is no automatic receiver shortcut, extra lambda syntax, or closed-enum
catch-all. Statement and block endings follow the agreed AP13 rules; the
`case` expression syntax follows AP21.

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
