# Correctness and missing tests

Paths below are relative to `editors/vscode/` unless explicitly prefixed with `crates/`. Line numbers refer to the reviewed commit. Focused probes loaded the compiled implementation and replaced the VS Code API with small test doubles where necessary; they did not modify implementation code.

## F01, P2: The project selection command cannot change a remembered selection

Source: `src/workflow/project.ts:56`, particularly the remembered-selection return; `src/workflow/controller.ts:129`.

The command palette invokes `selector.select(undefined)`. If a remembered candidate still exists, `select` returns it before displaying the picker. The same method serves ordinary operations and the explicit Select Project or Workspace command. Consequently, after the first selection, that command does not let the user switch projects.

Reproduced with two candidates and a remembered first project: calling `select()` returned the first project and called `showQuickPick` zero times. Existing host tests always pass an explicit URI and bypass the failing path.

Separate choosing a project from resolving the current project. The explicit selection command should open the picker, while Check/Build can reuse the current target. Add a command-palette test that selects A, invokes selection without arguments, chooses B, and verifies the next operation uses B.

## F02, P2: Test execution and result mapping discard directory identity

Source: `src/workflow/testing.ts:171` and `:188`; `src/workflow/arguments.ts:32`. CLI counterpart: `crates/fpas-cli/src/cli_test/run/mod.rs:33` and `crates/fpas-cli/src/cli_test/discover.rs:67`.

Discovery returns absolute source paths. Selected execution reduces each path to its basename and sends it as the CLI's substring filter. The JSON report also contains basenames. The extension resolves report names relative to the manifest directory, which does not reconstruct the original path for tests in subdirectories.

A real scratch project contained `a/same_test.fpas` and `b/same_test.fpas`. `fpas test --list` returned both absolute paths. The invocation produced by selecting either file, `--filter same_test.fpas`, ran both tests. Both report entries had `file: "same_test.fpas"`. A probe of the real `WorkflowTesting` implementation with that report recorded zero statuses for the nested items.

This also affects uniquely named nested tests: result mapping still points to the wrong directory. Duplicate names add unintended execution and ambiguity. The existing host fixture places every test beside its manifest and gives each a distinct basename.

Preserve a stable source identity in the CLI JSON report and support exact selection, retaining project linking context. Update the extension and CLI contract together. Do not infer identity from a display label. Add nested, duplicate-basename, and substring-overlap fixtures; verify both the files executed and the items whose status changes.

## F03, P2: TestRunRequest.exclude is never honored

Source: `src/workflow/testing.ts:146`, `:150`, and `:221`.

`requestedItems` only visits `include`, or returns all mapped items. It never consults `exclude`. The all-tests path also invokes the entire manifest without exclusions, so filtering the displayed queue alone would not fix execution.

A request excluding one of two discovered items still produced two requested items in the probe. The official [Testing API guide](https://code.visualstudio.com/api/extension-guides/testing#running-tests) requires run handlers to account for both included and excluded tests.

Expand the requested tree, subtract excluded items and their descendants, and execute precisely the remaining set. Coordinate with F02 so basename filters do not reintroduce excluded tests. Cover run-all with an exclusion, included parent with excluded child, and an entirely excluded request. `test/workflow/host.ts` calls the extension's `runTests(files)` helper and does not exercise exclusion requests.

## F04, P2: Concurrent LSP starts violate single-client ownership

Source: `src/languageClient.ts:24`, `:56`, `:61`, and `:70`; restart callers in `src/extension.ts:87` and `:111`.

`start` checks `this.client`, then awaits toolchain resolution and client startup before assigning it. Two callers can both pass the check and start separate clients. The last assignment replaces the first reference. `stop` only stops the retained client. A stop during startup can similarly return before the new client is assigned.

Using a delayed fake LanguageClient, `Promise.all([controller.start(), controller.start()])` created two clients. A subsequent `stop()` stopped only one. This demonstrates the controller race, not two observed native LSP processes in the full host suite. Activation, manual restart, and configuration-change restart can overlap in the real extension.

Serialize start/stop/restart or use an explicit pending-operation state with generation handling. Ensure invalidated startup results are stopped and disposed. Test concurrent start, stop during start, repeated configuration changes, and failed startup followed by retry. The existing sequential restart test cannot catch this race.

## F05, P2: Terminal input decoding depends on chunk boundaries and can stall permanently

Source: `src/debugger/terminal/input.ts:39`, `:89`, and the final `break` in `feed`.

An isolated ESC byte is immediately emitted as Escape. If the remaining bytes of an arrow sequence arrive in the next socket chunk, they become literal characters. Conversely, an unknown complete CSI sequence is retained forever because no branch consumes or rejects it. Every subsequent ordinary character accumulates behind it and produces no event.

Reproduced directly:

```text
feed(ESC), then feed("[A") -> Escape, Character "[", Character "A"
feed(ESC + "[99~"), then feed("abc") -> no events
```

The external terminal receives a byte stream, where chunk boundaries do not identify key boundaries. Use a bounded incremental parser, a defined escape-key ambiguity timeout, and recovery after an unsupported complete sequence. Test each possible split of supported CSI, SS3, mouse, and paste sequences. Verify that unsupported input does not suppress subsequent normal text. Existing decoder tests split pasted text, but not the escape prefix itself.

## F06, P2: Buttonless mouse movement is misclassified

Source: `src/debugger/terminal/input.ts:180`, especially lines 186-189.

The release branch checks `buttonCode === 3` before the motion-bit branch. For motion with no button, code 35 contains both the motion bit and button code 3. It therefore produces `Up` and never reaches the intended `Move` case. That `Move` expression is unreachable for button code 3.

Reproduction: `feed(ESC + "[<35;12;7M")` returns `action: "Up", button: "None"`; it should represent movement. The [xterm mouse tracking specification](https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h2-Mouse-Tracking) distinguishes motion reports, including motion without buttons, from releases.

Decode the motion flag before interpreting the no-button code as release, while retaining the SGR lowercase-`m` release rule. Add move, drag, release, wheel, and modifier cases. The existing terminal test covers only one left-button press.

## F07, P2: Terminal batches exceed the adapter's hard event limit

Source: `src/debugger/terminal/session.ts:137` and `src/debugger/terminal/external.ts:218`. Receiver: `crates/fpas-debug/src/dap/server/io.rs:78`.

Both terminal transports send every decoded event from an input chunk in one `fpas/terminalInput` request. The DAP adapter rejects arrays larger than 256. An unbracketed 257-character input chunk generates 257 key events and is sent unchanged. The whole request is rejected, losing that chunk of program input. Queued input flushed after initialization has the same problem.

A probe routed 257 ordinary characters through the real integrated terminal manager and recorded one request containing 257 events. The adapter's rejection was established from its explicit `events.len() > 256` check. Bracketed paste becomes one paste event and does not exercise this failure.

Share an ordered batching implementation between transports, limiting requests to the negotiated/current adapter bound and handling errors without silently discarding later input. Test 256, 257, and several batches, including pre-initialization queued input.

## F08, P2: Case folding confuses distinct paths on case-sensitive filesystems

Source: `src/workflow/project.ts:19` and `:65`; `src/workflow/testing.ts:90`, `:101`, and `:192`.

Project comparison and test map keys always use `toLocaleLowerCase`. On a case-sensitive filesystem, `A.fpasprj` and `a.fpasprj`, or similarly named test paths, are distinct. Selection can return the wrong project; the test map overwrites one item with another.

The pure helper returns `/work/A.fpasprj` for a remembered `/work/a.fpasprj` when both candidates are supplied in that order. This was run with strings on Windows, not on a Linux filesystem. The filesystem consequence follows from the unconditional key transformation. `src/debugger/projectTarget.ts` already uses a platform-aware comparison, so the extension is internally inconsistent.

Use one path-identity policy that respects the supported filesystem semantics. Add a Linux host case with both distinct paths and retain Windows comparison tests. Avoid locale-sensitive string transformations for identity.

## F09, P2: The hand-written TOML subset changes valid main paths

Source: `src/debugger/projectTarget.ts:45`, `:87`, and `:106`.

Zero-configuration F5 scans project files with a line parser and decodes basic strings through `JSON.parse`. It does not implement TOML string syntax. A valid multiline literal written on one line, `main = '''src/main.fpas'''`, is interpreted by stripping one quote at each end, leaving two extra quotes in the path. The source then fails ownership matching and falls back to a loose-file launch, losing the selected project's context.

The probe returned a path ending in `''src/main.fpas''`. Multiline literal strings are part of the [TOML specification](https://toml.io/en/v1.0.0#string). Other differences include multiline basic strings and TOML Unicode escape forms; those were not individually executed here.

Use the existing authoritative project loader through a suitable query, or a proper TOML parser if local parsing remains necessary. Test valid TOML spellings of the same main path and malformed manifests. Keep project interpretation shared with the compiler rather than expanding another partial parser incrementally.
