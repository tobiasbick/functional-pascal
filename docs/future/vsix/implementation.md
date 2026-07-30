# VSIX implementation plan

## Status tracking

| Phase | Status | Deliverable |
|------:|:------:|-------------|
| 0 | complete (2026-07-29) | installable Hello World VSIX |
| 1 | complete (2026-07-29) | confirmed protocol, package, and fixture contracts |
| 2 | complete (2026-07-29) | syntax-only development extension |
| 3 | complete (2026-07-29) | language-service foundation |
| 4 | complete (2026-07-29) | functioning stdio language server |
| 5 | complete (2026-07-30) | diagnostics and formatting |
| 6 | complete (2026-07-30) | symbols, hover, definitions, and completion |
| 7 | complete (2026-07-30) | reproducible final VSIX packaging |
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
- Pin `typescript`, `@types/vscode`, `@vscode/vsce`, and
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

### Verification — 2026-07-29

- `npm ci` completed with a clean audit (`0` known vulnerabilities).
- `npm test` compiled the TypeScript sources, validated the manifest, activated
  the extension in an isolated VS Code Extension Host, and executed
  `functionalPascal.showOutput`.
- `npm run package` produced and inspected
  `dist/functional-pascal-0.0.1-bootstrap.vsix`.
- Archive verification confirmed that the VSIX contains only the runtime
  manifest, compiled extension, README, and license, and that the compiled
  extension contains the exact command and activation message.
- The exact VSIX installed and uninstalled successfully with isolated VS Code
  and Cursor profiles.
- In isolated VS Code, the installed command opened the `Functional Pascal`
  output channel with the exact Hello World line. The same check passed again
  after reloading the editor window.
- VSCodium was not checked because it was not locally installed; clone testing
  is optional for this hobby-project phase.

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

### Verification — 2026-07-29

- [`contracts.md`](contracts.md) and
  `editors/vscode/contracts/phase1.json` fix the LSP 3.17, stdio, UTF-16,
  full-document synchronization, capability, native-host, and remote-host
  contracts.
- `tower-lsp-server` 0.23.0 was selected over `lsp-server` 0.10.0. The selected
  library owns async dispatch, lifecycle state, stdio serving, and request
  cancellation; the rejected option would leave more of that plumbing in this
  hobby project.
- `npm run verify:contracts --prefix editors/vscode` confirmed every LSP method
  has a named service query, every source-API reference still exists, both
  rejected-host policies have actionable messages, and every required fixture
  category is present.
- The valid standalone fixture passed `fpas fmt --check` and `fpas check`. A
  copied instance of the multi-project workspace also passed `fpas check`
  without writing derived sidecars into the checked-in fixtures.
- The malformed syntax fixture failed with `F1001`; the current unsupported
  Unicode-identifier fixture failed with `F0012`. Unicode source text in a
  string passed.
- `npm test` passed the contract, manifest, compile, and Extension Host checks.
  `npm run package` rebuilt the bootstrap VSIX with the same six runtime files;
  contracts and fixtures remain excluded.
- `cargo fmt --check`, `cargo build`, and `cargo test --workspace` passed.
- No FPAS language syntax, semantics, or current documentation changed. Runtime
  enforcement of the contracted host errors belongs to the packaging and
  server-lifecycle phases.

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

### Verification — 2026-07-29

- `package.json` registers `.fpas` as language ID `fpas` and contributes
  `language-configuration.json` plus `syntaxes/fpas.tmLanguage.json`.
- The language configuration covers line and brace-block comments, both FPAS
  block-comment styles as editor pairs, brackets, auto-closing and surrounding
  pairs, word selection, indentation, and region folding markers.
- The grammar covers declarations, named programs/units/routines/types,
  control flow, word and symbolic operators, built-in and composite types,
  language and numeric constants, strings, all FPAS comment styles, and
  qualified names. It follows the language's case-insensitive keyword and
  non-nesting comment rules.
- `vscode-textmate` 9.3.2 and `vscode-oniguruma` 2.0.1 are exact-pinned
  development dependencies. `npm run verify:grammar` loads the grammar with
  the same tokenizer stack used by VS Code and checks positive, negative, and
  edge fixtures, including escaped quotes, keyword boundaries, Unicode,
  nested-looking comments, and an incomplete file.
- The valid positive and negative fixtures passed `fpas fmt --check` and
  `fpas check`. The incomplete edge fixture is tokenizer-only by design.
- `npm test` compiled the TypeScript sources, validated the manifest and
  referenced language files, ran the grammar and Phase 1 contract regressions,
  opened a `.fpas` fixture as language `fpas` in an isolated VS Code Extension
  Host, and exercised the existing output command without host errors.
- `npm run package` produced and inspected
  `dist/functional-pascal-0.0.1-bootstrap.vsix`. Its eight packaged files
  include the grammar and language configuration while excluding development
  dependencies, scripts, contracts, tests, fixtures, and source maps.
- `cargo fmt --check`, `cargo build`, and `cargo test --workspace` passed; no
  Rust source changed in this editor-only phase.
- No compiler logic was added to TypeScript, no language server is required
  for highlighting, and no FPAS language syntax or semantics changed.

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

### Verification — 2026-07-29

- `fpas-language-service` is split into focused `document/`, `analysis/`,
  `workspace/`, and `symbols/` modules plus diagnostic and formatting facades;
  every production source file remains below 400 lines.
- `DocumentStore` provides immutable snapshots, monotonically versioned editor
  buffers, disk revisions, reusable UTF-8 line indexes, and normalized paths.
  Open buffers override disk text, including before the first save.
- `WorkspaceContext` discovers or explicitly loads loose files, projects, and
  workspaces through `fpas-project`. Invalid manifests, missing dependencies,
  invalid workspace members, and absent metadata are recoverable structured
  states.
- A new `fpas-project` parsed-source graph entry point lets analysis reuse
  existing dependency ordering and validation with in-memory ASTs. It neither
  reads compiled-unit sidecars nor requires the source paths to exist on disk.
- `LanguageService` caches loose and project analysis by the exact revisions
  of every participating source. It reuses parser and semantic-analysis
  authority, resolves project unit interfaces dependency-first, merges
  diagnostics, delegates formatting to `fpas-fmt`, and builds a declaration
  index that preserves equal short names from different units.
- Regression tests cover loose, project, and workspace analysis; invalid
  metadata, a missing dependency, malformed source and malformed unsaved unit
  text; empty, CRLF, Unicode, deleted, recreated, saved, reopened, and stale
  documents; unsaved overlays; formatting; cache invalidation; and colliding
  unqualified symbols.
- `cargo tree -p fpas-language-service --depth 1` contains only FPAS parser,
  diagnostics, formatter, project, semantic-analysis, and unit crates. There
  is no VS Code, Node, LSP, compiler, linker, VM, or execution dependency.
- `cargo clippy -p fpas-language-service --all-targets -- -D warnings`,
  `cargo clippy -p fpas-project --all-targets -- -D warnings`,
  `cargo fmt --check`, `cargo build`, and `cargo test --workspace` passed.
- The project analysis test confirms that no `.fpascu` sidecar is created.
  No FPAS source, syntax, semantics, or current language documentation changed.

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

### Verification — 2026-07-29

- `fpas-lsp` is a separate binary/library crate using exact-pinned
  `tower-lsp-server` 0.23.0. Lifecycle, synchronized-document state,
  capabilities, and protocol conversion live in focused modules; every
  production source remains below 200 lines.
- Initialize advertises only UTF-16 positions and open/close, full-change, and
  save-without-text synchronization. Diagnostics, formatting, hover,
  definitions, symbols, completion, and proposed LSP 3.18 features remain
  unadvertised.
- `didOpen`, `didChange`, `didSave`, and `didClose` feed the Phase 3
  `DocumentStore`. Incremental changes, unsupported URI schemes, unopened
  saves, and stale versions are rejected recoverably and logged to stderr.
- Position regressions cover ASCII, CRLF, BMP Unicode, surrogate-pair text,
  a split surrogate pair, end-of-line, end-of-file, invalid lines,
  out-of-range UTF-16 columns, and invalid UTF-8 byte offsets.
- Raw process transcripts cover initialize/initialized, open/change/save/close,
  shutdown/exit, a request before initialization, malformed parameters,
  malformed JSON, cancellation, unsupported URIs, stale versions, and
  incremental changes. The parser test accepts only valid
  `Content-Length`-framed JSON on stdout.
- The TypeScript extension uses exact-pinned `vscode-languageclient` 10.1.0,
  resolves only the repository debug binary in development, rejects remote or
  unsupported hosts, bundles the client into the JavaScript entry point, and
  exposes a restart command. It never searches the system `PATH`.
- A real isolated VS Code Extension Host started the Rust server, completed
  the LSP handshake, opened a `.fpas` document, restarted the server, and
  stopped cleanly. A process check after shutdown found no remaining
  `fpas-lsp` process.
- `npm audit` reported no known vulnerabilities. `npm test` and the bootstrap
  `npm run package` path passed; the resulting bootstrap VSIX intentionally
  still excludes the native binary until Phase 7.
- `cargo clippy -p fpas-lsp --all-targets -- -D warnings` passed. Full
  workspace formatting, build, tests, and Rust documentation are recorded in
  the final Phase 4 verification run.
- No FPAS source, language syntax, semantics, or current language
  documentation changed.

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

- `README.md`
- `docs/pascal/tools/fmt-style.md`
- a new current editor-integration page under `docs/pascal/tools/`
- relevant documentation indexes linking the new current page

Current documentation must describe only features verified in this phase.

### Acceptance

- An unsaved syntax error appears and disappears in the editor as expected.
- Format Document produces exactly the canonical formatter output.
- Enabling the editor's standard format-on-save setting requires no FPAS
  watcher or extra extension setting.

### Verification — 2026-07-30

- `fpas-lsp` now owns focused `diagnostics/` conversion/publication modules
  and a separate formatting-edit module. All production sources remain below
  220 lines.
- Open and full-change notifications schedule a 120 ms debounced analysis.
  Per-document generations are invalidated before synchronization, analysis
  requires the exact current editor version, and publication checks the
  generation again. Close and shutdown cancel pending work.
- Published diagnostics preserve `Fxxxx` codes, error/warning severity,
  UTF-16 ranges, the `fpas` source label, and non-empty compiler help text.
  Parser errors publish without semantic analysis; valid syntax receives
  project-aware semantic diagnostics.
- Protocol regressions cover parser and semantic failures, clearing after a
  correction, multiple ranges, both severity conversions, a project dependency
  unit, rapid versions, and close during debounce. No obsolete version is
  published in the rapid-change transcript.
- `textDocument/formatting` is advertised and formats the current unsaved
  snapshot through `fpas-fmt`. Tests prove canonical parity, a single
  whole-document LSP edit, idempotence, comment preservation, and no edit for
  malformed syntax.
- A real VS Code Extension Host opened the malformed fixture, observed
  `F1001`, cleared diagnostics after an unsaved correction, invoked the
  document formatter, applied canonical output, restarted the server, and
  shut it down cleanly.
- Current documentation now describes the implemented development-mode editor
  integration and explicitly retains the bootstrap VSIX native-binary
  boundary. No FPAS syntax or semantics changed.
- `cargo fmt --all -- --check`, `cargo build`, `cargo test --workspace`, and
  `cargo clippy -p fpas-lsp --all-targets -- -D warnings` passed.
- `npm test --prefix editors/vscode` passed, and
  `npm run package --prefix editors/vscode` recreated and inspected the
  bootstrap VSIX without adding a native server prematurely.

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

### Verification — 2026-07-30

- `fpas-language-service` now owns a protocol-independent navigation layer.
  It indexes recovered AST declarations and lexer identifier spans without
  changing `fpas-sema` or duplicating compiler behavior in TypeScript.
- Hierarchical symbols cover compilation units, types, routines, parameters,
  variables, enum members, and record fields, methods, properties, and events.
  Full and selection spans are converted from UTF-8 to LSP UTF-16 ranges.
- Hover and definition resolve declarations and references with sequential
  lexical scopes, shadowing, direct `uses` imports, qualified unit names,
  public/private visibility, record-member visibility, and library export
  policy. Cross-unit locations use the defining snapshot.
- Completion returns visible lexical and directly imported declarations.
  Member completion works after unit, type, and typed-value dots; equal
  imported candidates retain their qualified identities.
- Queries use current open overlays for every project source. Comments,
  strings, keywords, unknown or inaccessible names, files outside the loaded
  project, and incomplete member access return empty or partial results rather
  than a server error.
- Language-service regressions cover same-file and cross-unit definitions,
  shadowing, qualified names, private declarations and members, library
  exports, equal import candidates, Unicode-adjacent positions, unsaved
  declaration changes, partial syntax, and sources outside the project.
- LSP transcripts cover all four capabilities and handlers, hierarchical
  ranges, UTF-16 positions, completion details, and empty partial/unknown
  results. A real VS Code Extension Host verifies hover, document symbols,
  cross-unit definition, completion, diagnostics, formatting, restart, and
  shutdown.
- The symbol extractor is split by declarations, source spans, type members,
  and routine scopes; every new production file remains below 400 lines.
- `cargo fmt --all -- --check`, `cargo build`, `cargo test --workspace`, and
  warning-free Clippy runs for both editor crates passed. `npm test` and
  `npm run package` passed, including archive inspection of the regenerated
  bootstrap VSIX.
- No FPAS syntax, semantics, current language specification, or semantic
  analyzer behavior changed.

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

### Verification — 2026-07-30

- `scripts/package.mjs` provides the single non-interactive entry point,
  supports `--help`, rejects unknown arguments, maps Windows, Linux, and macOS
  x64/arm64 hosts, and reports an actionable error for unsupported hosts.
- The package path clears stale staged targets, runs the complete extension
  test suite, builds `fpas-lsp` in release mode without a Rust target override,
  and stages only `server/<host-target>/fpas-lsp[.exe]`.
- On the local Windows x64 host, `npm run package --prefix editors/vscode`
  produced
  `dist/functional-pascal-0.0.1-win32-x64.vsix`. Its nine archive entries
  contain exactly the target metadata, license, manifest, README, grammar,
  language configuration, bundled extension, and native server.
- Archive regressions reject missing runtime files, extra host servers,
  development files, source maps, Cargo output, unrelated executables, and
  machine-specific paths. The extension lookup regression rejects any system
  `PATH` fallback.
- The package test extracted the finished VSIX to a temporary directory and
  completed initialize, initialized, shutdown, and exit against its bundled
  server. No server executable remained after the test.
- Two consecutive package builds replaced the same output path. Every archive
  entry name and SHA-256 content hash was identical; packaging-tool metadata
  was intentionally excluded from this comparison.
- `cargo fmt --all -- --check`, `cargo build`, `cargo test --workspace`,
  `npm test --prefix editors/vscode`, both final package runs, and `npm audit`
  passed. The dependency audit reported no known vulnerabilities.
- The root README, extension README, current editor-integration documentation,
  architecture, and plan now describe the host-native local build. No FPAS
  syntax, semantics, or language specification changed.

## Phase 8 — editor acceptance and completion

### Implementation checkpoint — 2026-07-30

The installed-editor smoke test exposed false unknown-type diagnostics when
the complete Rust repository was opened: the initialized folder had no root
FPAS manifest, so the language service remained in loose-file mode.

The language service now treats the editor folder as a discovery boundary and
resolves context per source. It walks source ancestors, lazily loads the
nearest manifest that directly owns the file, supports several nested projects
in one session, prefers a direct owner over a dependency consumer, preserves
unrelated loose files, and reports overlapping nearest owners as an actionable
ambiguity. Standard-library projects use the trusted `fpas-project` loader
while retaining source provenance.

Regression coverage includes the actual repository root with
`lib/Std/Tui.fpas`, two nested projects, ownership precedence, ambiguity, and a
loose file beside a loaded project. An LSP transcript starts from a
manifest-free repository root, and the real VS Code Extension Host now opens a
manifest-free parent of its nested test workspace. The installed rebuilt VSIX
still requires the manual smoke test below, so Phase 8 remains open.

`cargo fmt --all -- --check`, `cargo build`, `cargo test --workspace`, focused
Clippy checks with warnings denied, `npm test --prefix editors/vscode`, and
`npm run package --prefix editors/vscode` passed. The package command produced
and exercised
`editors/vscode/dist/functional-pascal-0.0.1-win32-x64.vsix`.

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
