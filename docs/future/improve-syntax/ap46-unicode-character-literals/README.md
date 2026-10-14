# AP46: Unicode character-code literals

Status: complete (AP46.1).
Effort: small. Completion is tracked in the [central plan](../README.md).

## Design boundary

Strings, indexing, Ord, `string.Chr` and `#N` literals use the same Unicode scalar
domain. Decimal codes denote characters rather than raw encoded bytes: `#8364`,
`'€'` and `string.Chr(8364)` have the same value. The
[primitive-type specification](../../../pascal/language/basics/primitive-types.md#character-codes)
describes the implemented literal contract.

## Implemented contract

- `#N` denotes one Unicode scalar with decimal value N: 0 through 1114111,
  excluding the surrogate range 55296 through 57343.
- Preserve decimal digit-separator rules and adjacent string-element
  concatenation. Previously valid literals keep exactly their existing meaning.
- Reject invalid scalars and oversized numeric text lexically, with an
  actionable diagnostic. Do not truncate, wrap, or emit replacement characters.
- Literal values agree with `string.Chr(N)` for valid N and remain compile-time
  known strings. The runtime factory retains its existing dynamic validation.
- Add no character type, byte-string type, hexadecimal escape syntax, or
  normalization/grapheme behavior.

This replaces only the existing 0..255 literal restriction, preserving AP33's
decimal digit-separator contract and AP42's Unicode string operations. The
handbook, grammar and compact reference describe the same scalar domain. No
string representation, artifact format or runtime-factory behavior changes.

## Dependencies and work packages

AP33 and AP42 are complete. AP44 and AP45 are independent.

- [x] [AP46.1: Implement Unicode scalar literals](01-unicode-literals.md)

## Open decisions

No selected AP46 decisions or implementation work remain open.

## Acceptance

Source character codes and the runtime character factory agree across the
Unicode scalar range. Invalid scalar codes receive precise diagnostics;
formatting, source spans, editor tooling and existing literal behavior remain sound.
Independent subagent review found no blocking issues. [AP46.1](01-unicode-literals.md)
records implementation ownership, regression coverage and independent baseline
verification exceptions.
