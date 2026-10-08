# AP14: Remove computed properties

Status: agreed direction. Effort: small. Completion is tracked in the
[central README](../README.md); the process is in
[development-process.md](../development-process.md).

## Goal

Computation and state changes are visible as calls, and no property grammar
remains.

Properties are used only in examples and one formatter golden file at planning
time; no library or app API depends on them.

## Decisions

- `property`, `read`, and `write` are removed without replacement grammar.
- Getters become ordinary instance functions called with parentheses
  (`Cam.Zoom()`); setters become instance procedures.
- Real record fields remain direct data access.

## Dependencies

- AP06 (complete): getters and setters become record methods, which use
  declared-member dot calls.

AP25 depends on this package.

## Order

AP14.1 migrates property uses while properties still work. AP14.2 removes the
grammar.

## Work packages

- [ ] [AP14.1: Migrate properties to methods](01-migrate-properties.md)
- [ ] [AP14.2: Remove property declarations](02-remove-property-declarations.md)

## Acceptance

Computation and state changes are visible as calls, and no property grammar
remains.
