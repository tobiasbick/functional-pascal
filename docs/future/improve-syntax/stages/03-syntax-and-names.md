# Stage 3: Syntax and names

Prerequisites: [contract](../language-contract.md) and
[diagnostics](02-diagnostics.md). This stage establishes the grammar and name
rules needed by the [functional core](04-functional-core.md); it does not claim
that future constructs are implemented merely by reserving their words.

## Block grammar

| Construct | Canonical target form |
|-----------|-----------------------|
| Program | `program Name; ... begin ... end program;` |
| Unit | `unit Name; ... end unit;` |
| Function | `function F(...): T; ... begin ... end function;` |
| Procedure | `procedure P(...); ... begin ... end procedure;` |
| Anonymous function | `function(...): T begin ... end function` |
| Anonymous procedure | `procedure(...) begin ... end procedure` |
| Plain lexical block | `begin ... end;` |
| Record / enum declaration | `type Name = record ... end record;` / `enum ... end enum;` |
| Conditional statement | `if C then ... elsif D then ... else ... end if;` |
| Case statement | `case V of when Pattern: ... else ... end case;` |
| Counted loop | `for I: integer := A to B do ... end for;` |
| Collection loop | `for Item: T in Items do ... end for;` |
| While loop | `while C do ... end while;` |
| Repeat loop | `repeat ... until C;` |
| Record update | `Value with Field := Expression; end with` |
| Conditional expression (stage 4) | `if C then A elsif D then B else E end if` |
| Case expression (stage 4) | `case V of when Pattern: E; ... end case` |
| Task scope (stage 5) | `scope ... end scope;` |

Named nested routines follow the same function/procedure rules; `pure` changes
the callable capability, not its closer. Programs no longer have an `end.`
exception. Plain blocks create local declaration scope and cannot close an
enclosing control structure. A repeat loop's condition-bearing `until` already
closes its body; adding a redundant second closer provides no useful boundary.

Every body statement ends with `;`, including the final statement before `else`,
`elsif`, `when`, `until`, or a named closer. Empty statement bodies contain `null;`;
standalone empty statements and empty body lists are invalid. A branch needs no
`begin`; an explicit plain block inside it still needs its own `end;`.

`else if` starts a nested conditional requiring its own closer. `elsif` continues
the same conditional. Diagnose a missing nested closer with an `elsif` hint.
The required semicolon before `else` is valid; an extra empty statement is not.

Expressions do not acquire statement terminators. Case-expression arm semicolons
delimit arms; they do not terminate the enclosing expression. An anonymous routine
in an argument closes directly before `,` or `)`, without an intervening `;`.

## Declarations and lookup

Use one keyword per declaration, also with `public`. Formal parameters remain
individually typed and semicolon-separated; actual arguments are comma-separated.
Type references are resolved across the unit/program before checking bodies;
constant/variable initializers retain declaration-order rules.

Imports always declare an alias, one unit per declaration:

```pascal
uses Std.Str as Text;
uses Std.Console as Console;
```

An alias is the only import access path; it opens no short names. Reserve it in
the lexical namespace so a local declaration cannot silently shadow it. Repeated
imports of the same unit under multiple aliases are rejected. Visibility checks
remain in effect for fields, types, and routines. A type and routine may not
share a name in the same scope. Case-insensitivity applies to all these checks.

Removing method/receiver/property/event syntax happens with its usable replacement
in stage 4, not as a premature parser deletion in this stage. Introduce each new
keyword with its owning implemented construct and migration diagnostic. In the
delivered language, obsolete `mutable`, `property`, `read`, `write`, `event`,
`nil`, `shl`, and `shr` no longer have their old language roles; token-aware
diagnostics must not accidentally forbid otherwise valid identifiers.

## Operators

From strongest to weakest:

| Level | Operators | Rule |
|-------|-----------|------|
| 1 | call, indexing, field selection, `with ... end with` | Postfix |
| 2 | unary `-`, `try` | Prefix |
| 3 | `*`, `/`, `div`, `mod` | Left associative |
| 4 | `+`, `-` | Left associative |
| 5 | `=`, `<>`, `<`, `>`, `<=`, `>=`, `in` | Non-associative |
| 6 | `not` | Boolean prefix |
| 7 | `and`, `or`, `xor` | Same-operator chains only |

`X > 0 and Y > 0` tests two comparisons. `not X > 0` means `not (X > 0)`.
`A and B or C` asks for parentheses rather than choosing an interpretation.
`A < B < C` is rejected with a two-comparison example. Migration must preserve
evaluation count when suggesting a rewrite of a side-effecting middle expression.

Provide `Std.Bits.BitAnd`, `BitOr`, `BitXor`, `BitNot`, `ShiftLeft`, and
`ShiftRight` as explicitly imported operations on 64-bit integer bit patterns.
Shifts accept counts 0 through 63; other counts fail under the checked-operation
rules. Left shifts discard shifted-out bits, and right shifts zero-fill. Bitwise
functions are pure; their pattern semantics do not change checked arithmetic.

## Work

- [ ] Inventory lexer tokens, parser productions/recovery, formatter/comment
  attachment, generated source, and editor snippets before modifying grammar.
- [ ] Implement block endings, terminators, declaration grouping removal, and
  qualifier-only resolution with targeted diagnostics.
- [ ] Implement the precedence/evaluation table and bit-function replacements.
  Inspect existing short-circuit lowering rather than assuming it is absent.
- [ ] Convert sources through resolved syntax/symbols. Preserve scopes, dangling
  branch ownership, comments, and evaluation order; reject ambiguous migrations.
- [ ] Update grammar, applicable handbook pages, formatter style, authoring
  guidance, source templates, and editor formatting together when implemented.
- [ ] Test every block-table row as its construct lands, missing/mismatched
  closers, extra/missing semicolons, nested branches, lexical visibility, import
  collisions, precedence boundaries, side effects, and bit-count boundaries.

## Acceptance

Implemented constructs have one grammar and canonical formatter output. Parse,
format, and reparse preserve structure; a second format is identical. Imported
names stay stable under unrelated imports. CLI and editor formatting agree.
Diagnostics teach the replacement without retaining an old-language mode.

Owners: `fpas-lexer`, `fpas-parser`, `fpas-fmt`, `fpas-sema`, `fpas-project`,
`fpas-compiler`, `fpas-bytecode`, `fpas-std`, `fpas-vm`, and editor integrations.
Current docs to migrate include basics/operators, control-flow, program structure,
and `docs/pascal/tools/fmt-style.md`; preserve grammar-production links.

Status: target syntax settled; implementation and migration pending.
Next: after diagnostics, inventory closing-token and import-resolution consumers.
