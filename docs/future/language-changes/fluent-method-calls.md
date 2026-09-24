# Optional fluent method calls

Status: language proposal. This document specifies an optional receiver-call spelling for existing FPAS routines. It does not describe implemented behavior. No pipe operator or new function declaration syntax is proposed.

## Goal

Allow a value of any type to call a visible routine whose first parameter accepts that value. Keep ordinary function and procedure calls available. Each step uses the previous step's result, so a chain may change type.

```pascal
uses Std.Arrays;

var Filtered: array of integer := Numbers.Filter(IsEven);
var Doubled: array of integer := Filtered.Map(Double);
var Total: integer := Doubled.Reduce(0, Sum);
```

The proposed calls mean `Std.Arrays.Filter(Numbers, IsEven)`, `Std.Arrays.Map(Filtered, Double)`, and `Std.Arrays.Reduce(Doubled, 0, Sum)`. Existing qualified and unqualified ordinary calls remain valid under their current import and ambiguity rules. This spelling changes neither the eager behavior of array operations nor their result types.

The result type controls the next step. `Doubled.Reduce(0, Sum).Filter(P)` is invalid when `Reduce` returns `integer`, unless a visible `Filter` accepts `integer` as its first parameter. A reducer returning an array could be followed by array `Filter`.

## Call rule

For `Receiver.Name(Arguments)`, evaluate `Receiver` once, then evaluate explicit arguments once from left to right. Resolve `Name` statically. For an eligible free routine, pass the receiver as its first argument and the explicit arguments in their written order. After selecting the routine, type-check the resulting call using the same parameter, generic inference, return-type, and error rules as an ordinary call to that resolved routine with `Receiver` first.

Receiver-based candidate selection is a proposed new lookup rule. It can select a routine even when the corresponding unqualified ordinary call would be ambiguous. For example, with both `Std.Arrays` and `Std.Dictionaries` imported, `Items.Map(F)` could select the array routine from the receiver type, while `Map(Items, F)` remains ambiguous and requires qualification.

The rule applies to visible named functions and procedures, including user-defined routines and imported public `Std.*` routines. The routine does not need to return its receiver type. A function may return another collection, a scalar, `Option`, or `Result`. The point call performs no implicit conversion, unwrapping, error propagation, retry, or lazy evaluation.

Receiver-call resolution should follow these rules:

1. An existing accessible record instance method wins. Existing record-member visibility and static-member errors retain their meanings.
2. Otherwise, consider visible routines named `Name` whose first parameter accepts the receiver type under the ordinary call's type rules. An imported unit must still appear in `uses`.
3. If exactly one routine matches, type-check its remaining arguments as an ordinary call. If several match the receiver type, report their qualified names as an ambiguity; import order and trailing-argument types must not silently pick one.
4. If none matches, report the receiver type and give a concrete call or import hint where possible. A fully qualified ordinary call remains available to resolve ambiguity.

Function-typed variables and bound method values are outside this initial named-routine rule. Whether they should also support receiver calls needs a separate decision.

The candidate rules still need decisions before implementation:

- Define how lexical shadowing affects the candidate set. Ordinary calls already let a local routine shadow an imported short name; collecting all same-named routines must not silently bypass that rule.
- Define partial generic inference during receiver matching. Type parameters known only from trailing arguments must remain unresolved until the selected routine's full call is checked. Specify when generic constraints can eliminate a candidate without using trailing arguments to resolve ambiguity.
- Define fallback for a same-named record field or callable field. Private members and static members must retain their existing diagnostics, and a real instance method with invalid arguments must not silently fall back to a free routine.

## HTTP example

`Std.Http.Send` already accepts a `Request` and returns `Result of Response, string`. With the proposed rule:

```pascal
uses Std.Http, Std.Results;

var ResponseResult: result of Response, string := Request.Get(Url).Send();
var TextResult: result of string, string := ResponseResult.AndThen(BodyText);
```

`Request.Get(Url).Send().AndThen(BodyText)` is also possible. `AndThen` handles the `Result` explicitly. A direct `.Send().BodyText()` does not unwrap `Result`, so it cannot call `BodyText(Response)` on the wrapped value.

## Procedures and mutable receivers

A procedure has no value to pass to a later step, so it can only end a statement chain. A routine with a mutable first parameter keeps the same argument and mutation rules as its ordinary call. For an ordinary user-defined routine, `mutable` allows reassignment of the local parameter binding; it does not itself require a mutable caller variable or write the binding back to the caller. A temporary is therefore not generally forbidden as a receiver. See [mutable parameters](../../pascal/language/functions/mutable-parameters.md).

`Std.Arrays.Push` and `Pop` have a stricter intrinsic rule: their first argument must be a simple mutable array variable. Receiver calls must preserve that restriction, so `Items.Push(Value)` could be supported for such a variable, while `MakeItems().Push(Value)` would remain invalid. `Items.Push(Value).Map(F)` cannot continue because `Push` returns no collection value. `Pop` returns the removed element, whose type would control any following call.

## Current implementation and work required

The existing parser, AST, and formatter already represent the proposed spelling, but through two paths. `Items.Map(F)` is an `Expr::Call` with a designator, while `MakeItems().Map(F)` and `(Items).Map(F)` use an `Expr::Postfix` method operation. The designator path resolves ordinary qualified calls and record methods; the postfix method path explicitly rejects non-record receivers. Neither path currently rewrites collection receiver calls to free routines.

Implementing this proposal requires receiver-aware lookup and ambiguity diagnostics in both paths, including statement calls; a resolved free or intrinsic call target in compiler lowering; preservation of receiver and argument evaluation order; and type-aware completion, navigation, and signature help in the language service. Intrinsic calls use specialized semantic checks, which must remain authoritative when adding receiver matching. Imported-unit interfaces and generated standard-library declarations must expose enough information for those tools to apply the same resolution rule.

Implementation references: [parser call and postfix paths](../../../crates/fpas-parser/src/parser/expr/postfix.rs), [designator call resolution](../../../crates/fpas-sema/src/check/expr/calls/mod.rs), [postfix method checks](../../../crates/fpas-sema/src/check/expr/postfix.rs), and [array mutation checks](../../../crates/fpas-sema/src/std_registry/builtins/array/mutation.rs).

Current references: [postfix calls](../../pascal/language/functions/postfix-chaining.md), [record methods](../../pascal/language/types/record-methods.md), [unit imports](../../pascal/program-structure/units.md), [array operations](../../pascal/std/collections/array/README.md), and [HTTP](../../pascal/std/network/http.md).

This proposal adds call syntax. The existing string and dictionary collection operations are documented under [string higher-order operations](../../pascal/std/text/str/higher-order.md) and [dictionary operations](../../pascal/std/collections/dict.md).

## Verification when implemented

Cover arrays, dictionaries, strings with matching functions, records with a real method, user units, generic routines, HTTP `Result` chaining, ambiguous imports, lexical shadowing, missing imports, procedures, mutable receivers, and evaluation order. Check designator, parenthesized, and returned-value receivers in both expressions and statements. Distinguish ordinary mutable parameters from the `Push`/`Pop` intrinsic restrictions. Compare receiver calls with ordinary calls to the selected qualified routines. Update the implemented language and tool documentation under `docs/pascal/` only after the behavior exists.
