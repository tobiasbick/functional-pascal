# Agent wishes

These debugging wishes are deferred until a concrete failure case justifies
further work. No implementation is currently planned.

## D2. Record, replay, and query

Record external inputs such as keyboard events, network messages, clock readings
and random values. Replay a recording headlessly and query the run, for example
for the last write to a value, a call's arguments or the order of channel writes.

Before considering implementation, identify a concrete case that existing tools
cannot explain. Input coverage, task ordering, executable identity, recording
limits and the smallest useful delivery remain design questions.

## D4. Reproducible task scheduling

Explore task interleavings in a repeatable way, with a seed or another explicit
schedule description attached to the result. State which tasks and effects are
controlled, including coverage limits for .NET tasks and external I/O. Use
targeted tests with explicit task coordination until a concrete concurrency
failure justifies a general-purpose scheduler.

## D5. Tracepoints

Write-by-write change history is deferred until a concrete failure case justifies
it. The existing [source-line tracepoints](../pascal/tools/cli-diagnostics.md#tracepoints)
provide bounded observations; they do not identify every writing location.

For change history, record selected values with the writing source location,
task or thread identity, and bounded old/new values. Define selector coverage and
lifetimes for globals, locals, fields, array positions and captures. Distinguish
changed-value history from periodic observations and stopping data breakpoints.
