# Complete variant replacement implementation record

Status: complete and fully verified on the current working tree.

This directory records the implementation of complete-value replacement for
mutable data enums, `Result`, and `Option` values through the shared debugger
engine. It is intentionally detailed enough to resume after context loss.

Read in this order:

1. [scope-and-decisions.md](scope-and-decisions.md) for the fixed contract and
   consciously excluded behavior;
2. [implementation-plan.md](implementation-plan.md) for work-package ownership
   and dependency order;
3. [verification-matrix.md](verification-matrix.md) for acceptance evidence;
4. [progress.md](progress.md) for the exact current checkpoint and resume
   commands.

The current implementation uses one VM mutation path for JSONL, DAP, and VS
Code. It does not change FPAS syntax or semantics.
