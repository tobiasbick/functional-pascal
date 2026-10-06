# AP01.1: Reference examples

Package: [AP01: Pascal conventions](README.md)

## Scope

Establish five to ten canonical draft examples for indentation, keywords,
parenthesized calls, `:=`, and declarations. They form the Pascal-oriented
reference style for this plan.

## Prerequisites

None.

## Implementation

- Add `reference-style.md` to this package directory with the examples.
- Cover one program, one unit, a record and an enum, a routine with
  parameters, a conditional and a loop, and a call with arguments.
- Use agreed spellings from AP13 (block endings), AP16 (`const`/`var`),
  AP17 (`var` parameters), and AP24 (`of` application). Label every form that
  is not yet implemented as a draft and name its owning package.
- Keep two-space indentation as used by the current formatter.

## Affected areas

- `docs/future/improve-syntax/ap01-pascal-conventions/reference-style.md` (new).

## Migration

None. No source, grammar, or current documentation changes.

## Documentation

Planning documentation only. `docs/pascal/` stays unchanged.

## Verification

- Every example states which parts are current syntax and which are drafts,
  with the owning package.
- No example contradicts a recorded decision in another package.
- Relative links resolve.

## Result

[Reference style](reference-style.md) supplies ten examples covering every
requested construct. Each separates current forms from draft forms and names
the owning packages. The examples retain explicit types and avoid deciding
open grammar details. Delivery and the applicable planning checks are complete
on the working branch.
