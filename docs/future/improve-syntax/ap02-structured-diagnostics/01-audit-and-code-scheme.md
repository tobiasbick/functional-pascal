# AP02.1: Diagnostic audit and code scheme

Package: [AP02: Structured diagnostics](README.md)

## Scope

Audit the existing diagnostic model and producers, apply the confirmed code
scheme, and improve a representative set of existing errors before building
anything new. Do not create a parallel error system.

## Prerequisites

- The open decision on code numbering in the [package README](README.md).

## Implementation

- Inventory `crates/fpas-diagnostics` (code type, code catalog, record,
  location, span, renderer) and every producer: lexer, parser, sema, compiler,
  VM, project, build, linker, CLI, runner, and language service.
- Apply the confirmed numbering scheme to the catalog and its range tests.
  If codes are renumbered, update every test, golden, and document that
  mentions a code.
- Show the code in all human-readable output.
- Improve a representative set of frequent errors with a source position,
  short explanation, expected versus found value or type, and a concrete
  correction hint that does not guess business decisions.

## Affected areas

- `crates/fpas-diagnostics/src/` (`code.rs`, `codes.rs`, `diagnostic.rs`,
  `render.rs`).
- Producers in `fpas-lexer`, `fpas-parser`, `fpas-sema`, `fpas-compiler`,
  `fpas-vm`; tests and goldens that assert codes or messages.

## Migration

Rewrite asserted codes and messages in tests and goldens. FPAS sources are
unaffected.

## Documentation

Update any current page that cites a code. The full reference follows in AP02.5.

## Verification

- Code uniqueness and phase-range tests pass for the confirmed scheme.
- Tests for each improved representative error assert the code, position,
  expected/found details, and hint.
- Required checks from the [process](../development-process.md#required-checks).
