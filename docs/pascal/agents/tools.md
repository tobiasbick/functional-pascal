# Find APIs and validate changes

Read this when selecting a tool, discovering APIs, or investigating program
behavior. Commands below assume `fpas` is the intended toolchain. Use focused
command help for current flags. In this repository, `--std-lib lib` explicitly
selects the checkout's standard library.

## Ask the matching tool

| Question | Command or reference |
| --- | --- |
| What is this declaration or built-in operation? | `fpas doc --report json Std.Fs.ReadText` or `fpas doc string.Slice` |
| Which declarations does a unit expose? | `fpas doc Std.Fs`; the editor's **Show Public Signatures** also includes unsaved unit edits |
| Which binding/type does this source expression have? | `fpas query definition file.fpas:12:5` or `fpas query type file.fpas:12:5` |
| Where is the declaration used? | `fpas query refs file.fpas:12:5` |
| Does the program check? | `fpas check --diagnostics json app.fpasprj` |
| Which tests are selected? | `fpas test app-tests.fpasprj --list` |
| What passes or fails? | `fpas test app-tests.fpasprj --report json` |
| How do I create a project? | `fpas init project my-app`; see [initializing](../program-structure/initializing.md) |

Use explicit project/workspace paths when discovery is ambiguous. Source queries
use `--context <manifest>`; inspect `coverage.complete` before treating no matches
as proof of absence. Incomplete analysis exits 3. Safe rename uses
`fpas query rename file.fpas:12:5 NewName --dry-run`; its context selector is
`--project`. Review the plan before applying. See [source queries](../tools/source-queries.md).

## Check the actual sources

Run `fpas fmt <paths>`, then `fpas fmt --check <paths>` and
`fpas check <file-or-project>`. Formatting uses two-space indentation and a fixed
100-column expression target, with no configuration. `fpas check --fix` applies
machine-applicable compiler fixes in place; format and inspect the result.
Run the owning test project for runtime behavior. Interactive programs can be
checked without launching their UI; use scripted/headless tests for execution.

Handbook Pascal fences are checked directly by
`fpas run tools/doc-examples/doc-examples.fpasprj`. The
[example instructions](../tools/documentation-examples.md) explain compile-only,
run, expected-error, fragment and companion-project fences.

## Inspect failures and performance

Use `fpas debug file.fpas --break file.fpas:42 --print Total` for one bounded
JSON snapshot at a breakpoint, failure or completion. It never reads host stdin;
`--script` supplies runner-compatible input. Print expressions do not execute
calls. See [batch debugging](../tools/debugger-batch.md) for bounds and exit codes.
For compiled images, use the `Debug source root` printed by build as `--source-root`.

For a scalar's history, add `--trace Total` or an exact selector such as
`--trace 'Update::State.Selected'`; inspect `trace.complete` and reasons.
[Tracepoints](../tools/debugger-tracepoints.md) explain fields, array positions,
local lifetimes and active captures. To explore controlled task order, use
`--schedule-seed 42` and read [seeded scheduling](../tools/debugger-scheduling.md).
It uses virtual time and controlled effects; it does not replay normal pool execution.

`fpas run app.fpasprj --failure-dump .temp-data/failure.json` retains an escaping
failure from normal execution. The report has bounded task/frame/value data;
inspect `capture.incomplete` before assuming omitted data was absent.
Successful runs produce no failure report.

Start performance diagnosis with
`fpas run app.fpasprj --profile .temp-data/profile.json`.
For a debugger target, use `fpas debug ... --profile <new-file>`.
[Profiles](../tools/profiler.md) contain hot routines, lines, counters and findings;
debug profiles also contain phase timings. Report destinations must be new files.
Use the [benchmark workflow](../../bench/README.md) for controlled unprofiled
before/after measurements. Read [CLI](../program-structure/cli.md) and
[formatter style](../tools/fmt-style.md) for full contracts.
