# Scope and decisions

## Accepted scope

- VTA-S01: textual assignment to an explicitly qualified inactive
  single-payload enum variant, such as `Selected.Item.value`.
- VTA-S02: textual `Option.None` to `Option.Some` transition through
  `Optional.Some.value`.
- VTA-S03: textual `Result.Ok`/`Result.Error` branch transition through
  `Outcome.Ok.value` or `Outcome.Error.value`.
- VTA-S04: active-variant qualified targets normalize to the existing payload
  replacement behavior.
- VTA-S05: local, global, parameter, capture-cell, and nested writable roots
  follow existing mutation ownership rules.
- VTA-S06: JSONL `expression.set`, DAP `setExpression`, and VS Code textual
  mutation expose identical behavior through the shared session.

## Fixed decisions

- VTA-D01: the variant name is mandatory and matched exactly,
  case-insensitively, against portable executable metadata. An unqualified
  inactive field never selects a variant implicitly.
- VTA-D02: the target variant must have exactly one payload slot and the target
  must end in that slot's exact name. The assignment therefore supplies the
  complete payload.
- VTA-D03: fieldless targets continue to use complete-root constructor
  replacement. Multi-field targets require a complete constructor expression.
- VTA-D04: no stale `variablesReference`, child handle, frame, or task handle is
  revived. Successful mutation keeps the existing global invalidation policy.
- VTA-D05: an uninitialized root must first receive one complete root value;
  transition assignment cannot synthesize its outer storage.
- VTA-D06: failed lookup, evaluation, validation, or construction performs no
  live write and emits no invalidation event.
- VTA-D07: this is debugger-target behavior only. FPAS syntax, semantics, and
  language documentation do not change.

## Stable errors

Reuse the existing protocol error families:

- `variable_target_unknown` for an unknown explicit variant or payload name;
- `variable_path_unsupported` for unqualified inactive fields, empty roots,
  fieldless variants, or multi-field partial construction;
- `variable_not_mutable` and `variable_unavailable` for existing ownership and
  lifetime failures; and
- `variable_value_type`, evaluation-limit, cancellation, and effect-policy
  errors from the current evaluator.

Every error needs a concrete hint showing either the qualified descendant form
or the existing complete-constructor alternative.
