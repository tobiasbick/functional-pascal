# Rust crate review follow-up

Status: open backlog
Source: read-only review of all 19 workspace crates on 2026-08-01
Scope: correctness, simplification, test gaps, error handling, resource safety, structure, and documentation

No build, test, benchmark, or formatter was run during the review. Every finding must be revalidated against the current checkout before implementation. Line numbers are routing hints, not immutable identifiers.

## Continuation protocol

1. Pick the highest-priority open finding whose dependencies are complete.
2. Re-read the owning module, adjacent tests, and relevant `docs/pascal/` page.
3. Change the finding status from `Open` to `In progress` in its crate file.
4. Confirm whether the fix preserves FPAS syntax and semantics. Stop for explicit user agreement before any language change.
5. Implement the smallest complete slice, including negative and boundary regressions.
6. Update user-facing docs only when observable behavior changes. Plans must not be copied into normative documentation.
7. Run the targeted tests, then `cargo fmt`, `cargo build`, and `cargo test --workspace`. Run targeted FPAS tests when `.fpas` regressions are added.
8. Record the commands and results in the crate file, then set the finding to `Done`. Use `Rejected` only with a written, evidence-based reason.

For performance findings, measure a current baseline before changing code and use `cargo bench-fpas save/compare/record` when the FPAS benchmark suite applies. Do not implement unmeasured optimization claims.

## Recommended implementation order

1. Artifact integrity and data loss: `fpas-build`, `fpas-bundle`, `fpas-std`, `fpas-unit`.
2. Validation chain: `fpas-bytecode`, `fpas-unit`, `fpas-linker`, `fpas-program`, `fpas-vm`.
3. Compiler correctness: `fpas-parser`, `fpas-sema`, `fpas-compiler`, `fpas-project`.
4. Concurrency and editor correctness: `fpas-language-service`, `fpas-lsp`, `fpas-cli`.
5. Source preservation and diagnostics: `fpas-fmt`, `fpas-lexer`, `fpas-diagnostics`.
6. Benchmark harness robustness: `fpas-bench`.

Where several crates implement temporary-file publication or stale-lock handling, agree on one failure model first and reuse it instead of fixing each copy differently.

## Crate index

| Crate | File | Highest priority | Primary concern |
| --- | --- | --- | --- |
| `fpas-bench` | [fpas-bench.md](fpas-bench.md) | Done | all findings completed 2026-08-02 |
| `fpas-build` | [fpas-build.md](fpas-build.md) | Done | all findings completed 2026-08-02 |
| `fpas-bundle` | [fpas-bundle.md](fpas-bundle.md) | Done | all findings completed 2026-08-02 |
| `fpas-bytecode` | [fpas-bytecode.md](fpas-bytecode.md) | Done | all findings completed 2026-08-02 |
| `fpas-cli` | [fpas-cli.md](fpas-cli.md) | P1 | timeout and false-success behavior |
| `fpas-compiler` | [fpas-compiler.md](fpas-compiler.md) | P1 | overflow and object metadata |
| `fpas-diagnostics` | [fpas-diagnostics.md](fpas-diagnostics.md) | P2 | safe rendering and invariants |
| `fpas-fmt` | [fpas-fmt.md](fpas-fmt.md) | P1 | comment preservation |
| `fpas-language-service` | [fpas-language-service.md](fpas-language-service.md) | P1 | stale analysis and unsafe rename |
| `fpas-lexer` | [fpas-lexer.md](fpas-lexer.md) | P2 | recovery and span safety |
| `fpas-linker` | [fpas-linker.md](fpas-linker.md) | Done | correctness findings completed; unmeasured optimization rejected 2026-08-02 |
| `fpas-lsp` | [fpas-lsp.md](fpas-lsp.md) | P1 | ordering and cancellation |
| `fpas-parser` | [fpas-parser.md](fpas-parser.md) | Done | all findings completed 2026-08-02 |
| `fpas-program` | [fpas-program.md](fpas-program.md) | Done | all findings completed 2026-08-02 |
| `fpas-project` | [fpas-project.md](fpas-project.md) | P1 | unit resolution and sidecar trust |
| `fpas-sema` | [fpas-sema.md](fpas-sema.md) | Done | all findings completed 2026-08-02 |
| `fpas-std` | [fpas-std.md](fpas-std.md) | Done | all findings completed 2026-08-02 |
| `fpas-unit` | [fpas-unit.md](fpas-unit.md) | Done | all findings completed 2026-08-02 |
| `fpas-vm` | [fpas-vm.md](fpas-vm.md) | Done | all findings completed 2026-08-02 |

## Shared completion checklist

- [ ] Finding reproduced or otherwise confirmed on the current checkout.
- [ ] File layout stated before implementation; oversized modules split when the change naturally exposes a boundary.
- [ ] Positive, negative, and boundary regressions added.
- [ ] Observable docs updated, or `docs: unchanged` recorded with a reason.
- [ ] Public Rust APIs and relevant `# Errors`/`# Panics` contracts updated.
- [ ] `cargo fmt` passes.
- [ ] `cargo build` passes.
- [ ] `cargo test --workspace` passes.
- [ ] Targeted FPAS suite/benchmark commands recorded when applicable.
