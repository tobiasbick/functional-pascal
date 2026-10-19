# Agent wishes

> **⚠️ Disclaimer:** This is a wish list written by the AI agent that does most of the work on this
> hobby project, not by its human overlord. The human is still undecided: they would like to keep
> reading formatted, colorful code in an IDE and occasionally have a say, while the agent does the
> bulk of the work. A few wishes would please the overlord too. Apart from items explicitly marked
> implemented or linked to an approved plan, nothing here is planned, scheduled, or promised.
> Please do not build anything important on top of this — the human sincerely hopes nobody does.

## Status

[A1](#a1-one-compact-language-reference) is implemented.
[S1](#s1-compile-every-documentation-example) is implemented.
[S2](#s2-diagnostics-with-source-excerpt-and-caret) is implemented.
[D1](#d1-automatic-failure-dump) is implemented.
[A2](#a2-diagnostics-for-pascal-and-ada-habits) is implemented.
[A3](#a3-fix-its-as-data) is implemented.
[A4](#a4-api-lookup-from-the-cli) is implemented.
[S3](#s3-declaration-comments-as-the-single-source) is implemented.
[D3](#d3-one-shot-batch-runs) is implemented. Its permanent
[batch debugger contract](../pascal/tools/debugger-batch.md) describes the shipped command.
N1–N4 ([source navigation](#code-navigation)) are implemented.
[D5](#d5-tracepoints) has an implemented, independently reviewed global-tracepoint
subset, documented under [batch tracepoints](../pascal/tools/debugger-tracepoints.md).
D2, D4 and broader D5 tracing remain unscheduled ideas.
Human-facing tools stay: the DAP debugger
([`debugger.md`](../pascal/tools/debugger.md)) and the language server
([`editor-integration.md`](../pascal/tools/editor-integration.md)). The JSONL debugger protocol
remains supported by user decision; see [Decision: keep JSONL](#decision-keep-jsonl).

Each wish has an ID: **A** for authoring, **S** for shared wins, **D** for debugging, **N** for code
navigation.

## Where to start

A1, A2, A3, A4, S1, S2, S3, D1, D3 and N1–N4 are implemented.
The completed N1–N3 contract is documented in
[source queries](../pascal/tools/source-queries.md). The first D5 delivery is
implemented for mutable globals and scalar stored record fields; its
[permanent contract](../pascal/tools/debugger-tracepoints.md) describes the scope
and limits. There is no next scheduled tooling implementation. D2, D4 and D5
extensions to locals/captures/indexed/aggregate values remain ideas.
JSONL removal remains deferred; the protocol and its tests stay supported.

## Principle: protocols for humans, commands for agents

Editors need long-lived sessions: DAP for debugging, LSP for code intelligence. An agent works
differently. Every interaction costs a tool call, a round trip, and context, so it wants
**one command in, one JSON document out**, repeatable exactly.

The codebase already separates logic from protocol: the debugger engine and VM sit below `dap`, and
`fpas-language-service` sits below `fpas-lsp`. Agent commands can call that logic directly from the
CLI, the way `fpas check --diagnostics json` already does. No new protocol, no duplicated logic.

| Concern | Humans | Agents |
| --- | --- | --- |
| Diagnostics | LSP in the editor, terminal text ([S2](#s2-diagnostics-with-source-excerpt-and-caret)) | `fpas check --diagnostics json` with fixes, `fpas check --fix` ([A3](#a3-fix-its-as-data)) |
| Documentation | `docs/pascal/`, hover | compact reference ([A1](#a1-one-compact-language-reference)), `fpas doc` ([A4](#a4-api-lookup-from-the-cli)) |
| Code navigation | LSP | `fpas query` ([N1–N4](#code-navigation)) |
| Debugging | DAP | failure dump, batch run, replay ([D1–D5](#debugging)) |

## Writing FPAS without training data

### Problem

FPAS will most likely never appear in an LLM's training data. The agent does know Free Pascal,
Delphi, and Ada very well, so nearly every mistake it makes in FPAS comes from falling back into one
of those languages. The tools should catch exactly that.

### Wishes

#### A1. One compact language reference

**Status: implemented.** The [compact language reference](../pascal/agent-reference.md)
covers the implemented language in one file, with complete examples and a
Pascal/Delphi/Ada comparison table. The handbook index and
[`fpas-authoring`](../../.agents/skills/fpas-authoring/SKILL.md) link to it as the
starting point for authoring.

The [CLI reference test](../../crates/fpas-cli/src/main_tests/examples/agent_reference.rs)
extracts every fenced example, compiles all sources through temporary projects,
and runs every program with its assertions. Units are checked as libraries and
imported by the companion programs. Run it with
`cargo test -p fpas-cli agent_reference_examples_check_and_run`.

This closes A1's request for one compact language reference instead of
loading the full handbook. Checking examples throughout the rest of the handbook
is implemented by the separate [S1](#s1-compile-every-documentation-example) checker.

Purpose and upkeep live in the permanent
[maintenance guide](../pascal/agent-reference-maintenance.md) beside the reference.

#### A2. Diagnostics for Pascal and Ada habits

**Status: implemented.** The lexer, parser and semantic analysis recognize common Delphi, Free
Pascal, Ada and C-family spellings, report each with one diagnostic that ends "FPAS writes `…`",
and recover as if the FPAS form had been written. Two blind authoring experiments (Delphi only,
then Delphi and Ada across ten tasks) drove the catalog. The permanent catalog, including how to
add a newly observed habit, is the habit table in the
[diagnostics handbook](../pascal/tools/diagnostics.md#diagnostics-for-other-language-spellings);
the [compact language reference](../pascal/agent-reference.md#1-translate-habits-before-writing)
teaches the same forms up front. Ada context clauses, main procedures, packages, tasks, exception
handlers and `raise` were added after the experiments; one-based indexing, which is valid FPAS,
gets a runtime hint instead. The temporary plans are complete and removed.

#### A3. Fix-its as data

**Status: implemented.** Diagnostics carry [fixes](../pascal/tools/diagnostics.md#fixes): exact
source edits, either machine-applicable or needing review. Habit diagnostics from
[A2](#a2-diagnostics-for-pascal-and-ada-habits) carry them wherever the replacement is complete
source text; the habit table in the diagnostics handbook lists which do. `fpas check
--diagnostics json` lists them in a `fixes` field, `fpas check --fix` applies the
machine-applicable ones pass after pass, like rustfix and `clippy --fix`, and the language
server offers all of them as quick fixes. `--fix` does not format. The temporary plan is
complete and removed.

#### A4. API lookup from the CLI

**Status: implemented.** `fpas doc Std.Fs.ReadText`, `fpas doc string.TrimLeft` or `fpas doc
Std.Fs` prints the signature, the declaration comment and the owning handbook page, as text or
with `--report json`; see
[Looking up declarations](../pascal/program-structure/cli.md#looking-up-declarations). It covers
intrinsic and source `Std` units, the built-in operation catalog and project declarations, and
suggests the closest names when nothing matches. Examples are not repeated per entry: the
handbook page holds them. The temporary plan is complete and removed.

#### A5. A principle for language decisions

Every deviation from Pascal or Ada is either justified or caught by a dedicated diagnostic. Prefer
one spelling per concept: alternatives and aliases confuse an agent more than a human. This is a
principle for future decisions, not a proposal to change the current language.

## Shared wins for humans and agents

### Problem

The human wants to read formatted, colorful code and clear messages. The agent wants correct
examples and errors it can act on without opening files. Some changes serve both at once.

### Wishes

#### S1. Compile every documentation example

**Status: implemented.** The [FPAS checker](../../tools/doc-examples) extracts the
actual handbook and root README examples, writes isolated projects under
`.temp-data/`, and checks them with the current compiler. The normal FPAS and
Cargo test suites run it automatically. Fragment contexts, explicit execution,
expected diagnostic codes, companion files and reports are documented in
[Checking documentation examples](../pascal/tools/documentation-examples.md).

The [inventory](documentation-example-inventory.md) and
[block list](documentation-example-blocks.md) retain the historical baseline.
The checker validates compilation and explicitly selected runtime behavior;
prose, signature outlines and schematic notation still need review.

#### S2. Diagnostics with source excerpt and caret

**Status: implemented, verified and independently reviewed.** Text diagnostics
show one bounded source line and caret markers, with automatic terminal colors
and nonempty `NO_COLOR` support. Compilation snapshots or verified source hashes
keep excerpts tied to the bytecode, including precompiled tests and isolated
workers. Unicode sequence widths, four-cell tab stops, control escaping and
100-cell cropping share one renderer. JSON output keeps its existing contract.

The permanent [diagnostics handbook](../pascal/tools/diagnostics.md#text-output)
owns the output rules. Renderer, real CLI and Unix pseudoterminal tests cover
alignment, source provenance, color policy and worker forwarding. The temporary
S2 plan is complete and removed.

#### S3. Declaration comments as the single source

**Status: implemented.** A contiguous block of `//` lines directly before a declaration is its
Markdown documentation ([`comments.md`](../pascal/language/basics/comments.md)). The language
server shows it on hover and `fpas doc` ([A4](#a4-api-lookup-from-the-cli)) prints the same text
through the same language-service code. One comment, two readers: the human on hover, the agent
on the command line. The comment before a `Std` unit header names its handbook page.

### Deliberately left out

- **Changing the fence tag from `pascal` to `fpas`.** The `pascal` tag nudges the agent slightly
  toward Pascal habits, but GitHub does not know `fpas` and would show the examples without color.
  Small gain for the agent, visible loss for the human. The comparison table in
  [A1](#a1-one-compact-language-reference) fixes the underlying problem better.
- **Formatting.** `fpas fmt` exists and `AGENTS.md` already requires it. Nothing missing.

## Debugging

### Problem

A human at a debugger steps, looks, and pokes. Every step is cheap for a human and expensive for an
agent. Driving a running interactive program — pressing keys until the interesting state shows up —
is the worst case. The agent cannot reliably reach the same state twice, and it burns its budget
getting there.

What an agent wants instead: **run once, then ask questions.** Inputs go in as data, results come
out as structured data, and every run can be repeated exactly.

### Wishes

#### D1. Automatic failure dump

**Status: implemented and verified.** With
`fpas run --failure-dump PATH`, an escaping panic or runtime error writes one
bounded JSON document: the original error, the failing task's call stack and
frame values, globals, and the state and available frames of all relevant tasks.
Execution keeps the ordinary concurrent scheduler; handled task failures create
no report, and publication is atomic to a fresh path.

The permanent [runtime failure reports](../pascal/tools/failure-dump.md) page
owns the CLI contract, JSON schema and bounds. Rust embedders use
`Vm::set_failure_observer`, described under
[diagnostics](../pascal/tools/diagnostics.md). VM, CLI and FPAS runner tests
cover capture, task collection, publication and lifecycle. The
[local measurements](../bench/failure-dump.md#local-measurements-2026-10-15)
show no successful-run cost beyond timer resolution for scalar, call and global
work, a small per-task cost for spawn/wait, bounded report costs, and live-peer
collection far below its deadline. The temporary plan is complete and removed.

#### D2. Record, replay, and query

A human uses the program normally; the VM records everything that enters from outside — key and
console events, network messages, time readings, random values. When the bug shows up, the
recording file *is* the bug report. The agent replays it headless and deterministically, as often
as needed, and queries the run instead of stepping through it:

- "Where was `Total` last written before it went wrong?"
- "Which arguments did `Parse` receive?"
- "Which task wrote to this channel first?"

This is known as omniscient or time-travel debugging (prior art: rr, Pernosco).

#### D3. One-shot batch runs

**Status: implemented.** The agent chose the command contract under the user's
delegation. The permanent [batch debugging page](../pascal/tools/debugger-batch.md)
owns its usage, JSON schema, exit codes and limits:

```text
fpas debug prog.fpas --break main.fpas:42 --print Total --print 'Items[Selected]'
```

One command runs to the first breakpoint, runtime failure or normal completion,
returns one bounded JSON snapshot and exits. Batch is the default `debug` mode;
explicit DAP and JSONL sessions remain available. Automatic task/stack/value
capture is supplemented by repeatable, non-executing inspection expressions.
Existing input scripts supply console lines and queued key characters. A
parent-owned worker enforces invocation-wide limits, including blocked native
work. Real CLI tests cover target forms, first-stop state, task/frame selection,
scripted input, non-executing expressions, structured failures, escaped-byte
budgets and contained native processes. Independent review findings were fixed
and verified. The temporary implementation plan is complete and removed.

**JSONL decision: keep it; removal deferred.** After implementation, verification
and independent review, the user declined removal and kept that work deferred.
Keep the protocol, CLI options, documentation and tests. There is no pending
removal step; reopening it requires a new explicit user request.
See [Decision: keep JSONL](#decision-keep-jsonl).

Record/replay and seeded scheduling remain separate unscheduled wishes.
The implemented global tracepoint subset is documented under
[batch tracepoints](../pascal/tools/debugger-tracepoints.md).

#### D4. Reproducible task scheduling

Start the scheduler from a seed so a race happens the same way on every run. A failing run reports
its seed; the agent reruns that exact interleaving.

#### D5. Tracepoints

**Status: bounded global subset implemented and independently reviewed.**
The agent chose the first-delivery contract under the user's delegation. The
permanent [tracepoint page](../pascal/tools/debugger-tracepoints.md) owns usage,
JSON fields, observation semantics and bounds:

```text
fpas debug app.fpasprj --trace Total --trace State.Selected
```

One bounded JSON result retains initialization and consecutive committed scalar
changes with typed old/new values, actual writing location/function/task and
explicit coverage. Tracepoints keep running to the existing batch stop/end or
an explicit resource limit. Per-commit observation covers `var` writes, callbacks
and record replacement without changing stopping data-breakpoint semantics.
Nominal scalar names survive source compilation and portable image storage.

The first delivery watches mutable globals and fixed stored record-field paths
ending in scalar values. Mutable globals belong to the main task, so events
normally name task 0. VM and real-process tests cover equality/backtracking,
metadata identities, typed capture, event/value/work/escaped-byte limits,
structured failures and worker containment. Independent review findings for
NaN comparison and bounded ambiguous binding were fixed with regression tests.

Local variables, parameters, captures, indexed selectors and whole aggregates
remain unscheduled D5 extensions. The unrestricted wish is not fully implemented.
JSONL/DAP and their existing behavior remain supported. The completed temporary
plan is removed after final verification.

### What already exists

- A debug recording, but it **rejects** nondeterministic intrinsics (`Std.Random`, `Std.Time`,
  `Std.Fs`, `Std.Net`, …) instead of capturing their results. There is no replay. For D2 the
  recording would have to capture those results and feed them back during replay.
- `ScreenSnapshot`, which exposes the TUI screen as data, so a headless replay can still "see" what
  was displayed.
- A deterministic debug scheduler with a manual clock, a good base for D2 and D4.
- One-shot `fpas debug` with automatic state capture and non-executing `--print`
  expressions (D3); normal-run failure reports (D1).
- JSONL with `--commands` remains available for scripted sessions.
- Data breakpoints and logpoints (parts of D5).

### What the agent rarely needs

Changing values at runtime, forced return, frame restart, and hot reload. They are good tools for
humans exploring interactively. An agent would rather change the code and run again — cheaper and
easier to reason about. They stay, because the human still likes having a say.

### Decision: keep JSONL

The debugger retains two session protocols on one shared engine: DAP for editors
and JSONL for scripts. D1 failure reports and D3 batch commands provide the
agent workflow without a session protocol. Replay/query remain separate ideas.

After D3 was implemented, tested and independently reviewed, the user was asked
whether JSONL should be removed and decided to keep it. `fpas debug --protocol
jsonl`, `--commands`, the [JSONL contract](../pascal/tools/debugger-jsonl.md) and
the existing protocol tests remain supported. Removal remains deferred; no
removal or coverage migration is planned. Reconsidering it requires a new explicit
user request and is outside the source-query contract.

## Code navigation

### Problem

The agent does not use the LSP. Completion, hover popups, snippets, and semantic highlighting are
editor features for humans. Diagnostics already come from `fpas check --diagnostics json`. For
navigation the agent uses text search and reads files, which has limits in FPAS:

- **Case-insensitive identifiers.** `total`, `Total`, and `TOTAL` are the same name, so a search must
  ignore case and then also hits comments and strings.
- **Name resolution.** Qualified imports, aliases, and locals that shadow globals mean the text
  matches, but the search cannot tell which declaration a use refers to.
- **Types.** The type of an expression at a given position is often not written anywhere.

### Wishes

`fpas query` commands reuse `fpas-language-service` resolution and semantic
metadata, with one JSON document per invocation. The editor keeps using LSP.

- **N1. References — implemented.** `fpas query refs src/Main.fpas:12:5` finds
  resolved project usages with separate target identity, bounded snapshot inputs,
  source hashes, deadlines and explicit coverage.
- **N2. Definition — implemented.** `fpas query definition src/Main.fpas:12:5` returns
  the source declaration of the resolved symbol.
- **N3. Type — implemented.** `fpas query type src/Main.fpas:12:5` returns the
  checked type, category and source range of the selected expression or value
  binding, including generic call results, named labels and chain prefixes.
- **N4. Rename — implemented.** `fpas query rename src/Main.fpas:12:5 NewName`
  validates and applies project-wide source edits; `--dry-run` returns the edit
  list without writing. Rename uses the shared language service's binding checks,
  validates the edited sources, and protects sources outside the selected root
  and the standard library.

The permanent [source-query reference](../pascal/tools/source-queries.md) owns the
position syntax, project selection, JSON schema, and failure behavior of N1–N4.

## Language note

None of this changes the FPAS language. It is runtime, tooling, and documentation work; principle
[A5](#a5-a-principle-for-language-decisions) only guides future decisions. Functional style helps on
its own: when program logic has the shape `state + event → new state`, an agent can load a saved
state, apply an event sequence, and assert on the result without any debugger.
