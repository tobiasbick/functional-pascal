# Phase 08: project index and invalidation

Status: planned

## Goal

Make project-wide analysis complete and predictable before adding more editor
features. References must not depend on which source happened to be opened
first, and external file or manifest changes must not leave stale results.

## Scope

- Build a bounded catalog of `.fpasprj` and `.fpasworkspace` manifests inside
  the opened editor folder.
- Enumerate manifests rather than recursively treating every `.fpas` file as a
  loose source. Authoritative project loading remains in `fpas-project`.
- Index direct ownership, project/workspace dependencies, library exports, and
  reverse consumer relationships.
- Include every indexed consumer in references and rename when the declaration
  is visible through the project/export graph.
- Watch indexed manifests and source files for create, change, rename, and
  delete events; invalidate only affected project analysis and navigation data.
- Preserve unsaved editor snapshots over disk content.
- Keep dependency declarations outside the opened folder readable for
  navigation but non-editable by rename.
- Add cancellation checks to project indexing and long reference scans.

## Out of scope

- A persistent on-disk database.
- Scanning arbitrary package caches or the entire machine.
- Changing compiler project or visibility rules.
- Multi-root editor workspaces; they remain a backlog item until one root is
  correct and measurable.

## Expected ownership

```text
crates/fpas-language-service/src/
  workspace/       — manifest catalog, ownership, dependency graph
  analysis/        — targeted invalidation of cached project analysis
  navigation/      — reverse-consumer query scope
crates/fpas-lsp/src/
  server/          — watched-file notifications and cancellation plumbing
editors/vscode/src/
  extension.ts     — watcher registration only when client support is needed
```

Keep each concern in a focused module; do not grow a generic workspace or
navigation utility file.

## Acceptance criteria

- [ ] `NotesUpdate` references include `apps/notes/src/notes.fpas` and the indexed
  Notes test project without opening those consumer sources first.
- [ ] Opening a library unit before or after a consuming program produces the same
  reference results.
- [ ] Changing a `.fpasprj` dependency or `[exports].units` entry updates
  navigation without restarting the language server.
- [ ] Creating or deleting a project source updates diagnostics and references.
- [ ] A same-named symbol in an unrelated project is not included.
- [ ] A cancelled scan produces no stale partial result or server failure.
- [ ] Rename never edits a declaration outside the opened editor folder.

## Required tests

- [ ] Real-repository regression for `NotesUpdate` across both Notes manifests and
  `tests/suite.fpasprj`.
- [ ] Temporary-project tests for source creation, deletion, dependency removal,
  export removal, ambiguous ownership, and equal symbol names.
- [ ] LSP watched-file and cancellation transcripts.
- [ ] Extension Host test that changes a manifest or source externally and observes
  refreshed results.

## Progress record

- Started: not started.
- Completed: not completed.
- Extension version: next unused patch version.
- Artifact: not built.
- SHA-256: not recorded.
- Verification: not run.
