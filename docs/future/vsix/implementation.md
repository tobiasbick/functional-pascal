# VSIX implementation plan

## Status tracking

| Phase | Status | Deliverable |
|------:|:------:|-------------|
| 0 | open | immediately installable Hello World VSIX |
| 1 | open | confirmed protocol, package, and fixture contracts |
| 2 | open | syntax-only development extension |
| 3 | open | language-service foundation |
| 4 | open | functioning stdio language server |
| 5 | open | diagnostics and formatting |
| 6 | open | symbols, hover, definitions, and completion |
| 7 | open | reproducible final VSIX packaging |
| 8 | open | local host acceptance and current documentation |

Work phases in order. Update the table and verification notes as each phase is
completed. Do not mark a phase complete while one of its acceptance checks is
open.

## Phase 0 — immediately build a real VSIX

This is the first implementation work. Do not begin the LSP architecture or
syntax grammar until the bootstrap VSIX exists and has been installed.

### Files

Create only the minimum extension and packaging slice:

```text
editors/vscode/
  package.json
  package-lock.json
  tsconfig.json
  .vscodeignore
  README.md
  src/
    extension.ts
  test/
    extension.test.ts
  dist/
    functional-pascal-0.0.1-bootstrap.vsix
```

Generated JavaScript, `node_modules/`, and `dist/` are ignored. Do not add the
later grammar, Rust crates, language client, or server placeholder in this
phase.

### Work

- Choose a conservative stable `engines.vscode` value for the APIs used by the
  bootstrap extension.
- Check that the local Node.js version satisfies the pinned `@vscode/vsce`
  requirement before installing dependencies.
- Create a TypeScript extension using the official VS Code extension API.
- Pin `typescript`, `@types/vscode`, `@vscode/vsce`, `@vscode/test-cli`, and
  `@vscode/test-electron` in `package-lock.json`.
- Use `functional-pascal.functional-pascal` as the stable local extension ID;
  the publisher name in the manifest does not imply Marketplace registration.
- Contribute **Functional Pascal: Show Output** with command ID
  `functionalPascal.showOutput`.
- On activation, create one output channel named `Functional Pascal` and append
  exactly:

  ```text
  Functional Pascal extension activated (Hello World).
  ```

- The command reveals the output channel; activation itself does not display a
  notification or steal editor focus.
- Dispose the command and output channel through the extension context.
- Add an npm `package` script that compiles, tests, and runs local packaging:

  ```text
  npx @vscode/vsce package --out dist/functional-pascal-0.0.1-bootstrap.vsix
  ```

- Do not run `vsce publish`, create a publisher account, or add marketplace
  metadata that local packaging does not require.

### Tests

- Extension-host test activates the extension and executes
  `functionalPascal.showOutput`.
- Manifest test checks the command ID, entry point, `engines.vscode`, and local
  package script.
- Package-content check opens the VSIX as an archive and confirms the compiled
  entry point, manifest, README, and license are present.
- Negative package check confirms source maps, tests, `node_modules`, local
  paths, and unrelated repository files are absent.

### Acceptance

Run from the repository root:

```text
npm ci --prefix editors/vscode
npm run package --prefix editors/vscode
```

The file below must exist and be non-empty:

```text
editors/vscode/dist/functional-pascal-0.0.1-bootstrap.vsix
```

Install this file through **Extensions: Install from VSIX** in one locally
available VS Code-compatible desktop editor:

1. Run **Functional Pascal: Show Output**.
2. Confirm the `Functional Pascal` output channel opens.
3. Confirm it contains the exact Hello World activation line.
4. Restart the editor and repeat the command.
5. Uninstall the extension cleanly.

Testing an additional installed clone is useful but not required. Record only
the editor version and pass/fail result. When these checks pass, retain the
working package path and proceed to Phase 1.

Primary guidance for this phase is collected in
[SDK and documentation references](references.md), especially Microsoft's
Hello World, extension anatomy, testing, and `vsce` packaging documentation.

## Phase 1 — contracts and fixtures

### Work

- Add representative `.fpas` fixtures covering programs, units, records,
  functions, comments, strings, Unicode identifiers/text, malformed syntax,
  and a small multi-project workspace.
- Define the initial LSP capability set and full-document synchronization.
- Compare `tower-lsp-server` and rust-analyzer's `lsp-server` against the
  criteria in [SDK and documentation references](references.md), select one,
  and record the rejected option with a short reason.
- Confirm that the current parser, semantic analyzer, formatter, diagnostics,
  and project crates expose the data needed by the first capabilities.

### Acceptance

- Every requested feature maps to an existing authoritative FPAS crate or to a
  named language-service query.
- No language syntax or semantic change is required.
- Unsupported host targets and remote-host behavior fail with actionable
  messages.

## Phase 2 — extension shell and syntax highlighting

### Files

Extend the working `editors/vscode/` bootstrap with the language configuration,
TextMate grammar, and fixtures described in the overview. Preserve the
installable package path established in Phase 0.

### Work

- Register `.fpas` as the `fpas` language.
- Add line comments, block comments, brackets, auto-closing pairs, surrounding
  pairs, indentation, and folding markers.
- Implement a TextMate grammar using repository examples and tests as fixtures.
- Keep extension activation minimal; no compiler logic is written in
  TypeScript.
- Pin Node development dependencies in `package-lock.json`.

### Tests

- Positive highlighting fixtures for declarations, types, routines, control
  flow, literals, comments, operators, and qualified names.
- Negative fixtures proving keywords inside strings/comments and keyword-like
  identifier substrings are not highlighted as keywords.
- Edge fixtures for escaped Pascal strings, nested-looking comments, Unicode,
  and an incomplete file.
- Manifest validation that `.fpas`, the grammar, and language configuration
  reference existing packaged files.

### Acceptance

- The development extension opens an `.fpas` fixture with useful highlighting
  even when no server exists.
- VS Code's extension host reports no manifest or grammar errors.

## Phase 3 — language-service foundation

### Files

Create `crates/fpas-language-service/` with focused document, workspace,
diagnostic, formatting, and symbol modules. Keep files below the repository's
structural thresholds; split symbol queries by responsibility.

### Work

- Implement immutable versioned document snapshots and a reusable line index.
- Load loose files and project/workspace context through `fpas-project`.
- Overlay open buffers on disk-backed project sources.
- Expose editor-oriented results without LSP types.
- Cache parse and analysis results by source version.
- Make absent or invalid project metadata a recoverable state.

### Tests

- Positive: valid loose file and project file produce snapshots and analysis.
- Negative: invalid manifest, missing dependency, and malformed source return
  structured failures without panic.
- Edge: empty file, CRLF, Unicode, unsaved overlay, file deletion, reopen with a
  newer version, and two units with the same unqualified symbol.

### Acceptance

- The crate has no dependency on VS Code, Node, or LSP protocol types.
- No editor query writes compiled-unit sidecars or executes FPAS code.

## Phase 4 — LSP transport and lifecycle

### Files

Create `crates/fpas-lsp/` as its own binary crate. Use a standard Rust LSP
protocol implementation over stdio and keep protocol conversion isolated from
request handling.

### Work

- Implement initialize, initialized, shutdown, and exit.
- Advertise only implemented capabilities.
- Handle didOpen, didChange, didSave, and didClose using full text sync.
- Convert file URIs and UTF-16 LSP positions safely.
- Send logs to stderr and keep stdout protocol-only.
- Add cancellation and malformed-request handling where supported by the
  selected protocol library.
- Connect the TypeScript language client to a development server binary.

### Tests

- Transcript tests for a valid initialize/open/shutdown lifecycle.
- Negative tests for requests before initialization, malformed parameters,
  unsupported URI schemes, and stale document versions.
- Position conversion tests for ASCII, CRLF, BMP Unicode, surrogate pairs,
  end-of-line, end-of-file, and out-of-range positions.
- Process test proving stdout contains valid framed LSP messages only.

### Acceptance

- The server starts and shuts down cleanly from the extension.
- Closing or restarting the editor leaves no orphan server process.
- Invalid client input does not panic the server.

## Phase 5 — diagnostics and formatting

### Work

- Publish parser diagnostics for every open/change analysis.
- Publish semantic diagnostics when the syntax is sufficiently valid.
- Preserve stable FPAS diagnostic codes, severity, ranges, and useful help.
- Debounce analysis during typing while never publishing a result for an older
  document version.
- Clear stale diagnostics on fix and close.
- Implement `textDocument/formatting` through `fpas-fmt` using the unsaved
  buffer and a whole-document edit.

### Tests

- Positive diagnostics for known parser and semantic errors.
- Negative case where a corrected document clears every previous diagnostic.
- Edge cases for multiple diagnostics, warning/error severity, dependency
  source ranges, rapid version changes, and close during analysis.
- Formatting parity test comparing the LSP edit result with `fpas fmt`.
- Formatting idempotence and comment-preservation tests.
- Malformed input test proving formatting returns no destructive edit.

### Documentation checkpoint

Once the behavior exists, update:

- `README.md`, which currently says there is no format-on-save
- `docs/pascal/tools/fmt-style.md`, which currently says there is no LSP
- a new current editor-integration page under `docs/pascal/tools/`
- relevant documentation indexes linking the new current page

Current documentation must describe only features verified in this phase.

### Acceptance

- An unsaved syntax error appears and disappears in the editor as expected.
- Format Document produces exactly the canonical formatter output.
- Enabling the editor's standard format-on-save setting requires no FPAS
  watcher or extra extension setting.

## Phase 6 — language navigation

### Work

- Implement document symbols with correct full and selection ranges.
- Implement hover for declarations and resolvable references.
- Implement go to definition within a file and across loaded project units.
- Implement basic completion using visible declarations and the current
  project context.
- Respect current visibility, exports, shadowing, and qualified-name behavior.
- Split `fpas-sema/src/interface.rs` only where a concrete query needs a
  focused reusable API; keep any such refactor behavior-neutral.

### Tests

- Positive same-file and cross-unit definition queries.
- Positive symbols for programs, units, types, routines, variables, parameters,
  and record members.
- Negative queries for comments, strings, unknown names, private declarations,
  and files outside the loaded project.
- Edge cases for shadowing, overload-like candidates supported by current FPAS,
  qualified names, incomplete member access, Unicode positions, and unsaved
  declaration changes.
- Completion tests assert stable relevant entries, not presentation ordering
  that the editor controls.

### Acceptance

- Navigation results agree with current compiler name resolution.
- Partial or invalid source returns an empty/partial result rather than a
  server failure.

## Phase 7 — deterministic final packaging

### Work

- Add `@vscode/vsce` and `vscode-languageclient` as pinned project dependencies.
- Add a cross-platform Node entry point at
  `editors/vscode/scripts/package.mjs`; do not add one script per operating
  system.
- Map the current desktop host and architecture to a supported VS Code target
  such as `win32-x64`, `linux-x64`, or `darwin-arm64`. Fail clearly for an
  unmapped target.
- Build the native server on the current host with:

  ```text
  cargo build --release -p fpas-lsp
  ```

- Do not set a Rust cross-compilation target. Stage only the current host's
  `fpas-lsp` or `fpas-lsp.exe` under
  `editors/vscode/server/<host-target>/`.
- Remove stale generated server directories before staging so a VSIX cannot
  accidentally contain binaries from earlier builds on another host.
- Compile and test the TypeScript extension.
- Package with the repository license and a narrow `.vscodeignore`.
- Invoke `vsce package --target <host-target>` and include the target in the
  output filename.
- Inspect the resulting archive contents after packaging.
- Ignore staged binaries, generated JavaScript, `node_modules/`, and `dist/`.
- Make repeated clean builds replace the same versioned output deterministically
  except for metadata inherently produced by the packaging tool.

### Package tests

- Fail if the expected host-native `fpas-lsp[.exe]`, grammar, language
  configuration, license, or JavaScript entry point is absent.
- Fail if a server directory for any other host target enters the archive.
- Fail if source maps, tests, `node_modules`, Cargo target data, machine paths,
  or unrelated executables enter the archive.
- Extract the VSIX to a temporary directory and start the packaged server for
  an initialize/shutdown smoke test.
- Confirm the extension never falls back to a globally installed server.

### Acceptance

From the repository root, one documented command:

```text
npm run package --prefix editors/vscode
```

produces:

```text
editors/vscode/dist/functional-pascal-<version>-<host-target>.vsix
```

No marketplace login, network publication, or manual archive modification is
needed. The build produces only the current host target. A user on a different
system builds there.

## Phase 8 — editor acceptance and completion

### Automated verification

Run at minimum:

```text
cargo fmt
cargo build
cargo test --workspace
npm test --prefix editors/vscode
npm run package --prefix editors/vscode
```

Also run the packaged-server transcript test against the executable extracted
from the final VSIX.

### Local smoke test

Install the host-native VSIX using **Extensions: Install from VSIX** in one
locally available VS Code-compatible desktop editor and verify:

- installation and activation succeed
- `.fpas` highlighting works
- the bundled server starts
- diagnostics update and clear
- Format Document works
- hover and definition work
- the restart command works
- uninstall removes the extension cleanly

If Cursor, VSCodium, or another clone is already installed, repeating the smoke
test there is useful but optional. Do not create virtual machines, CI jobs, or
a platform/editor matrix for acceptance. Record the product version and result,
but not usernames, hostnames, home paths, or other machine-identifying
metadata.

### Final cleanup

- Ensure current behavior is documented under `docs/pascal/`, not only here.
- Remove obsolete statements that editor integration does not exist.
- Update the root README with local build and installation instructions.
- Confirm docs links and Rust doc links resolve.
- Confirm no generated binary or VSIX was staged unintentionally unless the
  user explicitly asks to version the artifact.
- Remove this future plan once implemented behavior, tests, and current
  documentation fully replace it.

## Final deliverable

Implementation ends with a tested, locally built file:

```text
editors/vscode/dist/functional-pascal-<version>-<host-target>.vsix
```

It is the single installation artifact for a VS Code-compatible desktop editor
on the host where it was built. Users on another operating system or
architecture build their own host-native VSIX from the same repository.
