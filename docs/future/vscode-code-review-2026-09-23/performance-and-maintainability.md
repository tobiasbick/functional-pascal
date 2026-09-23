# Performance and maintainability

These costs follow from source inspection. No performance benchmark was run.

## P01, P2: Ordinary terminal text causes quadratic decoding work

`src/debugger/terminal/input.ts:45` evaluates `[...this.pending][0]` for each character, then slices off that character. Spreading the string traverses and allocates an array for the entire remaining input. A chunk of n ordinary characters therefore performs work proportional to n + n-1 + ... + 1. The Alt-key path at line 97 repeats the same pattern.

Use `codePointAt` with scalar width or a single-pass iterator and a cursor, slicing retained incomplete input once per feed. Preserve astral characters and control-sequence parsing. Benchmark doubling plain-text chunks and fragmented paste input after fixing F05/F07. Avoid optimizing by assuming one UTF-16 code unit per character.

## P02, P3: Project lookup repeatedly scans the complete workspace

`src/workflow/project.ts:34` calls `workspace.findFiles` whenever `current` or `select` runs. Status updates call `current` again, including after commands and selection events. `src/debugger/projectTarget.ts:22` independently scans manifests on each source-file F5 launch and reads every candidate serially.

Large workspaces pay repeatedly for discovery even when no manifest has changed. This is especially visible in a repository containing many examples and applications. The comment describing candidates as bounded is not backed by a result-count limit; only excluded directory names constrain the search.

Share a manifest index with invalidation on relevant file and workspace changes. Resolve project ownership through the authoritative loader where possible. Keep refresh cancellation and stale-result handling explicit. Measure repeated status updates and F5 lookup on a workspace with many manifests before choosing cache complexity.

## P03, P3: Output retention and terminal buffering have no byte budget

`src/workflow/processes.ts:24-40` accumulates all stdout and stderr in strings while also appending them to the output channel. `src/debugger/terminal/external.ts` queues output until authentication and ignores socket write backpressure. Terminal input can also accumulate before initialization. F05 introduces an additional indefinite input-retention path after unsupported CSI input.

Long-running or noisy commands can retain substantial data in the extension host. No memory measurement or host failure was reproduced. Separate bounded diagnostic/report capture from ordinary streamed logs, report truncation explicitly, and impose a lifecycle deadline on terminal connection setup. Preserve complete JSON reports through a deliberate limit or temporary file, rather than truncating them into invalid JSON.

The external client also calls `process.exit` immediately after receiving a finish message, following `stdout.write` calls in the same message loop. Node documents that forced exit can discard pending asynchronous output. This is a source-derived portability risk, not observed data loss in the Windows test run. Finish after output drains and terminal resources close. See [Node process.exit documentation](https://nodejs.org/api/process.html#processexitcode).

## M01, P3: Debugger commands duplicate selection, prompts, and error handling

`src/debugger/dictionaryCommands.ts:63` and `src/debugger/sequenceCommands.ts:62` repeat the active-session/stack-frame checks, prompt validation, and request wrappers. Related copies occur in `forcedReturnCommand.ts`, `storageInitializationCommand.ts`, and `variantConstructionCommand.ts`.

A focused debugger selection module and expression prompt module would centralize the repeated behavior. Keep each command's mutation semantics and messages in its own thematic file. Do not replace these small modules with a configurable command framework. Verify cancelled prompts, wrong session type, explicitly supplied frame IDs, and changed active frames while a prompt is open.

## M02, P3: The VSIX includes redundant debugger modules

`scripts/compile.mjs` emits individual modules with TypeScript and bundles the extension entry point with esbuild. The archive retains separate debugger command and adapter modules even though the extension entry point already contains their implementation. The standalone external terminal client and its input decoder do need their own files.

The reviewed archive contains 24 entries and is about 193 KB compressed. This is modest packaging overhead, not a demonstrated startup bottleneck. Remove unreachable individual modules from the archive if simplifying packaging. `scripts/verify-package.mjs` currently hardcodes their presence, so update the expected inventory at the same time.

After exact entry-set equality, the verifier loops over the same expected names and asserts each is present again. That second loop adds no coverage. Keep meaningful negative assertions such as forbidden toolchain payloads and leaked local paths.

## Defensive programming assessment

Executable validation, toolchain schema/version checks, diagnostic report validation, cancellation cleanup, terminal authentication, and package-content checks protect actual boundaries and should remain. Exact CLI version matching is an explicit compatibility policy in this hobby project; this review does not classify it as a bug.

The clearest unnecessary defensive work is redundant package assertions. The more important design issue is duplicated ownership logic: repeated lifecycle guards do not prevent F04 because they do not serialize asynchronous operations. Prefer one owner and explicit lifecycle state over adding more independent booleans.
