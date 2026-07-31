# Phase 12: project workflow

Status: planned; depends on Phases 08 through 11.

## Goal

Make normal project work possible from a VS Code-compatible editor without
turning the extension into a second build system.

## Scope

- Provide commands for check, build, run, test, format, and format check.
- Reuse the authoritative `fpas` CLI and its documented non-interactive output
  contracts; do not reimplement project execution in TypeScript.
- Decide during implementation whether the host-built VSIX bundles `fpas` next
  to `fpas-lsp` or locates an explicitly configured executable. Prefer one
  deterministic default with an actionable error when unavailable.
- Resolve the active `.fpasprj` or `.fpasworkspace` through the same project
  context as the language server.
- Run interactive programs in an editor terminal.
- Surface check/build/test diagnostics in the Problems panel with file ranges
  and stable diagnostic codes.
- Integrate `*_test.fpas` cases with the VS Code Testing API using
  `fpas test --report json`; support run, rerun, and filtered run.
- Add a compact status item that shows the selected project and active
  operation without duplicating editor notifications.

## Out of scope

- A debugger or Debug Adapter Protocol implementation.
- A package manager, registry, or dependency UI.
- CI, publishing, cross-compilation, or a platform release matrix.
- Replacing terminal interaction for TUI or graphical applications.

## Expected ownership

```text
editors/vscode/src/workflow/
  project.ts        — active-project selection and status
  commands.ts       — command registration and argument validation
  processes.ts      — CLI invocation, cancellation, and output routing
  tests.ts          — Testing API discovery and execution
editors/vscode/scripts/package/
  ...               — optional authoritative CLI staging and verification
```

Rust changes should be limited to missing structured CLI contracts and their
own tests; the extension must call existing commands where they already fit.

## Acceptance criteria

- [ ] Check, build, format check, and tests run non-interactively and can be
  cancelled without orphaning child processes.
- [ ] Run opens an interactive terminal and forwards program arguments explicitly.
- [ ] Multiple projects require an explicit, remembered selection rather than an
  arbitrary first manifest.
- [ ] Problems use the real source URI, range, severity, code, and help text.
- [ ] The Testing view discovers project tests, runs one or all tests, and reports
  pass, fail, skip, timeout, and runtime error distinctly.
- [ ] A missing or incompatible CLI reports one actionable message and does not
  disable language features.
- [ ] The packaged workflow works in VS Code and a compatible clone on the host
  used to build the VSIX.

## Required tests

- [ ] TypeScript unit tests for executable discovery, argument construction,
  project selection, cancellation, and structured output parsing.
- [ ] Extension Host tests for command registration, Problems integration, and
  Testing API discovery/run on fixtures.
- [ ] Package smoke tests execute the staged CLI rather than a developer PATH copy.
- [ ] Negative tests cover missing executable, invalid project, failed build,
  failed test, timeout, cancellation, and paths containing spaces.

## Progress record

- Started: not started.
- Completed: not completed.
- Extension version: next unused patch version.
- Artifact: not built.
- SHA-256: not recorded.
- Verification: not run.
