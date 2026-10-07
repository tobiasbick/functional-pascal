# AP01 plan spelling review

This records the review required by [AP01.2](02-plan-spelling-review.md).
The [reference style](reference-style.md) supplies the ten canonical draft
examples. The review covers the central README, shared constraints, development
process, and every package README and work package under this plan.

## Rules checked

| Concern | Reference and result |
|---------|----------------------|
| Type applications | AP24 uses `of`, no parentheses for one argument, parentheses for several, and the retained `dict of K to V` form. No positive draft example applies a type with angle brackets. |
| Routine type parameters | The retained `function Identity<T>(...)` declaration is linked to AP24. It is a declaration, not a type application or explicit call-site instantiation. |
| Parameters and calls | AP08 keeps individually annotated parameters separated by `;`; call arguments use `,`. AP09's named form remains fully named, using `:=`. |
| Block endings | AP13 keeps program `end.`, unit `end unit;`, named declaration/control-flow closers, plain scoped `end;`, and `repeat ... until Condition;`. Expression closers have no separate terminator. |
| Bindings and mutation | AP16's implemented examples prefer annotated `const`, using `var` for reassignment. AP17 retains explicit call-site `var` for written arguments and the agreed native-receiver exception, with its requested follow-up. |
| Record construction | AP10's target construction is named-field-only; enum constructors follow AP09, while patterns remain positional. |
| Pattern bindings | AP20 uses `const Name` and `_` for payload fields; AP03's closed-enum coverage has no `else` escape. |
| Recorded exceptions | AP19's `distinct` (Q12), AP13's named boundaries (Q08/Q09), and AP23's `requires`/`ensures` (Q15/Q16) remain linked to their recorded decisions. |
| Draft versus current syntax | The reference identifies completed AP11 declarations, AP13 endings, branches and no-op statements, AP16 bindings, AP09 named calls, and AP17.1 positional `var` parameters as current forms. Unimplemented named `var` arguments, construction, patterns, and domain types retain their owning package and draft status. Superseded forms in inventories, migration instructions, diagnostics, and reference-branch notes remain identified as such. |

## Corrections

- [AP09](../ap09-named-arguments/README.md) no longer calls `F(3);` valid while
  silently dropping its return value. The positional and rejected named-call
  examples now consume the result, consistent with
  [AP04](../ap04-discarded-function-values/README.md). The callable binding uses
  `const`, as required by AP16's teaching convention.
- [AP22](../ap22-limited-local-inference/README.md) identifies its unannotated
  binding examples as local declarations inside a routine or scoping block.
  It does not imply inference for unit-level declarations.
- [AP01](README.md) links the routine type-parameter spelling exception to
  AP24. Its review verification distinguishes positive draft syntax from
  deliberately rejected forms and legacy migration input.
- The [central README](../README.md#document-map) links the reference style.
  Its reference-branch note attributes alias-only imports and removal of
  record methods to the earlier branch, rather than describing them as the
  current AP05/AP06 decisions.

## Open decisions retained

AP01 needs no additional language decision. The remaining decision gates and
requested follow-up discussions belong to their owning packages; the reference
examples do not settle them.

| Package | Remaining decision, specification, or follow-up |
|---------|-------------------------------------|
| [AP06](../ap06-dot-call-targets/README.md#open-decisions), [AP12](../ap12-callable-expressions/README.md#open-decisions) | Settle extended catalog names, type-qualified factories, and conflicts; native type operations and automatic availability are agreed. Revisit the agreed implicit-receiver exception with the user before AP06.3/AP17.3. Accept or reject callable-expression targets and their evaluation order. |
| [AP17](../ap17-visible-caller-mutation/README.md#follow-up-discussion), [AP17.1](../ap17-visible-caller-mutation/01-var-parameters.md#implementation) | Function types allow `var` parameters and writes completed before failure remain visible. Revisit the agreed unmarked implicit-receiver exception with the user before AP06.3/AP17.3. |
| [AP19](../ap19-distinct-domain-types/README.md#open-decisions) | Define constraints, dictionary keys, and case labels for distinct types. |
| [AP22](../ap22-limited-local-inference/README.md#open-decisions) | Decide the eligible literal forms and whether enum constructors qualify for local inference. |
| [AP20](../ap20-nested-patterns-and-explicit-bindings/README.md#open-decisions) | Specify the migration of the existing scalar guard-binding form under the explicit binding rule. |
| [AP21](../ap21-decision-expressions/README.md#open-decisions), [AP23](../ap23-preconditions-and-postconditions/README.md#open-decisions) | Specify compatible branch types and whether error returns from `try` run postconditions. |
| [AP25](../ap25-conservative-purity/README.md#open-decisions), [AP26](../ap26-structured-task-scopes/README.md#open-decisions), [AP27](../ap27-typed-placeholders/README.md#open-decisions) | Reassess purity after contract use and specify shared-handle aliasing; specify scoped closure-capture limits; select placeholder spelling only if the optional package proceeds. |

The AP02 numbering gate is resolved: the user confirmed Q01's
`FPnxxx` scheme. See [AP02's resolved decisions](../ap02-structured-diagnostics/README.md#resolved-decisions).

AP13's JSON variant name is confirmed as `JsonValue.NullValue`. See
[AP13's reserved keyword names](../ap13-explicit-block-boundaries/README.md#reserved-keyword-names).

## Delivery and integration

The reference examples and the spelling corrections are delivered on the
working branch. Current-language documentation, grammar, source code, and the
formatter are unchanged by AP01. Runtime tests and builds do not validate these
uncompiled planning drafts; the applicable checks are spelling review,
relative links and anchors, Markdown fences, and whitespace.

The user's selected working branch replaces the process's default branch per
work package for this delivery. Both work packages are complete under
[status tracking](../development-process.md#status-tracking); their checkboxes
describe completed delivery on that branch.
