# AP02.3: Project and build diagnostics

Package: [AP02: Structured diagnostics](README.md)

Status: complete.

## Result

Project, build, linker, CLI, and runner errors use FP4100–FP4999.
Source-read, manifest, lexer, parser, and compiler records retain their code,
category, and authoritative source path through orchestration. Multiple
records remain separate; text is rendered at output boundaries.

## Regression coverage

Project/build/linker and real CLI tests cover missing units, imported source
errors, dependency manifests, cycles, invalid artifacts, warnings and transport.
See the [implementation map](implementation-audit.md).
