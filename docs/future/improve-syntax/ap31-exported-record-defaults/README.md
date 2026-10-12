# AP31: Exported record constant defaults

Status: Decision pending (repair plan recorded; implementation not started).
Effort: small to medium. Completion is tracked in the [central plan](../README.md).

## Finding

The [record specification](../../../pascal/language/types/records.md#default-field-values)
allows exported scalar constant expressions. An exported default `:= 3`
works, but `:= 1 + 2` and a reference to a preceding scalar constant fail with
FP9001 and a misleading dependency-sidecar rebuild hint. The same expression
works in a program-local record. The interface exporter serializes literal
syntax rather than all supported constant values.

## Proposed scope

Honor the existing constant-expression contract and report unsupported
defaults at their source field. Preserve declaration scope, aliases, and
optional `None` defaults. This is a bounded implementation/spec repair, not
permission to export arbitrary computed defaults.

## Open decisions

Confirm the repair scope before selecting implementation. Making dynamic
defaults portable, restricting all local defaults, or exporting private
implementation types requires a separate decision and is outside this repair.

## Dependencies and work packages

AP10/AP24 provide record construction and unit interfaces. No new language
decision is needed to fulfill the existing scalar constant-expression rule.

- [ ] [AP31.1: Persist evaluated constant defaults](01-persist-constant-defaults.md)

## Acceptance

Equivalent literal, constant-reference, and constant-expression defaults
behave identically before and after independent compilation. Unsupported
defaults produce a useful source diagnostic rather than FP9001 at the importer.
