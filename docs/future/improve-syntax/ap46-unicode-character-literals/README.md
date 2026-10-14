# AP46: Unicode character-code literals

Status: agreed direction; implementation pending.
Effort: small. Completion is tracked in the [central plan](../README.md).

## Problem

Strings, indexing, Ord and `string.Chr` operate on Unicode scalar values, but
`#N` literals accept only 0 through 255. The handbook calls these byte values,
although the lexer converts them to Unicode characters rather than raw bytes.
For example, the current language accepts `'€'` and `string.Chr(8364)` but rejects
`#8364`.

## Agreed contract

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
handbook and grammar continue to describe the current restriction until delivery.

## Dependencies and work packages

AP33 and AP42 are complete. AP44 and AP45 are independent.

- [ ] [AP46.1: Implement Unicode scalar literals](01-unicode-literals.md)

## Open decisions

None within the selected contract.

## Acceptance

Source character codes and the runtime character factory agree across the
Unicode scalar range. Invalid scalar codes receive precise diagnostics;
formatting, source spans, editor tooling and existing literal behavior remain sound.
