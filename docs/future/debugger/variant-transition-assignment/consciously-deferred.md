# Consciously deferred

The following capabilities remain outside variant transition assignment.

- Accepting or rebinding any expired `variablesReference`, child handle, frame,
  task, or snapshot generation.
- Switching a variant through an old payload-child handle.
- Constructing a multi-field payload one field at a time or retaining hidden
  payload values from an earlier runtime variant.
- Choosing an inactive enum variant from an unqualified field name, source
  order, or a best-effort match.
- Synthesizing field defaults or exposing partially initialized enum, `Result`,
  or `Option` values.
- Creating a fieldless variant through a descendant target; use complete-root
  constructor replacement instead.
- Creating the outer record, collection, enum, wrapper, capture cell, or other
  storage when its root is uninitialized or unavailable.
- Advertising inactive variants as virtual Variables children or adding custom
  VS Code controls for variant selection.
- Assigning function values, task handles, cells, or opaque hosted resources.
- Forcing return values, changing the instruction pointer, restarting frames,
  or adding mutation breakpoint actions.

Each future package must define its own stable IDs, ownership and lifetime
rules, atomicity boundary, protocol parity, negative coverage, and resume log.
