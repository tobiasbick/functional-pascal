# Future: Observability

> Deferred. Console text is not a structured operational interface.

Long-running applications need machine-readable evidence about requests, background work,
dependencies, resource limits, and shutdown. Observability should report facts without coupling
application logic to a console, file, or network collector.

## Proposed scope

- Structured log events with level, timestamp, event name, message, and typed fields.
- Pluggable console and JSON-lines adapters, with file rotation considered only after the core event
  seam is stable.
- Counters, gauges, and duration histograms with bounded label cardinality.
- Liveness and readiness snapshots consumable by an HTTP adapter or local administration command.
- Correlation identifiers that applications can pass across tasks, storage, and transports.
- Redaction helpers and field classifications for credentials, tokens, personal data, and payloads.

## Interface rules

- Logging must not format secrets before redaction policy runs.
- A slow or failed adapter cannot block request processing indefinitely or grow an unbounded queue.
- Metrics names and label sets are registered explicitly; arbitrary user or peer identifiers are not
  labels.
- Health checks report dependency state without mutating dependencies or exposing confidential
  diagnostics.

## Acceptance requirements

- Concurrent emitters preserve complete events without corrupting structured fields.
- Adapter failure, queue saturation, shutdown flush, and redaction have deterministic tests.
- Request and background-operation correlation survives task handoff.
- Readiness changes align with server admission and shutdown phases.
- Test adapters can assert events and metrics without parsing presentation text.
