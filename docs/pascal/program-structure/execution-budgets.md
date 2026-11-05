# Execution budgets

`fpas run`, `fpas test` and `fpas debug` accept optional execution-wide bounds:

| Option | Unit | Scope |
| --- | --- | --- |
| `--heap-limit <bytes>` | Positive decimal bytes | Simultaneously retained managed value storage |
| `--instruction-limit <count>` | Positive decimal instructions | Combined bytecode work of all tasks and callbacks |
| `--timeout <seconds>` | Positive decimal seconds | Cooperative execution deadline; tests also retain their outer process deadline |
| `--output-limit <bytes>` | Positive decimal UTF-8 bytes | Combined logical `Console.WriteText`/`WriteLn` output, including newlines |

Ordinary runs have no budgets unless selected. Tests retain their existing
300-second outer timeout. Debuggers retain their existing default instruction,
time, output and input bounds; their managed heap is unlimited unless selected.
Options after `--` are application arguments. Duplicate, missing, zero, negative,
malformed and overflowing values are argument errors.

```sh
fpas run app.fpasprj --heap-limit 67108864 --instruction-limit 10000000 --timeout 10
fpas test tests/ --heap-limit 33554432 --output-limit 1048576
fpas debug app.fpascp --heap-limit 67108864 --protocol jsonl
```

All workers in one VM share an account. Spawning another task or invoking a
callback does not reset a limit. Each test body and each setup/teardown hook has
its own execution account with the same selected configuration. Parallel tests
have separate accounts, and isolated test workers receive the configuration.

## Managed storage

The heap account includes retained string data and scalar indexes, array and
dictionary and channel capacities, byte buffers, records, enums, closure captures, mutable
cells and Option/Result payload storage. Shared bodies are charged once. A
copy-on-write mutation reserves a new body while the original remains charged;
growth reserves additional capacity. Releasing the final owner returns its
reservation. Budgeted allocations bypass the reusable buffer pools so released
storage is not retained there. Garbage collection still follows its normal
ownership rules and collection schedule.

Known bulk-output sizes are checked before allocation. Native results are charged
when adopted into managed values. Temporary Rust allocations, native library
caches and I/O buffers, executable metadata, debugger inspection metadata, VM
stacks, allocator overhead and child processes are outside this heap account.
Stack depth and register slots retain their independent interpreter limits.
This account is not a process RSS quota or a replacement for operating-system
isolation. Use cgroups or Windows Job Objects for a hard process/tree bound.

## Exhaustion and cleanup

An ordinary run reports **FP5029** with the source location and exhausted resource.
The test runner classifies it as a runtime error. Debugger protocols return the
corresponding `heap_limit`, `instruction_limit`, `timeout` or
`output_limit` error. A refusal terminates the execution budget; freeing values
afterward does not restart the failed program.

After the first exhaustion, resource teardown has a shared emergency allowance:
up to 64 KiB of additional managed storage, 100,000 instructions and one second
of cooperative cleanup. The primary exhaustion diagnostic remains preserved.
That allowance cannot make program execution unbounded.

Deadlines are checked at interpreter and supported host-operation checkpoints.
Debugger pauses do not consume active execution time. Native calls that cannot
be interrupted must finish before their ownership can be retired; an outer
process deadline remains useful. Console formatting checks the output budget
before building an oversized request, and publication charges the shared counter
before emitting text. Terminal-control escape sequences and independently
inherited child-process streams are outside the logical output count.

The [resource guard](../../../bench/build-times.md#resource-guard) still supervises
external Cargo compilation. Cargo and Rust tests execute outside the FPAS VM.

## Rust hosts

`fpas_runtime::ExecutionLimits` is the common optional configuration. Configure a
VM or reusable callback session with `with_execution_limits` before starting it.
`execution_usage()` reports current and peak tracked heap storage and the
instruction/output counters enabled by their respective budgets. Returned host
values keep their storage reservation until their last owner releases them.

`DebugExecutionLimits.max_heap_bytes` selects the same managed heap account for
debugger execution, detached calls and stopped-state value edits. The other
debugger fields supply the corresponding instruction/time/output limits.
Debugger expression parsing, inspection and controlled-call limits remain
additional bounds on debugger work.

Managed allocation refusal crosses an internal typed Rust unwind boundary and
becomes a regular diagnostic; unrelated Rust panics propagate unchanged. The
supported Cargo profiles use unwinding. Hosts selecting `panic=abort` cannot
recover from an allocation refusal through that boundary.
