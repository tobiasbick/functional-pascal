# Rust standards review

This axis is separate from [behavioral/specification findings](compiler-and-semantics.md). The governing standards are [AGENTS.md](../../../../AGENTS.md): focused concerns, small thematic modules, reuse, no dead code introduced by changes, Rust 2024, public documentation, and synchronized handbook links. Tool-enforced formatting/lint findings are reported in [verification](README.md), not duplicated as subjective review findings.

No independent hard structural defect was confirmed in this slice. The new native catalog is split by receiver family, diagnostics have focused transport/location modules, and production file growth is controlled. A line-count scan of Rust files changed since AP02 found only [crates/fpas-compiler/src/lowering/context/bindings.rs](../../../../crates/fpas-compiler/src/lowering/context/bindings.rs) over 500 lines (501) outside test files. This is not sufficient evidence of an architectural defect: the documented threshold is approximate and that file has a binding ownership concern. Existing large test modules were not reported merely for their size.

A repository-wide scan of literal `docs/pascal/*.md` paths in Rust found no nonexistent target files. The scan checks file existence, not anchors, prose accuracy, or dynamically assembled links. Public documentation comments and small-module organization were inspected in the AP02/AP06 implementation owners; this is not a complete rustdoc audit of every public symbol.

The architectural recommendation tied to [T03](tooling-and-tests.md#t03) is
implemented. The language service reuses the parser's
[type-fragment entry point](../../../../crates/fpas-parser/src/lib.rs) and maps
recursive `TypeExpr` structures in a focused
[editor type module](../../../../crates/fpas-language-service/src/intellisense/native_receiver/types.rs).
Alias declarations and anonymous callback return types use their existing AST;
aliases resolve in the declaring unit's import environment. There is no second
text-based type grammar in native receiver inference. This repair preserves the
current language contract and is counted once as T03.

## Confirmed result and limits

- **Independent hard standards findings: 0.** This does not negate the confirmed Rust correctness defects in C01–C06.
- **Architectural recommendation: 1, resolved under T03.** The shared parser grammar and recursive editor mapping replace the duplicate text-based interpretation; it is not counted as a separate finding.
- At the reviewed revision, build, format checking, Clippy with warnings denied, and the full Rust workspace suite passed. Existing large test files and a file just over the approximate size target are not defects by themselves.
- The review inspected ownership boundaries of completed-package implementation modules and focused on changed production code. It did not perform a line-by-line proof of every Rust file, a release performance benchmark, or a portability run on other operating systems.

Current repair checks and independent validation blockers are recorded in [repair verification](README.md#repair-verification). No syntax or semantic change is part of the T03 repair.
