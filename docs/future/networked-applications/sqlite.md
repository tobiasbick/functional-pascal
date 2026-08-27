# Future: SQLite Storage

> Deferred. SQLite access is not currently part of `Std.*`.

FPAS filesystem operations can publish complete text files atomically, but they do not provide
queries, multi-step transactions, constraints, indexes, or coordinated concurrent updates. A first
database module should expose SQLite directly instead of introducing a generic SQL interface with
only one adapter.

## Proposed scope

- An explicit `Std.Sqlite` unit with opaque connection, statement, row, and transaction handles.
- Open read-only, read-write, and create modes with documented path and in-memory behavior.
- Parameter binding for null, Boolean, integer, real, string, and bounded byte-array values.
- Prepared execution and row iteration without constructing SQL through string interpolation.
- Explicit begin, commit, and rollback, including automatic rollback when an owned transaction is
  closed or the VM exits.
- Busy timeout, WAL selection, foreign-key enforcement, and SQLite result-code translation.
- Schema-version helpers sufficient for application-owned migrations without embedding a migration
  framework into the runtime.
- Backup to a consistent destination and integrity-check support.

## Ownership and concurrency

A connection must document whether it is task-bound or internally serialized. Statements, rows,
and transactions belong to their originating connection and cannot outlive it. Transaction
operations must not interleave accidentally through a shared connection.

Pooling should not be introduced until at least one measured workload needs multiple connections.
The first implementation should make one connection safe and predictable before adding another
resource manager.

## Excluded from the first slice

- A database-independent query interface.
- An object-relational mapper or generated models.
- Network database protocols.
- Automatic schema design or application-specific repositories.

## Acceptance requirements

- CRUD, typed parameters, nulls, BLOBs, constraints, and multi-row queries work end to end.
- Commit persists all changes and rollback persists none, including panic and cancellation paths.
- Concurrent callers follow the documented ownership model without lost updates or deadlocks.
- Busy databases, corrupt files, invalid SQL, conversion errors, and exhausted limits return useful
  errors.
- Handles close deterministically and VM teardown leaves no open transaction.
- File-backed databases reopen correctly after a fresh process start.
