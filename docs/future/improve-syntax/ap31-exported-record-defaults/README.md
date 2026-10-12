# AP31: Exported record constant defaults

Status: complete (AP31.1).
Effort: small to medium. Completion is tracked in the [central plan](../README.md).

## Result

Exported record defaults preserve the values promised by the
[record specification](../../../pascal/language/types/records.md#default-field-values).
The existing constant evaluator checks them in the declaration environment;
unit interfaces store the resulting scalar values and optional `None` defaults.
Literals, expressions, and preceding constant references behave identically
through independent compilation, transparent aliases, and sidecar reuse.

## Scope

Computed, aggregate, and unsuccessfully evaluated exported defaults produce
FP3014 at their original source field. Program-local and non-exported defaults
retain their declaration environment and may use preceding values and routines.
Scalar fields of generic records follow the same constant-value rule. Optional
callable fields retain `None`, and enum defaults retain canonical type identity.

## Open decisions

None within the delivered repair scope.

## Dependencies and work packages

AP10/AP24 provide the completed record-construction and unit-interface foundations.

- [x] [AP31.1: Persist evaluated constant defaults](01-persist-constant-defaults.md)

## Acceptance

Equivalent literal, constant-reference, and constant-expression defaults
behave identically before and after independent compilation. Unsupported
defaults produce a useful source diagnostic at the declaring field.
