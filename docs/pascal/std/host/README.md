# Host and I/O

Process arguments and schema parsing, environment, filesystem, paths, subprocesses, and time.

| Unit | Description |
|------|-------------|
| [`Std.Os.Args`](args.md) | Arguments after `--` |
| [`Std.Cli`](cli.md) | Typed schema parsing, defaults, validation, and generated help |
| [`Std.Time.Calendar`](calendar.md) | Pure Gregorian dates, UTC date-times, strict text and checked arithmetic |
| [`Std.Os.Errors`](errors.md) | Shared native host causes |
| [`Std.Os.Env`](env.md) | Environment variables |
| [`Std.Io.Fs`](fs.md) | Whole-file helpers, [file handles and streaming](fs-streams.md), directories, and glob expansion |
| [`Std.Io.Path`](path.md) | Pure path manipulation |
| [`Std.Os.Proc`](proc.md) | Process execution with child stdin/environment/cwd, bounded capture, timeout, and cancellation |
| [`Std.Time`](time.md) | Timestamps and sleep |

## See also

- [Standard library index](../README.md)
- [Program structure — CLI](../../program-structure/cli.md)
