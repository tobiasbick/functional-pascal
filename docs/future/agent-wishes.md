# Agent wishes

Proposals for making FPAS easier and more reliable to use when LLM agents
perform most development work. Prioritize fewer correction cycles, overlooked
dependencies and manual translations between .NET APIs and FPAS source.

These are proposals, not approved implementation work or language changes.
Syntax changes require explicit user approval. Debugging wishes remain deferred
until a concrete failure case justifies further work.

## Tooling and authoring

### T2. Semantic refactorings beyond rename

**Priority: highest**

Extend the [checked refactoring workflow](../pascal/tools/refactoring.md).

5. Add `extract-routine` with checked data and control flow.

Operate on resolved symbols and bindings. Produce reviewable plans, reject stale
inputs and validate resulting source against every affected owner/reference
context. Keep CLI and editor clients on the same semantic core.

Report semantic decisions explicitly, including values needed for new arguments
and behavior needed by an implementation. Never invent application behavior or
fill incomplete members with misleading defaults. This prevents missed callers
and edits to unrelated same-named symbols.

### T4. Limited type inference for local constants

**Priority: exploratory**

Explore omitting a local constant's annotation when its initializer determines
an unambiguous type. Illustrative proposed syntax:

```pascal
const Content := ReadContent(Path);
```

This form is not current FPAS syntax. Keep explicit types on public signatures,
parameters and record fields; explicit local annotations remain available.
Require annotations for untyped null and other context-dependent expressions.

The potential benefit is avoiding repeated types, especially nested generic
.NET types. The tradeoff is that an explicit annotation detects an unintended
callee return-type change, while inference can propagate it silently. Evaluate
this separately from the more direct correctness benefits of templates and
semantic refactorings.

### T5. Small, discoverable, executable recipes

**Status: Deferred.**

Connect the handbook, agent reference, compiler tests and examples into directly
discoverable minimal recipes: generic callbacks, nullable CLR results,
exhaustive variant matching, protocol implementation, `ref`/`out` calls and
asynchronous resource cleanup.

The [documentation checker](../pascal/tools/documentation-examples.md) already
compiles examples in explicit contexts. Build on that verification rather than
introducing another example-checking mechanism. Make the checked recipe easy to
find from the relevant API, rule or diagnostic, with its imports, project
references and dependency context. For difficult rules, pair valid examples
with rejection examples and expected diagnostic codes.

This lets agents reuse a current verified pattern without assembling it from
multiple reference sections or borrowing another language's syntax.

## Properties to preserve

- Clear construct endings such as `end function`.
- Explicit types at API and data-model boundaries.
- One canonical spelling for each language construct.
- Structured diagnostics and machine-readable tool results.
- Explicit distinctions between verified facts, incomplete information and
  unsupported features.
- Maintain the [agent reference](../pascal/agents/README.md) as the entrypoint
  for the language and the [entire public CLI](../pascal/agents/cli.md).
  Update the command index when a command or subcommand changes. Include its
  purpose and link the detailed contract. Keep the entrypoint short and measure
  the combined reference with the [token budget check](../pascal/agents/maintenance.md#budget-and-content).

These properties support work on small source excerpts without requiring the
whole project in context.

## Implementation order and evaluation

When T5 resumes, connect the [API usage templates](../pascal/tools/api-documentation.md#generate-checked-fpas-usage)
to checked recipes. Local inference remains a separate syntax
proposal requiring evaluation and user approval.

Evaluate repeatable tasks: use an unfamiliar .NET API, change a function signature
and its callers, implement a protocol and reproduce a language rule in a minimal
example. Measure correction cycles, manual documentation lookups, missed
dependencies and program correctness. Generating supported syntax and updating
bound references are the expected mechanical benefits; broader productivity
claims need evidence from these tasks.

## Deferred debugging and analysis

No implementation is currently planned for the following wishes.

### D2. Record, replay, and query

Record external inputs such as keyboard events, network messages, clock readings
and random values. Replay a recording headlessly and query the run, for example
for the last write to a value, a call's arguments or the order of channel writes.

Before considering implementation, identify a concrete case that existing tools
cannot explain. Input coverage, task ordering, executable identity, recording
limits and the smallest useful delivery remain design questions.

### D4. Reproducible task scheduling

Explore task interleavings in a repeatable way, with a seed or another explicit
schedule description attached to the result. State which tasks and effects are
controlled, including coverage limits for .NET tasks and external I/O. Use
targeted tests with explicit task coordination until a concrete concurrency
failure justifies a general-purpose scheduler.

### D5. Tracepoints

Write-by-write change history is deferred until a concrete failure case justifies
it. The existing [source-line tracepoints](../pascal/tools/cli-diagnostics.md#tracepoints)
provide bounded observations; they do not identify every writing location.

For change history, record selected values with the writing source location,
task or thread identity, and bounded old/new values. Define selector coverage and
lifetimes for globals, locals, fields, array positions and captures. Distinguish
changed-value history from periodic observations and stopping data breakpoints.
