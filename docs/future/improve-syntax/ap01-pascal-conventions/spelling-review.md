# AP01: Plan spelling rules

Status: complete. These rules apply to every package and work package in the
[syntax plan](../README.md). The [reference style](reference-style.md) supplies
examples of implemented forms and explicitly identified planned forms.

## Spelling rules

| Concern | Required form |
|---------|----------------------|
| Type applications | AP24 uses `of`, no parentheses for one argument, parentheses for several, and the retained `dict of K to V` form. No positive draft example applies a type with angle brackets. |
| Routine type parameters | The retained `function Identity<T>(...)` declaration is linked to AP24. It is a declaration, not a type application or explicit call-site instantiation. |
| Parameters and calls | AP08 keeps individually annotated parameters separated by `;`; call arguments use `,`. AP09's named form remains fully named, using `:=`. |
| Block endings | AP13 keeps program `end.`, unit `end unit;`, named declaration/control-flow closers, plain scoped `end;`, and `repeat ... until Condition;`. Expression closers have no separate terminator. |
| Bindings and mutation | AP16's implemented examples prefer annotated `const`, using `var` for reassignment. AP17 retains explicit call-site `var` for written arguments and the confirmed native-receiver exception: ordinary `Push`/`Pop` dot calls require a writable array. |
| Record construction | AP10's implemented construction is named-field-only, with no record literal; enum constructors follow AP09, while patterns remain positional. |
| Pattern bindings | AP20 uses `const Name` and `_` for payload fields; AP03's closed-enum coverage has no `else` escape. |
| Recorded exceptions | AP19's `distinct` (Q12), AP13's named boundaries (Q08/Q09), and AP23's `requires`/`ensures` (Q15/Q16) remain linked to their recorded decisions. |
| Current versus planned syntax | Completed APs describe implemented behavior. Open AP examples identify their owning decision and remain drafts. |

## Unresolved language rules

Open decisions stay in their owning package. A spelling convention does not
approve construction, inference, patterns, domain types, contracts or task-scope
semantics that an open AP still requires the user to decide.
