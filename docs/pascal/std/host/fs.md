# `Std.Io.Fs`

Filesystem operations for hosted FPAS programs. This page lists the full API;
[file handles and streaming](fs-streams.md) have a separate detailed guide.

```pascal id=example-1
program Example;

uses Std.Io.Fs as Fs, Std.Tasks as Tasks;

begin
  scope
    const ReadJob: task of result of (string, Fs.FsError) := go Fs.ReadText('input.txt');
    const Text: string := Tasks.Wait(ReadJob).Unwrap();
  end scope;
end.
```

`Std.Io.Fs` reads and writes host files using thread-safe Rust filesystem APIs.
Handle, whole-file and path operations suspend pool tasks and the all-stop debugger
while bounded native jobs execute. Inline callbacks suspend with their owning task.
Main-program calls and synchronous native callback boundaries execute on their calling
thread. See [handle ownership](fs-streams.md#execution-and-native-ownership).

**Trust boundary:** FPAS programs run with the same filesystem privileges as the host process. `ReadText`, `WriteText`, `WriteTextAtomic`, and related calls can access any path the OS allows for that process; the runtime does not sandbox paths.

Text reads and writes use UTF-8.
`ReadBytes` and `WriteBytes` transfer raw integer byte arrays without text
decoding or newline normalization.

## Execution and cancellation

Suspended filesystem operations share a per-VM budget of 64 native jobs, including results
not yet delivered to FPAS. Functions returning `Result` report saturation as
`FsError.LimitExceeded(Operation, 64)` before starting filesystem work. Plain queries
(`Exists`, `IsFile`, `IsDir`, `IsSymLink`, `TempDir`) wait for admission and retain their
Boolean or string return type. Failure to launch a native worker, or loss of that
worker, is a runtime `FP5027` diagnostic for those plain-return functions.

Native requests and outcomes contain no FPAS values. Conversion to strings, arrays
and typed errors happens on the owning VM task. Pool tasks and the debugger poll
readiness through their existing shared timer. Already dispatched native work may
complete while all-stop execution is paused.

Task abandonment signals and joins its native work. Cancellation is checked between
read/write blocks, directory entries, glob results and recursive-copy entries. An OS
or library call already in progress, including copying one file, recursive deletion
or atomic publication, cannot be forcibly interrupted. Cleanup may wait for it.
Completed filesystem effects are not rolled back.

An unpublished `CreateTempFile` or `CreateTempDir` outcome owns its new path and
removes it on abandonment. After its string result is delivered, the caller owns
cleanup through `DeleteFile` or `DeleteDir`/`DeleteDirAll`. Caller-supplied target
paths are never removed as abandoned-result cleanup. Atomic writes discard an
uncommitted replacement on cancellation and retain the previous target.

**Resource limits:**

- `ReadText` rejects files larger than 64 MiB (`67_108_864` bytes) and returns `Error(Cause)`.
- `ReadBytes` and `WriteBytes` accept at most 1 MiB (`1_048_576` bytes) per call.
- File handles allow at most 1024 simultaneous opens; reads, writes, and line content have 1 MiB bounds. See [streaming limits](fs-streams.md#limits-and-errors).
- `Glob` rejects patterns that match more than `1_000_000` files and returns `Error(Cause)`.


## Importing and names

Import with `uses Std.Io.Fs as Fs;` and access public symbols as `Fs.Name`. User enum variants require `Fs.Type.Variant`.

---

## `DeleteFile(Path: string): result of (boolean, Fs.FsError)`

Deletes one file entry and returns `Ok(true)`. Missing paths, directories, permission failures, and other OS errors return `Error(Cause)`. It does not recursively delete directories. On Windows and POSIX, a symbolic link is removed without deleting its target; directory links follow the platform file-removal rules and may be rejected. Open files can be unlinked on POSIX, while Windows may reject deletion when a handle does not permit delete sharing. Error messages come from the host OS.

## `DeleteDir(Path: string): result of (boolean, Fs.FsError)`

Deletes one empty directory and returns `Ok(true)`. Non-empty directories, missing paths, files, permission failures, and other OS errors return `Error(Cause)`.

## `DeleteDirAll(Path: string): result of (boolean, Fs.FsError)`

Deletes the directory at `Path` together with all files and subdirectories below it and returns `Ok(true)`. Symbolic links inside the tree are removed without following them. A missing path, a file path, permission failures, and other OS errors return `Error(Cause)`; entries removed before a failure stay removed.

```pascal id=example-delete-dir-all fragment context=body-console-fs
if Fs.DeleteDirAll('build/output') is Ok(_) then
  Console.WriteLn('output removed');
end if;
```

## `CopyFile(Source: string; Target: string): result of (boolean, Fs.FsError)`

Copies the contents of the file `Source` to `Target`, creating or replacing `Target`, and returns `Ok(true)`. The parent directory of `Target` must exist. Directories, missing sources, and OS errors return `Error(Cause)`. The copy is not atomic; use `WriteTextAtomic` when readers must never see a partial file.

## `Rename(Source: string; Target: string): result of (boolean, Fs.FsError)`

Renames or moves the file or directory `Source` to `Target` and returns `Ok(true)`. An existing target file is replaced on Unix and Windows; replacing an existing directory follows host rules and may fail. Moving across file systems or volumes returns `Error(Cause)`.

## `AppendText(Path: string; Text: string): result of (boolean, Fs.FsError)`

Appends the UTF-8 bytes of `Text` to the end of `Path`, creating the file when it is missing, and returns `Ok(true)`. A missing parent directory and OS errors return `Error(Cause)`.

```pascal id=example-append-text fragment context=body-console-fs
if Fs.AppendText('app.log', 'started') is Error(_) then
  Console.WriteLn('cannot write log');
end if;
```

## Quick reference

Requires `uses Std.Io.Fs as Fs;`.

| Kind | Name | Notes |
|------|------|-------|
| type | `File` | opaque VM-owned file handle; copies share position and close state |
| type | `FileMode` | `Read`, `Create`, `Append`, `ReadWrite` |
| type | `SeekOrigin` | `Start`, `Current`, `FileEnd` |
| function | `Open(Path: string; Mode: FileMode): Result of (File, Fs.FsError)` | opens a regular file; at most 1024 open handles |
| function | `Close(File: File): Result of (boolean, Fs.FsError)` | closes the file for every alias |
| function | `Read(File: File; MaxBytes: integer): Result of (array of integer, Fs.FsError)` | reads up to `MaxBytes`; an empty array is EOF |
| function | `Write(File: File; Bytes: array of integer): Result of (integer, Fs.FsError)` | writes validated bytes and returns the count written, which may be partial |
| function | `ReadLine(File: File): Result of (Option of string, Fs.FsError)` | strict UTF-8 line input; `None` is EOF |
| function | `Seek(File: File; Offset: integer; Origin: SeekOrigin): Result of (integer, Fs.FsError)` | changes the logical byte position and returns it |
| function | `Tell(File: File): Result of (integer, Fs.FsError)` | returns the logical byte position |
| function | `Flush(File: File): Result of (boolean, Fs.FsError)` | flushes output without an fsync guarantee |
| function | `Size(File: File): Result of (integer, Fs.FsError)` | byte length of the open file without moving its position |
| function | `SetLength(File: File; Length: integer): Result of (boolean, Fs.FsError)` | truncates or zero-extends a writable file, preserving its position |
| function | `WriteString(File: File; Text: string): Result of (boolean, Fs.FsError)` | writes complete UTF-8 content, at most 1 MiB |
| function | `WriteLine(File: File; Text: string): Result of (boolean, Fs.FsError)` | writes complete UTF-8 content followed by LF |
| function | `WriteStringWithCancellation(File: File; Text: string; Token: Tasks.CancellationToken): Result of (boolean, Fs.FsError)` | complete text output with cooperative cancellation |
| function | `WriteLineWithCancellation(File: File; Text: string; Token: Tasks.CancellationToken): Result of (boolean, Fs.FsError)` | complete line output with cooperative cancellation |
| function | `ReadWithCancellation(File: File; MaxBytes: integer; Token: Tasks.CancellationToken): Result of (array of integer, Fs.FsError)` | bounded byte read with cooperative cancellation |
| function | `WriteWithCancellation(File: File; Bytes: array of integer; Token: Tasks.CancellationToken): Result of (integer, Fs.FsError)` | validated byte write with cooperative cancellation |
| function | `ReadLineWithCancellation(File: File; Token: Tasks.CancellationToken): Result of (Option of string, Fs.FsError)` | UTF-8 line read with cooperative cancellation |
| function | `ReadText(Path: string): Result of (string, Fs.FsError)` | reads UTF-8 text |
| function | `ReadBytes(Path: string): Result of (array of integer, Fs.FsError)` | reads raw bytes unchanged; at most 1 MiB |
| function | `WriteText(Path: string; Text: string): Result of (boolean, Fs.FsError)` | writes UTF-8 text, returns `Ok(true)` |
| function | `WriteBytes(Path: string; Bytes: array of integer): Result of (boolean, Fs.FsError)` | validates bytes, then creates or replaces a file; at most 1 MiB |
| function | `WriteTextAtomic(Path: string; Text: string): Result of (boolean, Fs.FsError)` | publishes complete UTF-8 text through a same-directory temporary file |
| function | `AppendText(Path: string; Text: string): Result of (boolean, Fs.FsError)` | appends UTF-8 text, creating a missing file |
| function | `DeleteFile(Path: string): Result of (boolean, Fs.FsError)` | removes one file entry, returns `Ok(true)` |
| function | `DeleteDir(Path: string): Result of (boolean, Fs.FsError)` | removes one empty directory, returns `Ok(true)` |
| function | `DeleteDirAll(Path: string): Result of (boolean, Fs.FsError)` | removes a directory and all of its contents |
| function | `CopyFile(Source: string; Target: string): Result of (boolean, Fs.FsError)` | copies one file, replacing the target |
| function | `Rename(Source: string; Target: string): Result of (boolean, Fs.FsError)` | renames or moves a file or directory |
| function | `Exists(Path: string): boolean` | `true` when the path exists |
| function | `IsFile(Path: string): boolean` | `true` for a regular file |
| function | `IsDir(Path: string): boolean` | `true` for a directory |
| function | `CreateDir(Path: string): Result of (boolean, Fs.FsError)` | creates one directory, returns `Ok(true)` |
| function | `CreateDirAll(Path: string): Result of (boolean, Fs.FsError)` | creates a directory and missing parents; an existing directory is `Ok(true)` |
| function | `Glob(Pattern: string): Result of (array of string, Fs.FsError)` | expands a glob pattern to matching file paths |
| function | `ReadDir(Path: string): Result of (array of string, Fs.FsError)` | sorted immediate entries, including files and directories |
| function | `ReadDirAll(Path: string): Result of (array of string, Fs.FsError)` | sorted recursive entries below a directory |
| function | `CopyDirAll(Source: string; Target: string): Result of (boolean, Fs.FsError)` | copies a directory tree into the target |
| function | `FileSize(Path: string): Result of (integer, Fs.FsError)` | size of a regular file in bytes |
| function | `ModifiedTime(Path: string): Result of (integer, Fs.FsError)` | last modification in Unix milliseconds |
| function | `IsSymLink(Path: string): boolean` | `true` when the path itself is a symbolic link |
| function | `CurrentDir(): Result of (string, Fs.FsError)` | process working directory |
| function | `SetCurrentDir(Path: string): Result of (boolean, Fs.FsError)` | changes the process working directory |
| function | `TempDir(): string` | host temporary directory |
| function | `CreateTempDir(): Result of (string, Fs.FsError)` | creates a unique temporary directory |
| function | `CreateTempFile(): Result of (string, Fs.FsError)` | creates a unique empty temporary file |

Fallible operations return `Error(Cause)` with a typed `Fs.FsError`. Host-backed
variants retain the original native `HostError` cause; operational failures do not
raise a runtime panic.

See [File handles and streaming](fs-streams.md) for modes, shared aliases,
buffered positioning, cancellation, and explicit cleanup. Token arguments use
`Std.Tasks.CancellationToken`.

---

## Blocking and concurrency

Filesystem calls block the thread that executes them. When a call runs inside `go`, it blocks that worker thread only. Combine `go ReadText(...)` or `go WriteText(...)` with `Std.Tasks.Wait` for task-based file workflows.
The same task pattern applies to `ReadBytes` and `WriteBytes`.

---

## `function ReadText(Path: string): Result of (string, Fs.FsError)`

Reads the entire file at `Path` as UTF-8 text.

Files larger than 64 MiB return `Error(Cause)` instead of loading into memory.

```pascal id=example-2 fragment context=body-console-fs
const Content: result of (string, Fs.FsError) := Fs.ReadText('notes.txt');
case Content of
  when Ok(const Text):
    Console.WriteLn(Text);
  when Error(_):
    null;
end case;
```

---

## `function WriteText(Path: string; Text: string): Result of (boolean, Fs.FsError)`

Writes UTF-8 text to `Path`, creating or replacing the file.

```pascal id=example-3 fragment context=body-console-fs
if Fs.WriteText('out.txt', 'hello') is Ok(_) then
  Console.WriteLn('written');
end if;
```

---

## `function ReadBytes(Path: string): Result of (array of integer, Fs.FsError)`

Reads the entire file as integer bytes in `0..255`, preserving every byte,
including NUL, non-UTF-8 sequences, and CR/LF bytes. An empty file returns
`Ok([])`. No text decoding or newline normalization takes place.

The limit is **1_048_576 bytes (1 MiB)**. Larger files return
`Error(Fs.FsError.LimitExceeded('Std.Io.Fs.ReadBytes', 1048576))`. Missing files,
permissions, and other host I/O failures use the existing typed `FsError`
variants and preserve their native `HostError` causes. `ReadText` retains its
separate **64 MiB** limit and UTF-8 validation.

## `function WriteBytes(Path: string; Bytes: array of integer): Result of (boolean, Fs.FsError)`

Writes integer bytes unchanged, creating or replacing the file, and returns
`Ok(true)`. Parent directories must already exist. An empty array creates an
empty file or truncates an existing one to zero bytes.

Validation occurs before opening, creating, or truncating the destination:

1. More than **1_048_576 elements** returns
   `Fs.FsError.LimitExceeded('Std.Io.Fs.WriteBytes', 1048576)`.
2. Otherwise, every element must be in `0..255`. The first invalid element
   returns `Fs.FsError.InvalidByte(Offset, Value)`, retaining its zero-based
   array index and supplied integer value.
3. Only after validation are the values converted to bytes and the file written.

These validation failures leave an existing destination unchanged and do not
create an absent destination. The write uses ordinary filesystem write
semantics, as `WriteText` does: a host I/O failure may leave a truncated or
partially written destination. It has no atomic-publication guarantee.

```pascal id=example-binary run
program BinaryFiles;

uses Std.Io.Fs as Fs, Std.Test as Test;

begin
  const FileName := Test.ScratchDir() + '/binary-example.bin';
  Test.AssertTrue(Fs.WriteBytes(FileName, [0, 255, 128, 13, 10]).Unwrap());
  const Bytes := Fs.ReadBytes(FileName).Unwrap();
  Test.AssertEquals(5, Bytes.Length());
  Test.AssertEquals(0, Bytes[0]);
  Test.AssertEquals(255, Bytes[1]);
  Test.AssertEquals(128, Bytes[2]);
  Test.AssertEquals(13, Bytes[3]);
  Test.AssertEquals(10, Bytes[4]);
  Test.AssertTrue(Fs.WriteBytes(FileName, []).Unwrap());
  Test.AssertEquals(0, Fs.ReadBytes(FileName).Unwrap().Length());
  Test.AssertTrue(Fs.DeleteFile(FileName).Unwrap());
end.
```

---

## `function WriteTextAtomic(Path: string; Text: string): Result of (boolean, Fs.FsError)`

Writes all UTF-8 bytes to a collision-resistant temporary sibling, flushes that
file, and then atomically replaces `Path`. The previous file remains at `Path`
until the replacement commits. A failure returns `Error(Cause)`, preserves an
existing destination, and removes the temporary file owned by that call.
Callers therefore never observe a successfully published partially written
file.

```pascal id=example-4 fragment context=atomic-write
case Fs.WriteTextAtomic('note.note', EncodedNote) of
  when Ok(const Written):
    Console.WriteLn('saved');
  when Error(const Message):
    Console.WriteLn(Message);
end case;
```

Publication uses the host's same-directory atomic replacement primitive on
Unix and Windows. It does not rename the previous destination to a separate
backup and does not remove stale sibling files that it does not own.

---

## `function Exists(Path: string): boolean`

Returns `true` when the host filesystem reports that `Path` exists.

```pascal id=example-5 fragment context=body-console-fs
if Fs.Exists('config.json') then
  Console.WriteLn('config is present');
end if;
```

---

## `function IsFile(Path: string): boolean`

Returns `true` when `Path` exists and is a regular file.

```pascal id=example-6 fragment context=body-console-fs
Console.WriteLn(Fs.IsFile('data.txt'));
```

---

## `function IsDir(Path: string): boolean`

Returns `true` when `Path` exists and is a directory.

```pascal id=example-7 fragment context=body-console-fs
Console.WriteLn(Fs.IsDir('src'));
```

---

## `function CreateDir(Path: string): Result of (boolean, Fs.FsError)`

Creates a single directory at `Path`. Parent directories must already exist. An existing entry at `Path`, including an existing directory, returns `Error(Cause)`.

```pascal id=example-8 fragment context=body-console-fs
if Fs.CreateDir('build/output') is Ok(_) then
  Console.WriteLn('directory created');
end if;
```

---

## `function CreateDirAll(Path: string): Result of (boolean, Fs.FsError)`

Creates the directory at `Path` together with every missing parent directory and returns `Ok(true)`. The call is idempotent: when `Path` already is a directory, including one created concurrently by another task or process, it also returns `Ok(true)`. A path component that exists but is not a directory, permission failures, and other OS errors return `Error(Cause)`. Directories created before a failure are not removed.

```pascal id=example-9 fragment context=body-console-fs
case Fs.CreateDirAll('build/output/logs') of
  when Ok(const Created):
    Console.WriteLn('directory ready');
  when Error(const Message):
    Console.WriteLn('cannot create directory: ' + Message.Message());
end case;
```

---

## `function Glob(Pattern: string): Result of (array of string, Fs.FsError)`

Expands `Pattern` against the host filesystem and returns every matching **file** path in stable sorted order. Directory entries are never included.

```pascal id=example-10 fragment context=body-console-fs
case Fs.Glob('src/**/*.fpas') of
  when Ok(const Paths):
    begin
      Console.WriteLn(Paths.Length());
    end;
  when Error(const Message):
    begin
      Console.WriteLn(Message);
    end;
end case;
```

Behavior:

- Patterns use the same glob syntax as project `[sources].include` entries (`*`, `?`, `**`, `[...]`).
- Relative patterns are evaluated from the current working directory of the FPAS process.
- Repository FPAS tests and demos that create files for globbing or I/O round-trips write under `.temp-data/` (gitignored) when run from the repository root. See [`Std.Test`](../testing/test.md).
- A plain file path without glob metacharacters returns `Ok([Path])` when that file exists, otherwise `Ok([])`.
- A valid pattern with no file matches returns `Ok([])` rather than an error.
- Invalid pattern syntax or filesystem failures return `Error(Cause)`.
- Patterns that match more than `1_000_000` files return `Error(Cause)`.
- Returned paths use `/` separators for deterministic cross-platform ordering.

Platform notes: `Glob` follows the host OS filesystem and the Rust `glob` crate. On Windows, drive-relative patterns and separator normalization follow the same rules as the project loader and `fpas fmt` glob expansion.

---

## `function ReadDir(Path: string): Result of (array of string, Fs.FsError)`

Returns all immediate entry paths, including files, empty directories, and links,
in stable lexical order with `/` separators. Relative inputs produce relative
entry paths. The operation does not recurse or follow child directory links.
An empty directory returns `Ok([])`. Missing directories, file paths, read errors,
and non-UTF-8 entry paths return `Error(Cause)` rather than an empty result.
A call is limited to 1,000,000 entries. `Glob` retains its file-only behavior.
The call follows the [filesystem execution contract](#execution-and-cancellation),
releasing a pool worker while its owned native request is pending.

---

## `function ReadDirAll(Path: string): Result of (array of string, Fs.FsError)`

Returns every entry below the directory `Path`, recursively, including files,
directories, and links, in stable lexical order with `/` separators. Directory
links are listed but not followed. The same limits and errors as `ReadDir`
apply; the 1,000,000-entry limit covers the whole tree.

## `function CopyDirAll(Source: string; Target: string): Result of (boolean, Fs.FsError)`

Copies the directory tree `Source` into `Target` and returns `Ok(true)`.
Missing target directories are created, existing files are replaced, and
other existing entries in `Target` are kept. A `Source` that is not a
directory, a `Target` inside `Source`, a symbolic link in the tree, and OS
errors return `Error(Cause)`; entries copied before a failure remain.

The source/target relationship is checked before creating or writing target entries.
Relative and absolute targets are resolved through existing filesystem ancestors,
so aliases of the source itself or its descendants are rejected even when the
remaining target directories do not yet exist.

---

## Metadata

### `function FileSize(Path: string): Result of (integer, Fs.FsError)`

Returns the size in bytes of the regular file at `Path`. Directories, missing
paths, and OS errors return `Error(Cause)`.

### `function ModifiedTime(Path: string): Result of (integer, Fs.FsError)`

Returns the last modification time of the file or directory at `Path` in
milliseconds since the Unix epoch, the same scale as
[`Std.Time.TimestampMillis`](time.md). Resolution follows the host filesystem.

```pascal id=example-metadata fragment context=body-console-fs
case Fs.FileSize('notes.txt') of
  when Ok(const Size):
    Console.WriteLn(Size);
  when Error(const Message):
    Console.WriteLn(Message);
end case;
```

### `function IsSymLink(Path: string): boolean`

Returns `true` when `Path` itself is a symbolic link. Unlike `IsFile` and
`IsDir`, it does not follow the link; a dangling link is still `true`.

---

## Working and temporary directories

### `function CurrentDir(): Result of (string, Fs.FsError)`

Returns the absolute current working directory of the process with `/`
separators. Relative paths in every `Std.Io.Fs` call resolve against it.

### `function SetCurrentDir(Path: string): Result of (boolean, Fs.FsError)`

Changes the current working directory and returns `Ok(true)`. The working
directory belongs to the whole process: the change affects all tasks and every
later relative path. Prefer absolute paths built with `Std.Io.Path` in concurrent
programs.

### `function TempDir(): string`

Returns the host temporary directory with `/` separators. The directory is not
created or checked.

### `function CreateTempDir(): Result of (string, Fs.FsError)`

Creates a new, empty, uniquely named directory inside `TempDir()` and returns
its path. The caller owns it and removes it, for example with `DeleteDirAll`.

### `function CreateTempFile(): Result of (string, Fs.FsError)`

Creates a new, empty, uniquely named file inside `TempDir()` and returns its
path. Creation fails rather than reusing an existing entry. The caller owns the
file and removes it with `DeleteFile`.

```pascal id=example-temp fragment context=body-console-fs
case Fs.CreateTempDir() of
  when Ok(const Dir):
    begin
      Console.WriteLn(Fs.IsDir(Dir));
      Console.WriteLn(Fs.DeleteDirAll(Dir).Unwrap());
    end;
  when Error(const Message):
    Console.WriteLn(Message);
end case;
```

---

## Implementation (contributors)

| Concern | Location |
|---------|----------|
| Runtime dispatch | [`fs.rs`](../../../../crates/fpas-runtime/src/stdlib/fs.rs) |
| Atomic publication | [`fs/publication.rs`](../../../../crates/fpas-runtime/src/stdlib/fs/publication.rs) |
| Bounded reads | [`fs/read.rs`](../../../../crates/fpas-runtime/src/stdlib/fs/read.rs) |
| Directory enumeration | [`fs/directory.rs`](../../../../crates/fpas-runtime/src/stdlib/fs/directory.rs) |
| Glob expansion | [`fs/glob.rs`](../../../../crates/fpas-runtime/src/stdlib/fs/glob.rs) |
| Appending | [`fs/append.rs`](../../../../crates/fpas-runtime/src/stdlib/fs/append.rs) |
| Metadata queries | [`fs/metadata.rs`](../../../../crates/fpas-runtime/src/stdlib/fs/metadata.rs) |
| Temporary locations | [`fs/temp.rs`](../../../../crates/fpas-runtime/src/stdlib/fs/temp.rs) |
| Recursive listing and copy | [`fs/tree.rs`](../../../../crates/fpas-runtime/src/stdlib/fs/tree.rs) |
| Compiler intrinsic catalog | [`intrinsic_catalog.rs`](../../../../crates/fpas-compiler/src/intrinsic_catalog.rs) |
| Registration | [`std_registry/loaded/fs.rs`](../../../../crates/fpas-sema/src/std_registry/loaded/fs.rs) |
| Intrinsic ids | [`intrinsic/fs.rs`](../../../../crates/fpas-code/src/intrinsic/fs.rs) |

## See also

- [Host I/O index](README.md)
- [Standard library index](../README.md)

## Typed failures

Every Result-returning operation uses `FsError`. Native classifications select `FileNotFound(Path, Cause)`, `AccessDenied(Path, Cause)`, `AlreadyExists(Path, Cause)`, `NotDirectory(Path, Cause)`, `IsDirectory(Path, Cause)`, `NotEmpty(Path, Cause)` or `ReadOnly(Path, Cause)`. Other I/O failures use `IoFailure(Cause)`. Each Cause is the original [HostError](errors.md), including the attempted operation, supplied resources and optional native code. Multi-resource operations retain source then target; when the backend does not identify the failed path, the outer variant uses the attempted primary path. No preflight access check replaces the operation.

Validation uses `InvalidArgument(Name, Reason)`, `InvalidPattern(Pattern, Reason)`,
`InvalidEncoding(Path, Detail)`, `InvalidByte(Offset, Value)`, or
`LimitExceeded(Operation, Limit)`. `InvalidByte` reports the first invalid
`WriteBytes` array element; its index starts at zero. Binary size errors name
`Std.Io.Fs.ReadBytes` or `Std.Io.Fs.WriteBytes` and carry the limit `1048576`.
Handle operations also use `Closed(Operation)` and `Cancelled(Operation)`;
their details and bounds are in the [streaming guide](fs-streams.md).
Limits and success values remain as above. Branch on variants; render
`Cause.Message()` when producing text.
