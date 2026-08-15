# Consciously deferred

This package implements only task-owned construction of one named nested
routine from exact existing mutable capture cells. The following remain outside
its boundary even when nearby code makes them appear easy.

## Remaining `DBG-D03`

- Constructing anonymous `function` or `procedure` expressions in debugger
  evaluation.
- Copying an arbitrary already materialized task-bound function value from a
  visible binding or aggregate child.
- Assigning task-bound functions into globals, closure-cell roots, record or
  collection descendants, active/inactive payloads, or Dynamic storage.
- Migrating, transferring, clearing, or weakening a function's runtime task
  owner.
- Constructing a new mutable cell, copying a cell as a user-visible value, or
  editing a callable's capture slots in place.
- Accepting capture graphs that contain task handles, opaque resources, Dynamic
  endpoints, or nested task-bound functions as immutable value captures.
- Bound-receiver synthesis and newly materialized instance-method values.
- In-place changes to `FunctionId`, code, signature, source mapping, capture
  metadata, task-bound flag, or task owner.
- Inferring callable or cell identity from names, arity, rendered values,
  `<cell>`, `<function ...>`, or debugger handle order.
- Reviving stale frames, task IDs, inspection generations, variable references,
  or capture metadata.

## Other backlog packages

`DBG-D04` through `DBG-D09` remain unchanged: broader control-flow mutation,
task control/history, interactive hosted programs, attach/remote/native
debugging, advanced breakpoints, and reverse execution/hot reload are not part
of this plan.

Any later package must define its own stable IDs, ownership and lifetime proof,
portable metadata boundary, atomicity rules, protocol parity, resource limits,
negative coverage, documentation reconciliation, and resume log.
