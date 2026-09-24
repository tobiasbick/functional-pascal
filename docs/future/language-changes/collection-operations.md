# Collection operations for arrays, dictionaries, and strings

Status: proposal. This document records API gaps and decisions to make before implementation. The current language and standard-library behavior is documented under `docs/pascal/`.

## Current behavior

| Type | Map | Filter | Reduce | Other relevant operations |
| --- | --- | --- | --- | --- |
| `array of T` | `Std.Arrays.Map` | `Std.Arrays.Filter` | `Std.Arrays.Reduce` | `FlatMap`, `ForEach`, `Find`, `Any`, `All` |
| `dict of K to V` | `Std.Dictionaries.Map` transforms values and preserves keys | `Std.Dictionaries.Filter` receives key and value | None | `Keys` and `Values` return arrays in insertion order |
| `string` | None | None | None | `Std.Str` has indexed character access, text-specific transforms, `Split`, and `Join` |

Array `Reduce(A, Init, F)` folds left to right from an explicit initial value. The existing collection functions are eager free functions. Postfix method calls on expression results currently resolve record instance methods, so `Values.Map(F)` is not a supported spelling for an array operation.

Strings are worth treating as collections for these operations. Their current character indices count Unicode scalar values, and `CharAt` returns a one-scalar `string`. A scalar can differ from a user-perceived grapheme cluster. `for ... in` currently supports arrays and dictionary keys, but not strings.

Current references: [array operations](../../pascal/std/collections/array/README.md), [dictionary operations](../../pascal/std/collections/dict.md), [string operations](../../pascal/std/text/str/README.md), [iteration](../../pascal/language/control-flow/for-in.md), and [postfix calls](../../pascal/language/functions/postfix-chaining.md).

## Shared contract and its limits

The useful common contract for finite collections is: `Map` transforms each logical element, `Filter` retains elements whose predicate is true, and `Reduce` visits elements in a defined order from an explicit initial accumulator. Empty-input behavior, output order, and callback evaluation should follow the same rules in every unit that offers these names. For arrays and strings, a logical element is one array value or one Unicode scalar, respectively. A dictionary element is a key-value entry, even though its existing `Map` callback receives only the value and preserves the key. Matching behavior therefore does not require identical callback signatures.

Existing names outside this trio are not fully interchangeable. `Std.Arrays.Contains` checks for an element, `Std.Str.Contains` checks for a substring, and `Std.Dictionaries.ContainsKey` checks for a key. Array `IndexOf` searches for an element while string `IndexOf` searches for a substring. `Length` counts elements, entries, or Unicode scalars. Changing these meanings merely to make names identical would alter existing behavior. Short names shared by imported units can also be ambiguous; qualified calls remain necessary under current name resolution.

Other types do not need the entire collection API:

| Type | Current operation | Assessment |
| --- | --- | --- |
| `Option of T` | `Std.Options.Map` transforms `Some` and leaves `None` unchanged | A zero-or-one value can use `Map`, but forcing all sequence operations onto it adds little. |
| `Result of T, E` | `Std.Results.Map` transforms `Ok` and preserves `Error` | A failed `Filter` predicate has no specified error value; a `Reduce` that preserves errors would need a different result type. |
| `channel of T` and stream handles | Send/receive or explicit read operations | They can block, fail, or have no known end, so eager collection behavior does not apply. |
| Records and enums | Fields or variants, rather than a built-in element sequence | No general element order or common collection callback exists. |

Current references: [Option](../../pascal/std/result/option.md), [Result](../../pascal/std/result/result.md), and [channels](../../pascal/language/types/channels.md).

## Proposed additions

1. Add `Std.Str.Map`, `Filter`, and `Reduce` as eager operations over the string's Unicode scalar values. `Map` and `Filter` would produce strings; `Reduce` would produce the accumulator type. Preserve left-to-right order and use an explicit initial value for `Reduce`, matching `Std.Arrays`.
2. Add `Std.Dictionaries.Reduce` with an explicit initial value and a callback that receives the accumulator, key, and value. Traverse entries in insertion order. Reducing `Values(D)` already works when the keys are irrelevant; the direct operation covers reductions that need both parts of an entry.

These are proposed APIs, not implemented signatures. In particular, the exact callback types and behavior when a string mapping returns more than one scalar remain undecided.

## Decisions before implementation

- **String element:** The proposed baseline uses Unicode scalars, consistent with current indexing. Grapheme-cluster iteration is an alternative to decide explicitly. It would change callback inputs and `Map` output validation, but would not by itself change the existing scalar-based `Length` or indexing rules.
- **String `Map` result:** Require exactly one Unicode scalar per callback result, or permit arbitrary strings and concatenate them. The latter also permits deletion and expansion, so it overlaps with `Filter` and `FlatMap`.
- **Dictionary `Reduce` callback:** Finalize the proposed accumulator, key, value argument order and generic types. Reuse the existing [closure capture rules](../../pascal/language/functions/closures.md), which already allow callbacks to mutate captured mutable bindings. Any additional restriction would need a separate decision.
- **Common behavior:** Specify empty-input results, insertion or character order, callback invocation order, and error propagation for each added operation. Keep the current meaning of existing `Contains`, `IndexOf`, and `Length` operations.
- **Call syntax:** Decide separately whether built-in collections should gain receiver-style calls such as `Values.Map(F)`. The proposed free functions do not depend on that change. No pipe operator is proposed.
- **Scope:** Decide whether `for ... in` should accept strings. It is independent of the three proposed `Std.Str` functions.

## Verification if approved

Document the final APIs under `docs/pascal/` and cover empty collections, order, Unicode scalars, callback errors, and accumulator types with FPAS regression tests. Check compiler and language-service diagnostics for invalid callback types. Changes to collection call syntax or string iteration require a separate language decision.
