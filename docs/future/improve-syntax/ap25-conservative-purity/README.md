# AP25: Conservative purity

Status: rejected and closed after the AP25.1 reassessment; removed from the
active plan. Completion is tracked in the [central README](../README.md).

## Decision (Q18, reassessed)

Do not implement conservative purity. The user accepted closure after the
[assessment and geometry pilot](01-priority-reassessment.md#assessment).
This supersedes Q18's earlier decision to retain the package at low priority.

The demonstrated benefit is reuse of a few predicates in contracts. Existing
AP23 rules already support useful constructor invariants and reject unsafe
calls and mutable-state reads. No concrete production defect was identified
that a purity checker would prevent. Checking transitive calls, callable types,
captures, shared handles, imported interfaces, and standard-library metadata
would impose substantial implementation and ongoing maintenance costs.

Do not add `pure function`, purity annotations, or purity metadata under this
plan. AP23's implemented contract restrictions remain unchanged. AP25.2 and
AP25.3 are removed without implementation; their removal does not claim that
their proposed behavior was delivered.

## Decision work package

- [x] [AP25.1: Priority reassessment](01-priority-reassessment.md)

The assessment is retained as the rationale for closure. Reopening requires
concrete demand from application or library contracts and a new explicit user
decision; it is not a scheduled follow-up. An effect system remains excluded
from the [syntax plan](../README.md#excluded-and-deferred).
