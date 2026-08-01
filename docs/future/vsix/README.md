# VSIX roadmap

Status: planned

This roadmap expands the repository-owned VS Code-compatible extension from
the implemented `0.0.8` baseline. Phases 1 through 11 are complete and remain
documented by current user documentation, tests, and Git history. They are not
reopened here.

The roadmap is intentionally split into independently installable increments.
Each phase must end with a tested VSIX that remains useful even if no later
phase is implemented.

## Current baseline

The existing `0.0.8` extension provides:

- `.fpas` language detection, TextMate highlighting, indentation, comments,
  brackets, and folding markers
- parser and semantic diagnostics for open buffers
- canonical whole-document formatting
- document symbols, hover, definition, references, and rename
- workspace symbols, document highlights, type definition, and selection ranges
- a bounded, watched project catalog with open-order-independent references
- rich visibility-aware completion with lazy documentation
- signature help, checked FPAS snippets, and safe unambiguous auto-imports
- compiler-backed semantic highlighting with a TextMate fallback
- deterministic `uses` import quick fixes for eligible `F2001` and `F2003`
- a bundled host-native language server and source standard library
- local packaging and Extension Host/package smoke tests

Current behavior is documented in
[`docs/pascal/tools/editor-integration.md`](../../pascal/tools/editor-integration.md).
This roadmap must not be treated as implemented behavior.

Latest completed increment: Phase 11 shipped locally as extension `0.0.8` in
`editors/vscode/dist/functional-pascal-0.0.8-win32-x64.vsix` (Windows x64,
2,666,347 bytes, SHA-256
`9FFAD7AACDF0912170497C08BE8729FDD852D9A31F9E0F2524B1D5D59BBF772D`).

## Phase status

| Phase | Status | Outcome |
|------:|--------|---------|
| [12](phase-12-project-workflow.md) | Planned | Check, build, run, and test workflows in the editor |

Items that are not phase commitments are kept in [`backlog.md`](backlog.md).

## Execution rules

1. Implement phases in order unless the plan is explicitly revised first.
2. Mark exactly one phase as `In progress` in this table and its phase file.
3. Keep changes inside the phase scope; move discovered follow-ups to the
   backlog or a later phase.
4. Reuse the compiler, formatter, project loader, and language-service logic.
   The TypeScript extension remains transport and editor integration code.
5. Do not change Functional Pascal syntax or semantics as editor work.
6. Use the next unused extension patch version. Record the actual version and
   artifact name in the phase file when packaging succeeds.
7. After completion, move implemented behavior into current documentation and
   remove the completed phase file once tests and links make it obsolete.

## Definition of done for every phase

- The phase acceptance criteria are checked and recorded.
- Positive, negative, stale-state, and relevant UTF-16/path edge cases have
  regression coverage in the owning Rust crate.
- Protocol conversion has LSP-level tests.
- User-visible behavior has a real VS Code Extension Host test when practical.
- `cargo fmt --all -- --check`, `cargo build`, and `cargo test --workspace` pass.
- `npm test --prefix editors/vscode` passes.
- `npm run package --prefix editors/vscode` produces and verifies a host-native
  VSIX, including a smoke test against the server extracted from the archive.
- Current documentation describes only the behavior that now exists.
- The artifact path, target, version, size, and SHA-256 are recorded before the
  phase is closed.

## Deliberate constraints

- No Marketplace/Open VSX publication or release automation.
- No CI or platform build matrix; users build on the host they use.
- Local desktop editors remain the supported environment during these phases.
- No telemetry or automatic report submission.
