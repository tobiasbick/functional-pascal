# AP02.2: Shared diagnostic record

Package: [AP02: Structured diagnostics](README.md)

Status: complete.

## Result

`Diagnostic` carries code, severity, message, optional span, expected, found,
and hint. Phase is derived from the code. `FileDiagnostic` carries the
source path; a span retains source identity.

Byte spans resolve to one-based Unicode-scalar line and column coordinates.
Real end positions are exclusive. Missing positions stay absent; runtime
points have a start without an invented end. Token and type producers supply
expected/found directly. Text and JSON render the same records, and editor
adapters convert coordinates to the negotiated encoding.

## Regression coverage

Shared-record, Unicode-range, parser expectation, type-mismatch, and editor
conversion tests cover absent and synthetic locations.
See the [diagnostics reference](../../../pascal/tools/diagnostics.md).
