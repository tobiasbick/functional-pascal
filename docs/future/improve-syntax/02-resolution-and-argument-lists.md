# Resolution and argument lists

See the [steering document](README.md) for prerequisites and language gates.
All example spellings are proposals until specified and implemented.

## AP05: Qualified imports

Current behavior: `uses` brings every public symbol of a unit into scope under
its short name; ambiguous short names are reported at the point of use, and the
fully qualified name always works ([grammar](../../specs/grammar.ebnf), `uses_clause`).

### Decided rules (Q04)

- A plain `uses Std.Str;` keeps opening public symbols under their short names.
  Ambiguous short names are errors at the point of use; qualified names remain
  available.
- Add `uses Std.Str as Text;`. An aliased import opens no short names; its
  symbols are reached only through the alias, for example `Text.Trim(Input)`.
- Diagnose an alias that collides with another alias or a local name.

### Tasks

- [ ] Implement aliases and preserve the default behavior of plain imports.
- [ ] Test plain imports, alias-only access, ambiguous short names, and alias
  collisions with other aliases and local names.
- [ ] Keep public parameter and result types explicit; diagnose alias collisions.
- [ ] Verify that importing a colliding helper cannot change a qualified call.

Acceptance: qualified lookup is stable under unrelated imports, with concrete
collision diagnostics, and the default meaning of `uses` is a recorded decision.

## AP06: Remove automatic receivers

Current behavior: `Value.Name(Args)` may resolve to any visible free routine
whose first parameter accepts the receiver, with layered lookup rules
([receiver calls](../../pascal/language/functions/fluent-calls.md)).

### Decided rules (Q05)

- Replace automatic free-function receiver chains with nested calls, such as
  `Reduce(Map(Filter(Values, IsEven), Double), 0, Sum)`, or intermediate `const`
  bindings.
- Do not introduce a pipe operator or retain automatic receiver calls for
  routines from the receiver type's declaring unit.
- Dot notation remains available for actual members.

### Tasks

- [ ] Restrict dot notation to actual members; call free functions normally,
  with qualification where needed.
- [ ] Migrate automatic free-function receiver calls using resolved symbols,
  not textual substitution; distinguish real methods and colliding names.
- [ ] Migrate affected examples and libraries without changing member behavior.
- [ ] Test equivalent nested calls and intermediate bindings, rejection of
  free-function receiver calls, and preserved actual-member calls.

Acceptance: an imported free function cannot reinterpret a dot call, actual
record-member calls retain their meaning, and migrated chains use ordinary
calls with equivalent behavior.

## AP07: Boolean rules

Current behavior: `and` binds like `*` and `or`/`xor` like `+`, above the
comparisons, so `X > 0 and Y > 0` is parsed as `X > (0 and Y) > 0`
([operators](../../pascal/language/basics/operators.md)).

### Decided rules

- `and`, `or`, `xor`, and `not` are boolean-only. Comparisons bind more tightly
  than all of them.
- Different logical binary operators cannot be mixed without parentheses (as
  in Ada). A chain of the same operator is valid; `A and B or C` is an error
  whose diagnostic shows `(A and B) or C`.
- `not` binds below the comparisons and above the binary logical operators:
  `not Count > 0` means `not (Count > 0)`, and `not Done and Ready` means
  `(not Done) and Ready`.
- Bit operations become named functions in the new `Std.Bits` unit (Q02):
  `BitAnd`, `BitOr`, `BitXor`, `BitNot`, `ShiftLeft`, `ShiftRight`; `shl` and
  `shr` are no longer keywords. Integer operands of `and`, `or`, `xor`, or
  `not` are diagnosed with the matching function name.

```pascal
if X > 0 and Y > 0 and Z > 0 then
  Accept();
end if;

if (A and B) or C then
  Retry();
end if;

const Mask: integer := BitAnd(Flags, ReadMask);
```

### Decided precedence table (Q03)

From strongest to weakest:

| Level | Operators | Notes |
|-------|-----------|-------|
| 1 | postfix: call, index, field, `with ... end with` | |
| 2 | unary `-`, `try` | |
| 3 | `*`, `/`, `div`, `mod` | |
| 4 | `+`, `-` | |
| 5 | `=`, `<>`, `<`, `>`, `<=`, `>=`, `in`, `is` | non-associative |
| 6 | `not` | |
| 7 | `and`, `or`, `xor` | same-operator chains only |

Comparison chains such as `A < B < C` are errors. The diagnostic suggests
`A < B and B < C`. Mixed logical operators require parentheses, as specified
above; `not A = B` means `not (A = B)`.

### Tasks

- [ ] Verify whether `and` and `or` currently short-circuit; no documentation or
  implementation was found during planning. Specify left-to-right short-circuit
  evaluation for both.
- [ ] Write the complete precedence table into the grammar and the operator page.
- [ ] Add the bit functions and remove integer overloads of the logical
  operators and `shl`/`shr`. No `.fpas` source in the repository currently uses
  `shl`, `shr`, `xor`, or integer `and`/`or`; verify Rust-embedded fixtures.
- [ ] Test precedence boundaries, comparison-chain rejection, the mixing error,
  `not` placement, and
  short-circuit evaluation; migrate changed expressions with explicit
  parentheses where necessary to preserve intended behavior.

Acceptance: evaluation order and precedence are unambiguous and covered by
boundary tests; mixed logical operators require parentheses; migration does not
silently change meaning.

## AP08: Comma-separated parameter lists

Status: rejected and closed by decision Q06. No parameter-separator migration
is planned.

### Decided rules (Q06)

- Keep `;` between declared parameters and `,` between call arguments.
- Each parameter has its own type annotation; grouped declarations such as
  `A, B: integer` remain invalid.
- Diagnostics for comma separators and grouped declarations show the canonical
  form, for example `function Add(A: integer; B: integer): integer;`. Track
  these diagnostics and their regression coverage under AP02.
