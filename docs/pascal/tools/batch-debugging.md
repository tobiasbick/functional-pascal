# Batch debugging

Build the executable separately, then pass its DLL to `fpas debug`. Keep its
matching portable PDB, recorded sources and runtime dependencies available.
The command uses the packaged CLR debugger; it does not build, restore or
download a backend.

This page describes one-shot batch capture. Repeated `--tracepoint` capture and
`diagnose failure` use the same core/report family; see
[diagnostic commands](cli-diagnostics.md#tracepoints). `fpas schema debug` exports
the report's structural JSON Schema.

After the [local SDK setup](../../build/development.md#build-an-fpas-application),
run this example from the repository root.

```sh
fpas build examples/mandelbrot/Mandelbrot.fpasproj
fpas debug examples/mandelbrot/bin/Debug/net10.0/Mandelbrot.dll \
  --cwd examples/mandelbrot --break main.fpas:13 --print Iteration
fpas debug examples/mandelbrot/bin/Debug/net10.0/Mandelbrot.dll --timeout 10 -- first "two words"
```

The default protocol is `batch`. Relative DLL and explicit `--cwd` paths resolve
against the invocation directory. Default cwd is the DLL directory. Source paths
in repeatable `--break path:line[:column]` and the `--stdin` file resolve against
the selected cwd. Source coordinates are one-based UTF-16 positions. Equal
breakpoints are deduplicated in request order; numeric suffixes are parsed from
the right, preserving Windows drive letters. Target arguments follow `--` and
retain individual elements, including empty strings.

The debugger holds the target at entry, configures breakpoints and unhandled
exceptions, then resumes once. The first requested breakpoint or unhandled
exception supplies one all-stop snapshot. Every `--print` expression reads that
stop through the [passive evaluator](../../architecture/debugger-expressions.md).
The command terminates its owned target after capture. It does not continue after
the observation. A normal exit preserves output and the observed target exit
code; runtime inspection after exit is unavailable. Missing all requested
breakpoints fails the observation.

Target stdin defaults to EOF. `--stdin file` delivers at most 1 MiB of file bytes
and closes input. Writing runs independently of inspection/output; a target that
does not read can still be stopped. Command stdin is never consumed. Programs
perform their ordinary filesystem/network effects; retrying runs them again.
There is no interactive input or rollback.

## Report

Stdout contains exactly one UTF-8 JSON object and a newline. Ordinary errors use
the same report; stderr is empty. Help/version use normal CLI text. Target stdout
and stderr are escaped report data. Select UTF-8 output in the program when exact
Unicode text matters; invalid byte sequences are replaced and flagged.

Schema version 1 contains:

| Field | Meaning |
| --- | --- |
| `tracepoints` | Ordered detached observations for tracepoint mode; empty for one-shot capture. |
| `outcome` | `breakpoint`, `exception`, `completed`, `limit`, `cancelled`, `error`. |
| `exitCode`, `targetExitCode`, `termination` | CLI result, separately observed natural target exit or null, and termination cause. Debugger-requested termination does not invent a natural target failure. |
| `stop`, `breakpoints` | Actual source/thread stop and requested/bound/rejected/pending/hit breakpoint facts. |
| `threads`, `frames`, `bindings`, `expressions` | Detached stopped context, exact typed values and independent expression failures. Frame indexes and object aliases belong only to this report. |
| `exception` | Passive type, stored message, source and throw stack, without target calls. |
| `output` | Sequenced streams, total/retained original byte counts, truncation and encoding loss. Capture order is observed pipe order, not a global program-write ordering guarantee. |
| `capture`, `limits`, `issues` | Completeness, omission reasons, effective bounds and stable stage/code/message errors. |
| `toolchain` | Debugger/shim versions and hashes, host runtime, target/PDB hashes and target module identity. |

Values preserve type, kind, status, scalar text, raw UTF-16 where needed, array
rank/bounds, children, cycle aliases and omissions. Integer scalar text is exact,
including long/ulong; floating special values remain explicit text. No session,
stop, live frame/value handle or native module receipt escapes into the report.
Requested expressions precede automatic binding capture. Automatic clipping
does not fail a fulfilled observation; a failed explicit value does.

| Exit | Meaning |
| --- | --- |
| 0 | Fulfilled observation or zero natural exit without requested inspection. |
| 1 | Setup/debugger failure. |
| 2 | Invalid arguments or expression syntax. |
| 3 | Exception, nonzero target exit, missed breakpoint or failed requested value. |
| 4 | Deadline or captured-output limit. |
| 130 | Cancellation. |

Precedence is cancellation, limit, setup/debugger failure, observation failure,
success. The actual outcome remains independently reported.

## Bounds and shutdown

`--timeout` defaults to 30 seconds and accepts 1..3600. The deadline starts before
preparation. At most 5 seconds (one fifth for short invocations) is reserved for
cleanup; shutdown phases share one monotonic deadline. Cancellation uses the
same owned cleanup. Hard process termination/reaping is mandatory even after the
grace expires; an OS failure to reap has a separate maximum five-second failure
guard and is reported as cleanup failure rather than leaving a target running.

There are at most 64 breakpoints and 64 expressions; each expression has at most
8192 UTF-16 code units. Target argv has at most 1024 elements and 65536 code units.
The Linux installed `dotnet` executable path cannot contain a literal quote or
end in a backslash; use another path to the host. Target DLL paths and arguments
support quotes/backslashes through the platform's shim argument encoding.

`--output-limit` defaults to and cannot exceed 1048576 original stream bytes.
Reaching the positive limit ends execution, including an exact-sized final write.
Prefix retention preserves code point boundaries and exposes all omitted bytes.
The complete escaped JSON plus newline is separately bounded at 8 MiB.

Snapshots retain at most 64 threads, 32 frames per thread and 256 frames total;
4096 values across requested and automatic observations; depth 8, 64 immediate
children and 4096 Unicode scalars per string. Omissions are explicit. Stop facts
and requested observations take priority when reducing an oversized report.
An externally killed host or an unwritable report stream cannot guarantee a
complete JSON report.

## See also

- [Debugging](debugging.md)
- [Passive expressions](../../architecture/debugger-expressions.md)
- [Toolchain environment](environment.md)
