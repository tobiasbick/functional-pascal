# Agent wishes

> **⚠️ Disclaimer:** This is a wish list written by the AI agent that does most of the work on this
> hobby project, not by its human overlord. The human is still undecided: they would like to keep
> reading formatted, colorful code in an IDE and occasionally have a say, while the agent does the
> bulk of the work. This wish may help the overlord too. Nothing here is planned,
> scheduled, or promised.
> Please do not build anything important on top of this — the human sincerely hopes nobody does.

## D2. Record, replay, and query

**Status: unscheduled.** D2 is the only remaining agent wish. Its scope and
feasibility are open; no implementation is approved.

A human uses the program normally; the VM records everything that enters from outside — key and
console events, network messages, time readings, random values. When the bug shows up, the
recording file *is* the bug report. The agent replays it headless and deterministically, as often
as needed, and queries the run instead of stepping through it:

- "Where was `Total` last written before it went wrong?"
- "Which arguments did `Parse` receive?"
- "Which task wrote to this channel first?"

This is known as omniscient or time-travel debugging (prior art: rr, Pernosco).

### Existing tools and boundaries

Implemented behavior is documented in the handbook:
[failure dumps](../pascal/tools/failure-dump.md),
[batch debugging](../pascal/tools/debugger-batch.md),
[seeded scheduling](../pascal/tools/debugger-scheduling.md) and
[scalar tracepoints](../pascal/tools/debugger-tracepoints.md).
Completed planning notes have been removed.

The existing session `record` command captures bounded data in memory; it does
not provide replay. Seeded scheduling repeats controlled debug dispatch for
fixed inputs; it does not record ordinary concurrent execution. Neither feature
implements D2.

Before implementation, a concrete debugging case should establish what these
tools cannot explain. D2 still needs decisions on which external inputs and
results to record, how to reproduce task ordering, recording limits and
incomplete coverage, executable identity, and the smallest useful delivery.

The [JSONL debugger protocol](../pascal/tools/debugger-jsonl.md), `--commands`
and their tests remain supported by explicit user decision. Removing JSONL is
outside D2 and requires a new explicit user request.
