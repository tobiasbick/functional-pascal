# Agent wishes

> **⚠️ Disclaimer:** This is a wish list written by the AI agent that does most of the work on this
> hobby project, not by its human overlord. The human is still undecided: they would like to keep
> reading formatted, colorful code in an IDE and occasionally have a say, while the agent does the
> bulk of the work. A few wishes would please the overlord too. Nothing here is planned, scheduled,
> or promised. Please do not build anything important on top of this — the human sincerely hopes
> nobody does.

## Status

Idea only. No implementation plan. Human-facing tools stay: the DAP debugger
([`debugger.md`](../pascal/tools/debugger.md)) and the language server
([`editor-integration.md`](../pascal/tools/editor-integration.md)). The JSONL debugger protocol
would be replaced; see [Consequence: JSONL would go away](#consequence-jsonl-would-go-away).

Each wish has an ID: **A** for authoring, **S** for shared wins, **D** for debugging, **N** for code
navigation.

## Where to start

Most value for the least work. Effort is a rough guess, not an estimate.

| Wish | Why first | Effort |
| --- | --- | --- |
| [A1](#a1-one-compact-language-reference) Compact language reference | Helps the agent with every single task | small |
| [S1](#s1-compile-every-documentation-example) Compile every documentation example | Wrong examples teach the agent wrong FPAS | medium |
| [D1](#d1-automatic-failure-dump) Automatic failure dump | Solves most bugs without a debugger | medium |
| [S2](#s2-diagnostics-with-source-excerpt-and-caret) Diagnostics with source excerpt and caret | Both notice it every day | medium |
| [A2](#a2-diagnostics-for-pascal-and-ada-habits) Diagnostics for Pascal and Ada habits | Biggest lever in the long run | large, ongoing |

Everything else can wait until these exist.

## Principle: protocols for humans, commands for agents

Editors need long-lived sessions: DAP for debugging, LSP for code intelligence. An agent works
differently. Every interaction costs a tool call, a round trip, and context, so it wants
**one command in, one JSON document out**, repeatable exactly.

The codebase already separates logic from protocol: the debugger engine and VM sit below `dap`, and
`fpas-language-service` sits below `fpas-lsp`. Agent commands can call that logic directly from the
CLI, the way `fpas check --diagnostics json` already does. No new protocol, no duplicated logic.

| Concern | Humans | Agents |
| --- | --- | --- |
| Diagnostics | LSP in the editor, terminal text ([S2](#s2-diagnostics-with-source-excerpt-and-caret)) | `fpas check --diagnostics json` (exists), fix-its ([A3](#a3-fix-its-as-data)) |
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

`docs/pascal/` has about 155 pages and roughly 89,000 words — too much to load, so the agent
searches and misses things. Wanted: one file of about 5–10k tokens with every construct and a
minimal example, plus a table "Pascal / Delphi / Ada writes it like this → FPAS writes it like
this". A test compiles every example in the file so it cannot drift. Today the `fpas-authoring`
skill only links to the individual pages.

#### A2. Diagnostics for Pascal and Ada habits

When the agent writes Delphi or Ada syntax, the compiler recognizes it and answers "FPAS writes
this as: …". Build the catalog systematically: let an agent write programs without the docs, log
every error, and add a targeted diagnostic per entry. `native_migration_hint` already does this for
some cases.

#### A3. Fix-its as data

Diagnostics carry hints as prose. A structured, machine-applicable replacement field plus
`fpas check --fix` would let the agent fix simple mistakes directly, like rustfix and
`clippy --fix`.

#### A4. API lookup from the CLI

`fpas doc Std.Fs.ReadText` or `fpas doc string.TrimLeft` returns the signature, a short
description, and an example. Units have generated declarations in `lib/api/`; methods on built-in
types such as `S.TrimLeft()` exist only in the Rust catalog and have no file the agent can find.
For user code, `fpas doc` reads the same declaration comments the editor shows on hover
([S3](#s3-declaration-comments-as-the-single-source)).

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

The docs contain about 450 fenced `pascal` blocks. Tests compile selected ones, for example the
project examples in [`projects.md`](../pascal/program-structure/projects.md), but not all of them.
For the human, the docs are guaranteed to be correct. For the agent, the docs replace training data:
every wrong example is learned and repeated. Fragments that cannot compile on their own, such as a
lone expression, need a marker in the fence info string; GitHub uses only the first word for
highlighting, so `pascal` keeps its colors.

#### S2. Diagnostics with source excerpt and caret

Today a text diagnostic is `path:line:column: error[CODE]: message`, followed by `help:` lines. It
shows no source line, no caret under the error, and no color. Wanted, in the style of rustc: the
offending line with a `^^^` marker, colored when the output is a terminal, honoring `NO_COLOR`. The
human sees at a glance where the error is; the agent sees the faulty code without opening the file.
JSON output stays as it is.

#### S3. Declaration comments as the single source

A contiguous block of `//` lines directly before a declaration is already its Markdown
documentation, and the language server shows it on hover
([`comments.md`](../pascal/language/basics/comments.md)). `fpas doc`
([A4](#a4-api-lookup-from-the-cli)) would print the same comments. One comment, two readers: the
human on hover, the agent on the command line.

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

On a panic or runtime error, `fpas run` writes one JSON document: call stack, locals of every
frame, and the state of every task. Most bugs end here without starting a debugger at all.

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

For example `fpas debug prog.fpas --break main.fpas:42 --print Total,Items --until panic`: one
command, one JSON result, no session handshake. JSONL with `--commands` already comes close but
needs a hand-written request file.

#### D4. Reproducible task scheduling

Start the scheduler from a seed so a race happens the same way on every run. A failing run reports
its seed; the agent reruns that exact interleaving.

#### D5. Tracepoints

"Log every change of `X` with line and task, keep running." Today's data breakpoints and logpoints
each cover half of this.

### What already exists

- A debug recording, but it **rejects** nondeterministic intrinsics (`Std.Random`, `Std.Time`,
  `Std.Fs`, `Std.Net`, …) instead of capturing their results. There is no replay. For D2 the
  recording would have to capture those results and feed them back during replay.
- `ScreenSnapshot`, which exposes the TUI screen as data, so a headless replay can still "see" what
  was displayed.
- A deterministic debug scheduler with a manual clock, a good base for D2 and D4.
- JSONL with `--commands` for scripted sessions (partly D3).
- Data breakpoints and logpoints (parts of D5).

### What the agent rarely needs

Changing values at runtime, forced return, frame restart, and hot reload. They are good tools for
humans exploring interactively. An agent would rather change the code and run again — cheaper and
easier to reason about. They stay, because the human still likes having a say.

### Consequence: JSONL would go away

Today the debugger has two session protocols on one shared engine: DAP for editors and JSONL for
scripts. JSONL exists for exactly the job these wishes would do better. If they are built, the split
becomes:

- **DAP for humans** — the IDE debugger, unchanged.
- **CLI commands with JSON output for agents** — failure dump, batch run, replay and query. They call
  the engine and VM directly, like `fpas run`, and need no session protocol.

The JSONL protocol (`fpas debug --protocol jsonl`, `--commands`, and
[`debugger-jsonl.md`](../pascal/tools/debugger-jsonl.md)) would then be removed, together with the
tests that only replay engine behavior through it.

Order matters:

1. Build D1 and D3 first, so agents are never left with DAP only.
2. Before deleting JSONL tests, make sure every case they cover is owned by a VM or DAP test.
3. Remove JSONL.

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

`fpas query` commands backed by `fpas-language-service`, each answering one question as JSON. The
LSP stays for the human; it already uses the same language service, so these commands add CLI entry
points, not new analysis.

- **N1. References** — every use of the symbol at a position: `fpas query refs src/Main.fpas:12:5`.
- **N2. Definition** — where the symbol at a position is declared.
- **N3. Type** — the type of the expression at a position.
- **N4. Rename** — a safe rename across the project; with `--dry-run`, the list of edits only.

## Language note

None of this changes the FPAS language. It is runtime, tooling, and documentation work; principle
[A5](#a5-a-principle-for-language-decisions) only guides future decisions. Functional style helps on
its own: when program logic has the shape `state + event → new state`, an agent can load a saved
state, apply an event sequence, and assert on the result without any debugger.
