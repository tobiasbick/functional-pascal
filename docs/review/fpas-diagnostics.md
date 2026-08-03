# `fpas-diagnostics` review follow-up

Classification: diagnostics API and rendering. No language change expected.
Status: all findings completed 2026-08-03.

| ID | Priority | Evidence | Finding and impact | Implementation direction | Required regression |
| --- | --- | --- | --- | --- | --- |
| DIAG-01 | P2 | `crates/fpas-diagnostics/src/render.rs`, `crates/fpas-build/src/engine.rs`, `crates/fpas-project/src/common.rs` | **Done.** Untrusted path, message, and help text were rendered verbatim. ESC, CR, and LF could manipulate terminal layout or synthesize apparent diagnostics. | Escape unsafe control characters and render multiline content line-by-line with a stable prefix. Keep path and message normalization explicit. Replace duplicate rendering in `fpas-build` and the raw outer path in `fpas-project`. | Unicode and Windows paths; ESC, CR, LF, tabs, and multiline help. |
| DIAG-02 | P3 | `crates/fpas-diagnostics/src/diagnostic.rs` | **Done.** `Diagnostic` publicly stored both `code` and derived `stage`, allowing callers to make them contradictory. | Remove stored `stage` and derive it from `code`, or encapsulate mutation behind invariant-preserving APIs. | Construction and mutation paths always report the stage derived from the code. |
| DIAG-03 | P3 | `crates/fpas-diagnostics/src/code.rs`, `src/location.rs`, `src/span.rs`, `crates/fpas-lexer/src/span.rs` | **Done.** Public value constructors could panic for dynamic out-of-range codes, zero coordinates, or overflowing spans without a fallible alternative. | Offer `try_new`/`TryFrom` for dynamic inputs, keep fields private where practical, and document the remaining static panic constructors. | Zero, maximum, overflow, stage boundaries, and malformed dynamic inputs. |

## Implementation notes

Locations, spans, and terminal rendering now have separate focused modules. Control escaping and
normalized multiline layout form one concern, while `SourceSpan` owns byte-range overflow behavior
for `offset + length`. Public API docs describe one-based locations and all validation contracts.

## Implementation record

- Terminal rendering now preserves printable Unicode, escapes control characters, keeps paths on
  one line, normalizes all common line endings, and prefixes every message/help continuation.
- `fpas-build` and `fpas-project` use the shared renderer instead of maintaining unsafe alternate
  formatting paths.
- `Diagnostic` no longer stores a mutable derived stage; `Diagnostic::stage()` always derives it
  from the current code.
- Dynamic diagnostic codes, locations, and spans have fallible constructors with typed errors.
  The panic contracts for static constructors are explicit.
- `SourceLocation` and `SourceSpan` fields are private. Accessors, source-ID rebasing, and the
  overflow-safe `SourceSpan::end()` preserve their invariants across the workspace.
- Conversion from the lexer's publicly constructible `Span` is fallible. Internal diagnostic paths
  use an explicit non-panicking synthetic fallback for malformed spans.
- Invalid serialized program-image locations are rejected through fallible construction at the raw
  payload boundary. No FPAS syntax or semantic behavior changed.
- User documentation: `docs/pascal/program-structure/cli.md`.

## Verification

- `cargo test -p fpas-diagnostics`: 27 tests passed.
- `cargo test -p fpas-lexer span::tests`: 3 targeted conversion tests passed.
- `cargo fmt --all -- --check`: passed.
- `cargo build --workspace`: passed.
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`: passed.
- `cargo test --workspace`: passed.
- No `.fpas` source changed, so no targeted FPAS formatting or suite run was required.
