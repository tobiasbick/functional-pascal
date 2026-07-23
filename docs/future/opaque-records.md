# Opaque records and transient capabilities

FPAS records currently expose their representation and can be constructed and copied by callers.
That prevents a library from using a record as an unforgeable capability.

The concrete motivating case is a transient `TuiCanvas`: a library cannot prove that a canvas was
created for the active paint callback, because application code can construct or retain an
equivalent public record. Current TUI APIs therefore must not promise runtime enforcement of that
lifetime.

## Possible language direction

Add either opaque record handles or per-field visibility with constructors callable only inside the
declaring unit. A future transient canvas could then contain a private application generation token
and reject operations after its paint callback ends.

This needs compile-time tests for representation and constructor visibility, plus a runtime test
that rejects a stale capability. It is a language-design item, not a blocker for the current TUI
MVU implementation until custom painting is introduced.
