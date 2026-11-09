# Agent wishes

This wish list was restored from the Rust-era catalogue and reviewed against the
current CLR toolchain. Only deferred wishes remain. No implementation is
currently planned; revisit an item when a concrete failure case justifies it.

## Recovery and review

The complete Rust-era catalogue is available in
[b5661da6e](https://github.com/tobiasbick/functional-pascal/blob/b5661da6e80969de682e60d97f88a7447bffde1b/docs/future/agent-wishes.md).
Commit `58d7e28aa` later removed completed wishes and retained only D2.
Commit `433bf6546` deleted the remaining file during the CLR rewrite.

The remaining wishes retain their original IDs and goals, while Rust-only
commands, obsolete links and old implementation reports are omitted. The Rust
status does not establish whether the CLR toolchain meets a wish. Current
references are starting points for that review, not declarations that an item
is missing or complete.

Completed wishes have been removed; implemented contracts belong in their
owning references. Language changes still require explicit approval.

## Principle: protocols for humans, commands for agents

Editors benefit from long-lived LSP and DAP sessions. An agent often benefits
from one command returning one bounded JSON document, with explicit inputs,
output, failures and coverage. Both interfaces should share their analysis or
debugging engine rather than duplicate it.

Human-facing editor and debugger workflows remain useful. These wishes do not
authorize removing them. The Rust version also retained JSONL by explicit user
decision; that history does not establish a JSONL implementation in the CLR
toolchain or make its removal part of this review.

## Debugging

The goal is to reach an interesting state reproducibly and inspect useful data
with few interactions. For state-and-event logic, a saved state and event sequence
may explain a problem without a debugger.

### D2. Record, replay, and query

Status: deferred until a concrete failure case justifies recording and replay
beyond the existing diagnostic tools. No implementation is currently planned.

Record external inputs such as keyboard events, network messages, clock readings
and random values. A recording becomes a reproducible bug report: replay it
headlessly and ask questions about the run, for example:

- Where was `Total` last written before it became incorrect?
- Which arguments did `Parse` receive?
- Which task wrote to this channel first?

This is the omniscient or time-travel debugging idea associated with tools such
as rr and Pernosco. It was the only remaining unscheduled wish before the CLR
rewrite. Before considering implementation, identify a concrete case that existing
tools cannot explain. Input coverage, task ordering, executable identity,
recording limits and the smallest useful delivery remain design questions.

### D4. Reproducible task scheduling

Status: deferred until a concrete concurrency failure justifies controlled
scheduling. Use targeted tests with explicit task coordination in the meantime;
no general-purpose scheduler is currently planned.

Explore task interleavings in a repeatable way, with a seed or another explicit
schedule description attached to the result. State which tasks and effects are
controlled. The Rust implementation's bounded scheduler does not establish an
equivalent guarantee for arbitrary .NET tasks or external I/O.

### D5. Tracepoints

Status: deferred for write-by-write change history until a concrete failure case
justifies it. Keep the existing source-line tracepoints for bounded observations;
they do not establish a complete history or identify every writing location.

Keep running while recording changes to selected values, with writing source
location, task or thread identity, and bounded old/new values. Define selector
coverage and lifetimes for globals, locals, fields, array positions and captures.
Distinguish changed-value history from periodic observations and stopping data
breakpoints; do not assume they provide the same evidence.

Current starting point: [CLI tracepoints](../pascal/tools/cli-diagnostics.md#tracepoints).
