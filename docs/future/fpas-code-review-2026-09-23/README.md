# Functional Pascal source review

Review date: 2026-09-23. Reviewed commit: `b3d3b84c3c1952364a545d786548eb617e3b596a`.

Scope: the Functional Pascal implementations in `lib/`, applications in `apps/`, examples in `examples/`, their tests, and associated documentation. The inventory contains 267 `.fpas` files: 131 in `lib/`, 121 in `examples/`, and 15 in `apps/`. Inspection focused on application state transitions, network parsing, text processing, resource limits, and duplicated example code. This is a targeted review, not proof that every execution path is correct.

No implementation or language specification was changed. Generated declarations in `lib/api/` were treated as intrinsic API declarations; their placeholder bodies are not implementation defects. The earlier [Rust review](../rust-code-review-2026-09-23/) remains separate.

## Reports

- [Correctness](correctness.md): seven findings, including six reproduced through public APIs and one established by source inspection.
- [Performance and maintainability](performance-and-maintainability.md): algorithmic costs, duplication, and unnecessary error propagation boilerplate.
- [Documentation](documentation.md): invalid project examples and a stale shutdown description.
- [Tests and verification](tests-and-verification.md): completed checks, reproduction recipes, and missing regression coverage.

## Priorities

| ID | Priority | Finding | Evidence |
| --- | --- | --- | --- |
| F01 | P1 | Notes directory glob can load a sibling directory | Reproduced |
| F02 | P2 | Successful save retry retains the failure overlay | Reproduced |
| F03 | P2 | Duplicate note IDs select the wrong file | Reproduced |
| F04 | P2 | Redirect resolution changes significant slashes | Reproduced over loopback HTTP |
| F05 | P2 | HTTP framing accepts Pascal integer syntax | Reproduced over loopback HTTP |
| F06 | P2 | URI validation depends on whether a port is present | Reproduced |
| F07 | P2 | SSE error handling retains oversized input | Source inspection |
| P01 | P2 | UTF-8 conversion repeatedly scans or copies prefixes | Source inspection |
| P02 | P3 | TUI queues copy the remaining queue on every pop | Source inspection |
| P03 | P3 | Text area caret lookup repeatedly measures prefixes | Source inspection |
| P04 | P3 | Notes uses quadratic sorting after saves | Source inspection |
| M01 | P3 | Fractal explorers duplicate terminal interaction code | Source inspection |
| M02 | P3 | Notes action IDs are repeated across units | Source inspection |
| M03 | P3 | HTTP setup uses dummy values for simple result propagation | Source inspection |
| D01 | P2 | Project documentation omits required public exports | Extracted example fails to compile |
| D02 | P3 | Example index describes obsolete server shutdown | Source and documentation comparison |

P1 should be addressed first because ordinary directory names can cause edits outside the selected directory. P2 covers incorrect behavior or a concrete resource problem. P3 covers maintenance and scaling concerns. Performance priorities reflect algorithms and call sites; no timing or allocation benchmark was run.

## Verification summary

- FPAS suite: 436 passed, 1 skipped, 0 failed, 437 total.
- Curated Rust example integration tests: 61 passed, 0 failed.
- Formatting check over `apps examples lib`: passed.
- Both application project checks: passed.
- Direct checking of `lib/stdlib.fpasprj` was rejected because normal project loading reserves the `Std` namespace. This command does not validate the standard library through its toolchain loading path. The suite and application checks exercise that path.

The passing suite does not cover the reproduced failures. Suggested regression cases are listed with each finding and collected in the verification report. Interactive applications were not manually exercised in a real terminal, and no external AI endpoint was contacted.
