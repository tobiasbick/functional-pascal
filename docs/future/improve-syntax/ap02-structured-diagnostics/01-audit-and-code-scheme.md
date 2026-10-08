# AP02.1: Diagnostic audit and code scheme

Package: [AP02: Structured diagnostics](README.md)

Status: complete.

## Result

Diagnostics use stable `FPnxxx` codes in text and structured output:

| Range | Producer |
| --- | --- |
| FP1xxx | Lexer |
| FP2xxx | Parser |
| FP3xxx | Semantic analysis |
| FP4000–FP4099 | Compiler |
| FP4100–FP4999 | Project, build, linker, CLI, test runner |
| FP5xxx | Runtime |
| FP9xxx | Internal invariants |

The shared catalog is `crates/fpas-diagnostics/src/codes.rs`. Code identity is
preserved through consumers, with concrete correction hints where applicable.

## Regression coverage

Catalog tests check allocation uniqueness and phase ranges. Producer and CLI
regressions check representative codes, positions, structured details and hints.
See the [implementation map](implementation-audit.md) and
[diagnostics reference](../../../pascal/tools/diagnostics.md).
