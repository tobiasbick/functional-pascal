# `fpas-diagnostics` review follow-up

Classification: diagnostics API and rendering. No language change expected.
Status: all findings open.

| ID | Priority | Evidence | Finding and impact | Implementation direction | Required regression |
| --- | --- | --- | --- | --- | --- |
| DIAG-01 | P2 | `crates/fpas-diagnostics/src/diagnostic.rs:113` | Untrusted path, message, and help text are rendered verbatim. ESC, CR, and LF can manipulate terminal layout or synthesize apparent diagnostics. | Escape unsafe control characters and render multiline content line-by-line with a stable prefix. Keep path and message normalization explicit. | Unicode and Windows paths; ESC, CR, LF, tabs, and multiline help. |
| DIAG-02 | P3 | `crates/fpas-diagnostics/src/diagnostic.rs:27,86` | `Diagnostic` publicly stores both `code` and derived `stage`, allowing callers to make them contradictory. | Remove stored `stage` and derive it from `code`, or encapsulate mutation behind invariant-preserving APIs. | Construction and mutation paths always report the stage derived from the code. |
| DIAG-03 | P3 | `crates/fpas-diagnostics/src/code.rs:11`, `src/location.rs:3` | Public value constructors panic for code values above 9999 or zero line/column; the panic contract is undocumented. | Offer `try_new`/`TryFrom` for dynamic inputs and make fields private where practical. If panic constructors remain, add `# Panics`. | Zero, maximum, overflow, stage boundaries, and malformed dynamic inputs. |

## Implementation notes

The current four-module layout is already focused and needs no split. Also specify and test `SourceSpan` overflow behavior for `offset + length`. Public API docs should describe one-based locations and all validation contracts.
