# Scope and decisions

## Accepted scope

- UBA-S01: initialize a visible mutable local through a current Variables
  handle.
- UBA-S02: initialize a visible mutable global through a current Globals
  handle.
- UBA-S03: initialize either root through case-insensitive textual assignment.
- UBA-S04: expose the same behavior through JSONL, DAP, and VS Code.
- UBA-S05: preserve evaluation limits, atomicity, selected-task ownership, and
  stopped-state handle invalidation.
- UBA-S06: represent initialized `unit` independently from empty storage.

## Fixed decisions

- UBA-D01: track register initialization explicitly; `Value::Unit` is a valid
  initialized value and is not a sentinel.
- UBA-D02: initialization applies only to complete roots with portable debug
  type metadata.
- UBA-D03: source execution remains authoritative. A later initializer
  overwrites a debugger value normally.
- UBA-D04: missing parameters and captures are unavailable, because valid
  frames create them initialized and capture cells cannot be synthesized.
- UBA-D05: every successful mutation expires handles and refreshes clients in
  the same way as replacement of initialized values.
- UBA-D06: no FPAS syntax, semantics, or language documentation changes are
  part of this package.

## Exit criteria

The package is complete only when every in-scope matrix row has a regression,
all required verification commands pass, the progress log records the evidence,
and deferred boundaries remain documented separately.
