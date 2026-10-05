# AP03.2: Reject catch-alls on closed enums

Package: [AP03: Explicit closed-enum cases](README.md)

## Scope

Make `else` on a `case` over a closed enum an error and require every variant
to be listed explicitly. Guarded arms do not count as coverage.

## Prerequisites

- AP03.1 (repository has no closed-enum catch-alls).
- AP02 (diagnostic code).

## Implementation

- In the exhaustiveness check, reject `else` when the scrutinee is a closed
  enum, `Option`, or `Result`. Keep `else` for open domains.
- The diagnostic lists the variants the `else` replaced and suggests a
  `null;` arm or an `is` test.
- A guarded arm alone does not cover its variant; a field wildcard `_` does
  not cover a missing variant.

## Affected areas

- `crates/fpas-sema/src/check/stmt/control_flow/if_case/` (exhaustiveness).
- `crates/fpas-diagnostics/src/codes.rs`.

## Migration

None beyond AP03.1.

## Documentation

- `docs/pascal/language/pattern-matching/exhaustiveness.md` and
  `docs/pascal/language/control-flow/case-of-intro.md`.
- Diagnostics reference entry.

## Verification

- Add a variant to a test enum: every incomplete case reports the missing
  variant; complete cases remain accepted.
- Tests for `else` on user enums, `Option`, and `Result`; guarded-only arms;
  field wildcards; open-domain `else` still valid.
