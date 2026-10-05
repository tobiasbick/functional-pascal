# AP02.3: Project and build diagnostics

Package: [AP02: Structured diagnostics](README.md)

## Scope

Give every project, build, and linker failure and warning a code in the
project/build range and carry structured records, not formatted text, through
project loading and build APIs.

## Prerequisites

- AP02.2 (shared diagnostic record).

## Implementation

- Assign codes to project loading, dependency, standard-library loading,
  graph/snapshot, build, and linker failures and warnings.
- Carry source-read, lexer, and parser records through `fpas-project` errors.
- Keep compiler and parser records in build errors; render text only at the
  display boundary. Preserve source paths of imported units.

## Affected areas

- `crates/fpas-project/src/` (errors, source reading, unit graph).
- `crates/fpas-build/src/` (engine errors), `crates/fpas-linker/src/`.
- `crates/fpas-diagnostics/src/codes.rs`.

## Migration

Adapt CLI and language-service consumers to the structured errors. No FPAS
source change.

## Documentation

Add the new codes to the diagnostics page (AP02.5 or the page started in AP02.2).

## Verification

- Tests for failures in imported units, missing units, cyclic graphs, invalid
  manifests, and linker failures assert codes and source paths.
- Multiple diagnostics from one build stay separate records.
