# `Std.Fs`

Basic blocking filesystem operations for hosted FPAS programs. This page is the full API for the unit.

```pascal
program Example;

uses Std.Fs as Fs;
uses Std.Results as Results;
uses Std.Tasks as Tasks;

begin
  const ReadJob: task := go Fs.ReadText('input.txt');
  const Text: string := Results.Unwrap(Tasks.Wait(ReadJob));
end program;
```

`Std.Fs` reads and writes host files. Calls are blocking and may run on worker threads when invoked from `go`, but the runtime uses thread-safe Rust filesystem APIs.

**Trust boundary:** FPAS programs run with the same filesystem privileges as the host process. `ReadText`, `WriteText`, `WriteTextAtomic`, and related calls can access any path the OS allows for that process; the runtime does not sandbox paths.

Text reads and writes use UTF-8.

**Resource limits:**

- `ReadText` rejects files larger than 64 MiB (`67_108_864` bytes) and returns `Result.Error(message)`.
- `Glob` rejects patterns that match more than `1_000_000` files and returns `Result.Error(message)`.


## Importing and names

Import with `uses Std.Fs as Fs;`. Access every exported member through `Fs`, for example `Fs.ReadText(...)`. Imports open no short names.

---

## `DeleteFile(Path: string): result of (boolean, string)`

Deletes one file entry and returns `Result.Ok(true)`. Missing paths, directories, permission failures, and other OS errors return `Result.Error(message)`. It does not recursively delete directories. On Windows and POSIX, a symbolic link is removed without deleting its target; directory links follow the platform file-removal rules and may be rejected. Open files can be unlinked on POSIX, while Windows may reject deletion when a handle does not permit delete sharing. Error messages come from the host OS.

## Quick reference

Requires `uses Std.Fs as Fs;`.

| Kind | Name | Notes |
|------|------|-------|
| function | `ReadText(Path: string): Result of (string, string)` | reads UTF-8 text |
| function | `WriteText(Path: string; Text: string): Result of (boolean, string)` | writes UTF-8 text, returns `Result.Ok(true)` |
| function | `WriteTextAtomic(Path: string; Text: string): Result of (boolean, string)` | publishes complete UTF-8 text through a same-directory temporary file |
| function | `DeleteFile(Path: string): Result of (boolean, string)` | removes one file entry, returns `Result.Ok(true)` |
| function | `Exists(Path: string): boolean` | `true` when the path exists |
| function | `IsFile(Path: string): boolean` | `true` for a regular file |
| function | `IsDir(Path: string): boolean` | `true` for a directory |
| function | `CreateDir(Path: string): Result of (boolean, string)` | creates one directory, returns `Result.Ok(true)` |
| function | `CreateDirAll(Path: string): Result of (boolean, string)` | creates a directory and missing parents; an existing directory is `Result.Ok(true)` |
| function | `Glob(Pattern: string): Result of (array of (string), string)` | expands a glob pattern to matching file paths |

Fallible operations return `Result.Error(message)` with a host error string instead of raising a runtime panic.

---

## Blocking and concurrency

Filesystem calls block the thread that executes them. When a call runs inside `go`, it blocks that worker thread only. Combine `go ReadText(...)` or `go WriteText(...)` with `Std.Tasks.Wait` for task-based file workflows.

---

## `function ReadText(Path: string): Result of (string, string)`

Reads the entire file at `Path` as UTF-8 text.

Files larger than 64 MiB return `Result.Error(message)` instead of loading into memory.

```pascal
uses Std.Console as Console;
uses Std.Fs as Fs;
uses Std.Results as Results;

const Content: result of (string, string) := Fs.ReadText('notes.txt');
if Results.IsOk(Content) then
  Console.WriteLn(Results.Unwrap(Content));
end if;
```

---

## `function WriteText(Path: string; Text: string): Result of (boolean, string)`

Writes UTF-8 text to `Path`, creating or replacing the file.

```pascal
uses Std.Console as Console;
uses Std.Fs as Fs;
uses Std.Results as Results;

if Results.IsOk(Fs.WriteText('out.txt', 'hello')) then
  Console.WriteLn('written');
end if;
```

---

## `function WriteTextAtomic(Path: string; Text: string): Result of (boolean, string)`

Writes all UTF-8 bytes to a collision-resistant temporary sibling, flushes that
file, and then atomically replaces `Path`. The previous file remains at `Path`
until the replacement commits. A failure returns `Result.Error(message)`, preserves an
existing destination, and removes the temporary file owned by that call.
Callers therefore never observe a successfully published partially written
file.

```pascal
uses Std.Console as Console;
uses Std.Fs as Fs;

case Fs.WriteTextAtomic('note.note', EncodedNote) of
  when Result.Ok(const Written):
    Console.WriteLn('saved');
  when Result.Error(const Message):
    Console.WriteLn(Message);
end case;
```

Publication uses the host's same-directory atomic replacement primitive on
Unix and Windows. It does not rename the previous destination to a separate
backup and does not remove stale sibling files that it does not own.

---

## `function Exists(Path: string): boolean`

Returns `true` when the host filesystem reports that `Path` exists.

```pascal
uses Std.Console as Console;
uses Std.Fs as Fs;

if Fs.Exists('config.json') then
  Console.WriteLn('config is present');
end if;
```

---

## `function IsFile(Path: string): boolean`

Returns `true` when `Path` exists and is a regular file.

```pascal
uses Std.Console as Console;
uses Std.Fs as Fs;

Console.WriteLn(Fs.IsFile('data.txt'));
```

---

## `function IsDir(Path: string): boolean`

Returns `true` when `Path` exists and is a directory.

```pascal
uses Std.Console as Console;
uses Std.Fs as Fs;

Console.WriteLn(Fs.IsDir('src'));
```

---

## `function CreateDir(Path: string): Result of (boolean, string)`

Creates a single directory at `Path`. Parent directories must already exist. An existing entry at `Path`, including an existing directory, returns `Result.Error(message)`.

```pascal
uses Std.Console as Console;
uses Std.Fs as Fs;
uses Std.Results as Results;

if Results.IsOk(Fs.CreateDir('build/output')) then
  Console.WriteLn('directory created');
end if;
```

---

## `function CreateDirAll(Path: string): Result of (boolean, string)`

Creates the directory at `Path` together with every missing parent directory and returns `Result.Ok(true)`. The call is idempotent: when `Path` already is a directory, including one created concurrently by another task or process, it also returns `Result.Ok(true)`. A path component that exists but is not a directory, permission failures, and other OS errors return `Result.Error(message)`. Directories created before a failure are not removed.

```pascal
uses Std.Console as Console;
uses Std.Fs as Fs;

case Fs.CreateDirAll('build/output/logs') of
  when Result.Ok(const Created):
    Console.WriteLn('directory ready');
  when Result.Error(const Message):
    Console.WriteLn('cannot create directory: ' + Message);
end case;
```

---

## `function Glob(Pattern: string): Result of (array of (string), string)`

Expands `Pattern` against the host filesystem and returns every matching **file** path in stable sorted order. Directory entries are never included.

```pascal
uses Std.Console as Console;
uses Std.Fs as Fs;
uses Std.Arrays as Arrays;

case Fs.Glob('src/**/*.fpas') of
  when Result.Ok(const Paths):
    begin
      Console.WriteLn(Arrays.Length(Paths));
    end;
  when Result.Error(const Message):
    begin
      Console.WriteLn(Message);
    end;
end case;
```

Behavior:

- Patterns use the same glob syntax as project `[sources].include` entries (`*`, `?`, `**`, `[...]`).
- Relative patterns are evaluated from the current working directory of the FPAS process.
- Repository FPAS tests and demos that create files for globbing or I/O round-trips write under `.temp-data/` (gitignored) when run from the repository root. See [`Std.Test`](../testing/test.md).
- A plain file path without glob metacharacters returns `Result.Ok([Path])` when that file exists, otherwise `Result.Ok([])`.
- A valid pattern with no file matches returns `Result.Ok([])` rather than an error.
- Invalid pattern syntax or filesystem failures return `Result.Error(message)`.
- Patterns that match more than `1_000_000` files return `Result.Error(message)`.
- Returned paths use `/` separators for deterministic cross-platform ordering.

Platform notes: `Glob` follows the host OS filesystem and the Rust `glob` crate. On Windows, drive-relative patterns and separator normalization follow the same rules as the project loader and `fpas fmt` glob expansion.

---

## Implementation (contributors)

| Concern | Location |
|---------|----------|
| Runtime dispatch | [`fs.rs`](../../../../crates/fpas-std/src/fs.rs) |
| Atomic publication | [`fs/publication.rs`](../../../../crates/fpas-std/src/fs/publication.rs) |
| Bounded reads | [`fs/read.rs`](../../../../crates/fpas-std/src/fs/read.rs) |
| Glob expansion | [`fs/glob.rs`](../../../../crates/fpas-std/src/fs/glob.rs) |
| Compiler intrinsic catalog | [`intrinsic_catalog.rs`](../../../../crates/fpas-compiler/src/intrinsic_catalog.rs) |
| Registration | [`std_registry/loaded/fs.rs`](../../../../crates/fpas-sema/src/std_registry/loaded/fs.rs) |
| Intrinsic ids | [`intrinsic/fs.rs`](../../../../crates/fpas-bytecode/src/intrinsic/fs.rs) |

## See also

- [Host I/O index](README.md)
- [Standard library index](../README.md)
