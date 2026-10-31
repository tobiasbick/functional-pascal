# AP55: Scope-bound resources

Status: complete. Effort: large. The user approved the syntax and implementation.
Completion is tracked in the [central plan](../README.md#package-status).

`using const Name := Acquire();` binds a read-only resource and releases it at
the nearest lexical block exit. A record opts in with a contextual
`release procedure Close(Self: Resource);` member. Native handles use an
explicit compiler catalogue of close operations. See the [contract](contract.md)
for lifetime, failure and escape rules.

## Decisions

| ID | Chosen contract | Work packages |
| --- | --- | --- |
| Q77 | Use `using const` resource locals and record release procedures. Do not add `finally` or `defer`. | AP55.1, AP55.2 |
| Q78 | Apply lexical resource lifetimes throughout programs, named and anonymous routines, generic and imported code, and record routines. | AP55.1, AP55.2 |
| Q79 | Register only a successfully initialized resource; protect acquisition and registration from cancellation. | AP55.1, AP55.2 |
| Q80 | Release in reverse lexical order on normal and structured exits, propagation, cancellation and managed failure. | AP55.1, AP55.2 |
| Q81 | Join nested task scopes in lexical order, mask cancellation during cleanup, then check routine postconditions. | AP55.1, AP55.2 |
| Q82 | Preserve a pending Result error or None and report cleanup failures separately. A cleanup failure replaces success or cancellation; an existing panic remains primary. The first non-cancellation runtime failure stays primary. | AP55.1, AP55.2 |
| Q83 | Debugger frame replacement must not abandon live cleanup obligations; stepping and cancellation retain them. | AP55.1, AP55.2 |

## Work packages

- [x] [AP55.1: Decide the cleanup contract](01-decision.md)
- [x] [AP55.2: Implement and adopt the contract](02-implementation.md)

The implementation and its regressions are in place. No ownership type system
or new public standard unit is part of this package.
