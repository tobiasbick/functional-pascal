# AP41: Canonical qualified names

Status: complete (AP41.1 and AP41.2).
Effort: medium. Completion is tracked in the
[central plan](../README.md).

## Findings

Before AP41, [enum variants](../../../pascal/language/types/enums.md) could use
short or type-qualified names. Adding another enum could invalidate earlier
short-name uses; a same-named type could hide the short variant. Plain
[imports](../../../pascal/program-structure/units.md) exposed short and full
names simultaneously. AP41 removes those alternatives.

## Selected contract

Qualification is mandatory language behavior, enforced by the compiler.

- Every direct import uses `uses Unit.Path as Alias;`. Imported public routines,
  types, constants, and variables are referenced only through `Alias.Name`.
  Remove plain imports, unqualified imported names, and access through an
  aliased unit's original path. The canonical unit path still appears in `uses`.
- User enum variants always carry their type qualifier: `Color.Red` for a local
  type and `Domain.Color.Red` for an imported type. Apply the same rule to
  construction, values, comparisons, and all pattern positions. Neither `Red`
  nor `Domain.Red` is an alternative enum spelling.
- `Some`, `None`, `Ok`, and `Error` retain their existing reserved language forms
  for Option and Result. Introduce no qualified alternatives for them.
- Retain AP05's case-insensitive alias uniqueness, namespace reservations,
  prohibition on alias shadowing, and one direct import per unit. Aliases remain
  local to their source file; interfaces and linked identities remain canonical.
- Editor completion, auto-imports, rename, templates, and authoring guidance
  produce the same required forms. Auto-import defaults to the final unit-name
  segment as its alias, reuses an existing alias, and chooses a readable,
  nonconflicting alias when that default is unavailable.
- Ordinary local names, explicit type aliases, and record/native dot operations
  retain their existing rules. AP06's import-independent operations, AP24/AP34
  generic types and inference, and AP36's positional-or-named field lists remain
  intact. Qualification adds no free-function receiver lookup or new call form.

The full decision and accepted/rejected example matrix are recorded in
[AP41.1](01-qualification-decision.md).

## Replaced decisions and retained contracts

AP41.2 replaces AP05's plain-import allowance and the optional short user-enum
variant names in the current enum/construction/pattern model. AP05's alias
semantics and editor identity remain the foundation; AP20/AP24/AP36/AP39 keep
their construction, binding, inference, field-list, and coverage contracts.
These completed packages retain their status. Language documentation, the
compact reference, authoring guidance, consumers, and editor declarations now
describe and use the delivered required forms.

## Open decisions

None. AP41.2 delivers compiler enforcement, semantic migration, documentation,
diagnostics, tooling, and regression coverage after independent review.

## Dependencies and work packages

AP05/AP20/AP24 are complete. AP41.1 precedes AP41.2. Retain completed import
tooling; do not reintroduce free-function receiver lookup from before AP06.

- [x] [AP41.1: Qualification decision](01-qualification-decision.md)
- [x] [AP41.2: Qualified-name rules](02-qualified-name-rules.md)

## Acceptance

Every import has an explicit alias and every user enum variant has an explicit
type qualifier. Adding a colliding import or variant cannot change already
qualified targets. Built-in Option/Result forms and local/native operations
remain consistent with their existing contracts. Compiler, consumers,
documentation, and editor tools agree before the package is marked complete.
