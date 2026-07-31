# VSIX backlog

This file holds possible follow-up work that is not committed to Phases 08
through 12. Promote an item into a new numbered phase only after its value,
scope, and prerequisites are agreed.

## Language navigation and presentation

- Call hierarchy and implementation hierarchy.
- Inlay hints for inferred types, parameter names, and generic arguments.
- CodeLens for references, tests, or runnable entry points.
- Document links for project paths and explicitly referenced source files.
- Server-provided folding ranges beyond declarative region markers.
- Range formatting and on-type formatting; canonical whole-document formatting
  may remain the more predictable FPAS contract.
- Manifest-aware program and unit rename, including source-file and `uses`
  updates.

## Workspace and execution

- Multi-root workspaces with isolated project indexes and standard libraries.
- Remote SSH, WSL, container, and virtual-workspace support.
- Project creation, manifest editing, and dependency-management UI.
- Persistent symbol indexes for very large repositories, if measurements show
  that the in-memory Phase 08 index is insufficient.
- Coverage display and richer test result attachments.

## Debugging

- A Debug Adapter Protocol server.
- Breakpoints, stepping, stack frames, scopes, variable inspection, and
  expression evaluation.
- Source mapping for compiled program images.

Debugging is a separate compiler/runtime project, not a small extension feature.
It requires explicit design and approval before implementation.

## Distribution and presentation

- Marketplace or Open VSX publication.
- Automated release builds, signing, update channels, or a platform matrix.
- Web extension support.
- A custom icon theme, walkthrough, welcome page, or other promotional UI.
- Telemetry or automatic crash reporting.

The hobby project currently keeps distribution local: build the VSIX on the
host where it will be installed.
