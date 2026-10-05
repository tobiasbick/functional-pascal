# AP10.2: Migrate record literals

Package: [AP10: Typed record construction](README.md)

## Scope

Rewrite every `record ... end` literal to typed construction while both forms
are valid.

## Prerequisites

- AP10.1 (typed construction).

## Implementation

- Take the record type from each literal's expected type as resolved by sema;
  qualify the type name where needed.
- Preserve field values and the observable evaluation order. Where the old
  order differs from written order and a field has side effects, introduce an
  intermediate binding.
- About 590 literal sites existed in `.fpas` sources at planning time.

## Affected areas

- `.fpas` sources under `lib/`, `apps/`, `examples/`, `tests/`; Rust-embedded
  fixtures; formatter goldens; templates; documentation examples; skills.

## Migration

This work package is the migration; the conversion tool is not merged.

## Documentation

Documentation examples move to typed construction.

## Verification

- No `record ... end` literal remains outside negative tests.
- Full FPAS suite, example/app checks, and workspace tests pass unchanged.
