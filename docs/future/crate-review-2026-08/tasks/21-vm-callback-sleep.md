# Task 21 — Decide Sleep/Yield semantics in synchronous callbacks

Status: decision required
Severity: P1 implementation failure; semantics pending
Difficulty: hard
Language gate: yes
Depends on: none

## Reproducer

A synchronous hosted callback inherits its owner's nonzero `task_id` in
`vm/callback_call.rs`. If it executes `Sleep` or `Yield`, it detaches callback state under the same
ID as the still-running owner and can fail with `Root register execution suspended unexpectedly`.
The internal failure is a defect; the replacement behavior is not specified.

## Contract gap

[`Std.Time`](../../../pascal/std/host/time.md) promises that Sleep in a spawned task suspends that
task cooperatively. Hosted callbacks such as `Std.Array.Map` execute synchronously, but current docs
do not say whether a callback suspension suspends its entire owner or blocks the current worker.

## Options for user agreement

1. **Suspend the owner cooperatively:** preserve the general spawned-task Sleep guarantee. This
   requires resumable nested callback state and is the larger, semantically uniform solution.
2. **Block the current worker for synchronous callbacks:** run the nested worker with a non-schedulable
   callback identity. Simpler, but document the callback exception to cooperative Sleep/Yield.

Do not spawn a new task per callback element and do not reuse the owner's schedulable ID.

## Tests after decision

- A spawned task maps one element through a callback that sleeps/yields, then completes and is
  successfully waited.
- Other task progress matches the selected scheduling rule.
- Callback failure is attributed to the owner call without ghost task IDs.

## Decision record

- Selected option: pending
- Approved by user: pending
