# Functional Pascal

This is the local editor extension for Functional Pascal. It provides `.fpas`
language detection, TextMate syntax highlighting, comment and bracket
configuration, indentation, and folding. The selected installed toolchain's
`fpas lsp` server provides parser and semantic diagnostics, canonical
whole-document formatting,
document symbols, hover, same- and cross-unit go to definition, find all
references, workspace symbol search, document highlights, go to type
definition, syntax-aware selection expansion, validated project-wide rename,
rich visibility-aware completion, lazy completion documentation, signature
help, checked FPAS snippets, and safe unambiguous auto-imports. The repository
builds and tests the extension without a Marketplace. Compiler-backed semantic
tokens distinguish resolved declarations and references, and an `F2001` or
`F2003` diagnostic can offer a safe import quick fix when exactly one public,
accessible unit provides the missing type or callable.
The extension resolves `fpas` from `functionalPascal.executablePath` or `PATH`
for project check, build, run, test, format, and format-check commands, Problems
integration, and the Testing view.
The extension runs in the local desktop extension host and uses a locally
installed FPAS toolchain. Remote workspaces (SSH, containers, and WSL opened
through VS Code Remote) are unsupported; toolchain commands report that
limitation. The extension does not declare support for Restricted Mode, so VS
Code disables its executable tooling until the workspace is trusted.
The extension also contributes the `fpas` debugger. Press **F5** with a
`.fpas`, `.fpasprj`, or `.fpasworkspace` editor active. For a `.fpas` program
main, zero-configuration launch discovers and debugs the owning `.fpasprj`, so
its project dependencies are retained. Use **Run and Debug** with the generated
**Debug Functional Pascal** configuration, or set `program` to a `.fpas`,
program `.fpasprj`, `.fpasworkspace`, or `.fpascp` target. Compiled
images additionally require `sourceRoot`. Set source breakpoints in an `.fpas`
editor gutter or with **F9**. Breakpoints, stepping, stack frames, scopes,
variables, read-only watches/hover/Debug Console evaluation, conditional
breakpoints, exact positive-integer hit conditions, non-stopping logpoints,
and program output use the selected CLI's DAP adapter and its adjacent source
standard library. The
language server remains responsible only for static editor features.
Use the Debug toolbar for Continue, Pause, Step Into, Step Over, Step Out, and
Stop. While stopped, **Functional Pascal: Debug: Force Return** completes the
selected ordinary callee — including an older frame — with a validated result
and stays stopped in that frame's caller. The Run and Debug sidebar exposes the
call stack, lexical scopes, locals, parameters, globals, and expandable
aggregate values. Evaluated aggregates are also expandable until execution
resumes. Program terminal output appears in a dedicated integrated terminal by
default. Keyboard, mouse, paste, focus, and resize events are forwarded to
`Std.Console` and `Std.Tui` while the program is running. Set `console` to
`debugConsole` to retain non-interactive output-only behavior. Logpoint text
and structured runtime failures appear in the Debug Console. Log messages use
`{expression}` interpolation and `{{`/`}}` for literal braces. Debugger-side
calls may invoke deterministic functions, procedures, record methods,
constructors, readable properties, visible closures, and pure `Std.*`
intrinsics. They run against a detached copy of stopped state; writes are
discarded, and calls involving output, files, processes, environment, time,
randomness, blocking, tasks, or unknown dynamic effects are rejected.
Use **Functional Pascal: Debug: Reload Compatible Changes** to rebuild the
exact launch target while initialized or stopped. Changes limited to inactive
function bodies commit atomically and refresh sources, breakpoints, stacks, and
variables. **Debug: Roll Back Last Reload** restores the single preceding image
as a new version. Active bodies, function-set or layout changes, captures, and
anonymous closures are rejected before the live image changes. This reloads
FPAS program code only; it does not debug or replace the native Rust VM.
While stopped, the Variables view can edit mutable locals, parameters, globals,
closure captures, record fields, array elements, existing dictionary
values, active enum payload fields, `Result`/`Option` `.value` children, and
complete mutable enum, `Result`, and `Option` values using constructor
expressions such as `Choice.Pair(1, 2)` or `None`. A function-typed Variables
or Watch target can be replaced by copying one visible binding that already
holds a compatible function value, or by assigning a unique
executable routine such as `AddTwo`, `AddBase`, or `AddCell`. Named nested
routines materialize from the selected lexical-owner frame using recorded
immutable values and existing mutable cells. Constructed cell-capturing
functions are task-bound to the selected task and may be stored only in a
mutable local or parameter register of that owner frame. An already
materialized task-bound function may be copied only within that selected owner
task and frame; the exact function and cell handles are preserved. A task-typed Variables or
Watch target can be replaced by copying one visible binding that already holds
a compatible task handle, for example `Pending`. The editor uses the standard
Variables/Watch edit flow; it does not add a custom command. Numeric IDs,
`<task N>` display text, Dynamic endpoints, and complete aggregates that
contain task handles remain rejected. Editor clients can also use
the standard DAP `setExpression` request for an explicit inactive
single-payload variant such as `Optional.Some.value` or
`Selected.Count.Value`; in VS Code this is the **Set Value** action on a Watch
expression. Uninitialized mutable
locals and globals can be assigned one complete value through the same
Variables and Watch edits.
Rejected edits leave the session stopped and unchanged. A successful
edit refreshes the Variables view; continuing execution observes the committed
value unless a later source initializer overwrites it. Dictionary keys are not
edited through the standard Variables request; immutable bindings,
evaluation-only results, and opaque host values are not editable.
Function values are editable by copying an already materialized, visible,
function binding, or by assigning a unique executable routine
including a named nested routine whose captures are immutable values or
existing mutable cells. Task handles are editable by copying one visible
initialized binding whose declared task result type matches the destination.
Uninitialized bindings have no writable
descendants. Variables does not list inactive variants as children. A write to
an old payload-child handle never selects a
different variant. Task debugging is deterministic and all-stop;
attach remains unsupported.

Use **Functional Pascal: Debug: Insert Dictionary Entry**, **Debug: Remove
Dictionary Entry**, or **Debug: Replace Dictionary Key** while stopped to
change dictionary structure. The commands prompt for a complete mutable
dictionary target and FPAS key/value expressions. Insert appends a missing
pair, remove deletes an existing pair, and key replacement preserves the
associated value and iteration position. Failures and cancelled prompts leave
the stopped program unchanged.

Use **Functional Pascal: Debug: Insert Array Element**, **Debug: Remove Array
Element**, or **Debug: Replace String Character** for bounded sequence changes.
Array insertion accepts indexes from zero through the current length; removal
uses an existing zero-based index. String indexes count Unicode characters and
the replacement must be a one-character FPAS string. Successful commands
refresh debugger variables; failures and cancelled prompts send no mutation.

Use **Functional Pascal: Debug: Force Return** while stopped on a selected
ordinary callee. Procedures complete without a prompt; functions prompt for one
FPAS return expression unless the command is invoked with an expression. The
command completes the selected frame and every younger frame, leaves the
session stopped in the selected frame's caller, and refreshes the call stack
and Variables view. It does not resume the program.

Use **Functional Pascal: Debug: Construct Variant** while stopped on a mutable
enum, `Result`, or `Option` target. The command discovers legal variants,
prompts for each declared field expression, and commits one complete value
without resuming. Fieldless variants need no field expressions. Cancelled
prompts leave the stopped program unchanged.

Use **Functional Pascal: Debug: Initialize Empty Storage** while stopped on an
empty mutable local or global. The command prompts for a descendant target, a
complete root initializer, and a replacement expression, then commits the
rebuilt root once without resuming. Cancelled prompts leave the stopped
program unchanged. A later source initializer still overwrites the debugger
value.

## Build

Node.js 22 or newer and a stable Rust toolchain are required. From the
repository root, install the pinned Node dependencies once:

```text
npm ci --prefix editors/vscode
```

Then build the VSIX:

```text
npm run package --prefix editors/vscode
```

The command runs the extension tests, creates a platform-independent archive,
and verifies that no compiler, language-server binary, or standard-library
source is included. It produces:

```text
editors/vscode/dist/functional-pascal-<version>.vsix
```

Install an FPAS distribution containing `fpas`, `fpas-lsp`, `fpas-runner`, and
`lib/` separately. Put its directory on `PATH`, or use **Functional Pascal:
Select FPAS Executable**. The corresponding machine-scoped
`functionalPascal.executablePath` setting overrides `PATH`.

Install the resulting file through **Extensions: Install from VSIX** in a
VS Code-compatible desktop editor. No registry login or publication is
required.

## Verify

Open a `.fpas` file and confirm the status bar identifies the language as
**Functional Pascal**. `.fpasprj` and `.fpasworkspace` files are recognized as
**Functional Pascal Project** and can be launched with **F5**. Syntax highlighting works before the extension's
TypeScript entry point is activated.

Introduce a syntax or type error and confirm the editor reports an `Fxxxx`
diagnostic for the unsaved buffer. Run **Format Document** and confirm the
result matches `fpas fmt`. The editor's standard `editor.formatOnSave` setting
uses the same formatter without an FPAS-specific setting.

Open the Outline view to inspect FPAS declarations. A contiguous standalone `//` block immediately
before a declaration is Markdown documentation; hover and resolved completion items display it.
The selected toolchain's standard library contains editor-only declarations for Rust-backed intrinsic
`Std.*` units. They provide the same hover, completion, signature, and definition experience and
open as ordinary read-only `.fpas` files without becoming part of program compilation.
Hover a declaration or
reference, use **Go to Definition** or **Go to Type Definition**, search all
project declarations with **Go to Symbol in Workspace** (`Ctrl+T`), and invoke
completion in a routine body or after a unit/record `.`. Resolved identifiers
highlight their declaration, reads, and writes in the current document.
Selection expansion follows enclosing FPAS syntax. Project-aware results use
the same `.fpasprj`,
`.fpasworkspace`, visibility, and library-export boundaries as the compiler.
Use **Find All References** (`Shift+F12`) to list resolved declarations and
usages, including uses in indexed programs that consume a directly owned
library, and **Rename Symbol** (`F2`) to validate and edit a normal declaration
across those loaded projects. Program/unit renames and declarations outside
the opened folder are intentionally rejected.

Completion includes parameters, locals, imported declarations, record and enum
members, and context-appropriate keywords with accurate kinds, owners, types,
signatures, and replacement ranges. Signature help tracks nested and multiline
calls. Type a prefix such as `function`, `record`, `if`, or `for` to select a
repository-owned snippet. A completion may add a `uses` entry only when one
accessible public declaration has one unambiguous unit import; ambiguous or
inaccessible names are never guessed.

Semantic highlighting refines the TextMate colors for resolved units, types,
enums, type parameters, routines, parameters, variables, members, enum values,
and constants. TextMate highlighting remains the startup and recovery fallback.
Use **Quick Fix** (`Ctrl+.`) on an unknown type or callable diagnostic to add a
`uses` import when that edit is uniquely determined and the resulting source is
parseable and canonically formatted. Stale, ambiguous, or inaccessible
diagnostics produce no edit.

The folder opened in the editor may be the complete Functional Pascal Rust
repository or another parent folder without an FPAS manifest. The server
catalogs the `.fpasprj` and `.fpasworkspace` manifests in that folder and uses
the normal project loader to determine their sources and relationships.
Multiple nested FPAS projects can be used in one editor session; files without
a matching manifest remain available as loose files. External source and
manifest changes refresh affected analysis, references, and rename results
without a language-server restart, while unsaved open buffers remain
authoritative.

The extension obtains the source standard-library directory from
`fpas env --json`. `Std.Tui` and the other source-defined `Std.*` units
therefore come from the same installed toolchain used for builds and debugging,
not from the opened project or the VSIX.

Select **Functional Pascal: Select Project or Workspace** before using project
commands in a folder containing multiple manifests. The remembered selection
appears in the status bar. Run the selection command again to switch projects.
**Check Project**, **Build Project**, **Test
Project**, **Format Project**, and **Check Project Formatting** run the selected
CLI without a shell and publish compiler failures in Problems. **Cancel Active
Operation** stops a running non-interactive command. **Run Project in Terminal**
starts the normal interactive CLI and accepts program arguments as a JSON
string array. `functionalPascal.programTerminal` chooses an integrated editor
terminal or a separate operating-system terminal window. F5 uses the same
default; an explicit `console` value in `launch.json` overrides it. External
debug terminals retain the same keyboard, mouse, resize, paste, focus, and TUI
event bridge as the integrated terminal.

The Testing view discovers `*_test.fpas` files for the selected manifest and
supports all, selected, filtered, excluded, and rerun requests. Selected files
are passed to `fpas test --file` by exact path, so nested tests with the same
basename retain separate results. Outcomes distinguish
pass, assertion failure, skip, compile error, runtime error, and timeout. Set
`functionalPascal.testTimeoutSeconds` to change the default 10-second per-test
limit.

CLI output captured for a workflow command is limited to 16 MiB per stream;
an overlong report stops the command with an error instead of parsing partial
JSON. Terminal input is limited to 1 MiB per paste and 4 MiB in its pending
queue. The external terminal buffers at most 4 MiB of output awaiting its
local socket; exceeding that limit stops the debug session with an error.

Run **Functional Pascal: Show Output** from the Command Palette. The
`Functional Pascal` output channel must contain:

```text
Functional Pascal extension activated.
```

The test command builds `target/debug/fpas[.exe]`, adds that directory to the
Extension Host's test-only `PATH`, and starts `fpas lsp` from a real VS Code
Extension Host. It verifies diagnostics, formatting, document symbols,
hover, cross-unit definition and type definition, workspace symbols, document
highlights, references, rename, rich completion, signature help, snippets, and
a safe auto-import, semantic tokens, an applied diagnostic quick fix, project
commands, Problems, cancellation, Testing API outcomes, read-only evaluation
and controlled calls in every supported DAP context, detached-state recovery,
mutable scalar and aggregate Variables-view edits with invalidation and
continued-execution checks, function-value and task-handle assignment through
the standard edit flow, conditional and exact-hit stops, and non-stopping
logpoints,
restarts it once, and shuts it down with the extension:

```text
npm test --prefix editors/vscode
```

The runner pins VS Code 1.137.0 and creates a fresh user-data directory for each
run, removing it after the Extension Host exits. Downloaded VS Code binaries
remain cached. Set `FPAS_VSCODE_TEST_VERSION=1.91.0` when running the test
command to check the minimum declared editor version. Run
`npm run test:restricted --prefix editors/vscode` to check that the extension
stays inactive in an untrusted workspace. The F9 check opens the source with its breakpoint selection
already applied before invoking the editor command.
Debugger tests capture sessions through the start event and normally wait for
their session to become active. The non-stopping logpoint test skips the active
session wait because its program can finish before the UI selects the session.

For daily use, record reproducible problems with the local
[bug-report template](BUG_REPORT.md). The extension has no telemetry and sends
nothing automatically.
