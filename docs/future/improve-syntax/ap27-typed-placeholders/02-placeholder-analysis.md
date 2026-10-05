# AP27.2: Placeholder analysis

Package: [AP27: Typed placeholders](README.md)

## Scope

Accept the agreed placeholder in incomplete code for analysis, report expected
types, local values, and matching signatures, and reject unresolved
placeholders when compiling an executable.

## Prerequisites

- AP27.1 (spelling), AP22 complete, AP02.4 (JSON diagnostics for tools).

## Implementation

- Parser: the placeholder as an expression.
- Sema: compute the expected type at the placeholder and list local values and
  visible routines with a matching result type, within AP22's inference limits.
- `fpas check` and the language service report the analysis; `build`, `run`,
  and `test` reject unresolved placeholders.

## Affected areas

- Parser expressions, sema expected-type propagation, `fpas-cli` command
  handling, `fpas-language-service`.

## Migration

None.

## Documentation

- New page under `docs/pascal/tools/` or `docs/pascal/language/`, diagnostics
  reference, `docs/specs/grammar.ebnf`.

## Verification

- Placeholders in argument, return, and initializer positions; reported
  candidates; rejection in executable builds; no inference beyond AP22.
