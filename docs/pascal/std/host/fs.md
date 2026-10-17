# `Std.Fs`

Basic blocking filesystem operations for hosted FPAS programs. This page is the full API for the unit.

```pascal id=example-1
program Example;

uses Std.Fs as Fs, Std.Tasks as Tasks;

begin
  scope
    const ReadJob: task of result of (string, string) := go Fs.ReadText('input.txt');
    const Text: string := Tasks.Wait(ReadJob).Unwrap();
  end scope;
end.
```

`Std.Fs` reads and writes host files. Calls are blocking and may run on worker threads when invoked from `go`, but the runtime uses thread-safe Rust filesystem APIs.

**Trust boundary:** FPAS programs run with the same filesystem privileges as the host process. `ReadText`, `WriteText`, `WriteTextAtomic`, and related calls can access any path the OS allows for that process; the runtime does not sandbox paths.

Text reads and writes use UTF-8.

**Resource limits:**

- `ReadText` rejects files larger than 64 MiB (`67_108_864` bytes) and returns `Error(message)`.
- `Glob` rejects patterns that match more than `1_000_000` files and returns `Error(message)`.


## Importing and names

Import with `uses Std.Fs as Fs;` and access public symbols as `Fs.Name`. User enum variants require `Fs.Type.Variant`.

---

## `DeleteFile(Path: string): result of (boolean, string)`

Deletes one file entry and returns `Ok(true)`. Missing paths, directories, permission failures, and other OS errors return `Error(message)`. It does not recursively delete directories. On Windows and POSIX, a symbolic link is removed without deleting its target; directory links follow the platform file-removal rules and may be rejected. Open files can be unlinked on POSIX, while Windows may reject deletion when a handle does not permit delete sharing. Error messages come from the host OS.

## `DeleteDir(Path: string): result of (boolean, string)`

Deletes one empty directory and returns `Ok(true)`. Non-empty directories, missing paths, files, permission failures, and other OS errors return `Error(message)`.

## `DeleteDirAll(Path: string): result of (boolean, string)`

Deletes the directory at `Path` together with all files and subdirectories below it and returns `Ok(true)`. Symbolic links inside the tree are removed without following them. A missing path, a file path, permission failures, and other OS errors return `Error(message)`; entries removed before a failure stay removed.

```pascal id=example-delete-dir-all fragment context=body-console-fs
if Fs.DeleteDirAll('build/output') is Ok(_) then
  Console.WriteLn('output removed');
end if;
```

## `CopyFile(Source: string; Target: string): result of (boolean, string)`

Copies the contents of the file `Source` to `Target`, creating or replacing `Target`, and returns `Ok(true)`. The parent directory of `Target` must exist. Directories, missing sources, and OS errors return `Error(message)`. The copy is not atomic; use `WriteTextAtomic` when readers must never see a partial file.

## `Rename(Source: string; Target: string): result of (boolean, string)`

Renames or moves the file or directory `Source` to `Target` and returns `Ok(true)`. An existing target file is replaced on Unix and Windows; replacing an existing directory follows host rules and may fail. Moving across file systems or volumes returns `Error(message)`.

## `AppendText(Path: string; Text: string): result of (boolean, string)`

Appends the UTF-8 bytes of `Text` to the end of `Path`, creating the file when it is missing, and returns `Ok(true)`. A missing parent directory and OS errors return `Error(message)`.

```pascal id=example-append-text fragment context=body-console-fs
if Fs.AppendText('app.log', 'started') is Error(_) then
  Console.WriteLn('cannot write log');
end if;
```

## Quick reference

Requires `uses Std.Fs as Fs;`.

| Kind | Name | Notes |
|------|------|-------|
| function | `ReadText(Path: string): Result of (string, string)` | reads UTF-8 text |
| function | `WriteText(Path: string; Text: string): Result of (boolean, string)` | writes UTF-8 text, returns `Ok(true)` |
| function | `WriteTextAtomic(Path: string; Text: string): Result of (boolean, string)` | publishes complete UTF-8 text through a same-directory temporary file |
| function | `AppendText(Path: string; Text: string): Result of (boolean, string)` | appends UTF-8 text, creating a missing file |
| function | `DeleteFile(Path: string): Result of (boolean, string)` | removes one file entry, returns `Ok(true)` |
| function | `DeleteDir(Path: string): Result of (boolean, string)` | removes one empty directory, returns `Ok(true)` |
| function | `DeleteDirAll(Path: string): Result of (boolean, string)` | removes a directory and all of its contents |
| function | `CopyFile(Source: string; Target: string): Result of (boolean, string)` | copies one file, replacing the target |
| function | `Rename(Source: string; Target: string): Result of (boolean, string)` | renames or moves a file or directory |
| function | `Exists(Path: string): boolean` | `true` when the path exists |
| function | `IsFile(Path: string): boolean` | `true` for a regular file |
| function | `IsDir(Path: string): boolean` | `true` for a directory |
| function | `CreateDir(Path: string): Result of (boolean, string)` | creates one directory, returns `Ok(true)` |
| function | `CreateDirAll(Path: string): Result of (boolean, string)` | creates a directory and missing parents; an existing directory is `Ok(true)` |
| function | `Glob(Pattern: string): Result of (array of string, string)` | expands a glob pattern to matching file paths |
| function | `ReadDir(Path: string): Result of (array of string, string)` | sorted immediate entries, including files and directories |
| function | `ReadDirAll(Path: string): Result of (array of string, string)` | sorted recursive entries below a directory |
| function | `CopyDirAll(Source: string; Target: string): Result of (boolean, string)` | copies a directory tree into the target |
| function | `FileSize(Path: string): Result of (integer, string)` | size of a regular file in bytes |
| function | `ModifiedTime(Path: string): Result of (integer, string)` | last modification in Unix milliseconds |
| function | `IsSymLink(Path: string): boolean` | `true` when the path itself is a symbolic link |
| function | `CurrentDir(): Result of (string, string)` | process working directory |
| function | `SetCurrentDir(Path: string): Result of (boolean, string)` | changes the process working directory |
| function | `TempDir(): string` | host temporary directory |
| function | `CreateTempDir(): Result of (string, string)` | creates a unique temporary directory |
| function | `CreateTempFile(): Result of (string, string)` | creates a unique empty temporary file |

Fallible operations return `Error(message)` with a host error string instead of raising a runtime panic.

---

## Blocking and concurrency

Filesystem calls block the thread that executes them. When a call runs inside `go`, it blocks that worker thread only. Combine `go ReadText(...)` or `go WriteText(...)` with `Std.Tasks.Wait` for task-based file workflows.

---

## `function ReadText(Path: string): Result of (string, string)`

Reads the entire file at `Path` as UTF-8 text.

Files larger than 64 MiB return `Error(message)` instead of loading into memory.

```pascal id=example-2 fragment context=body-console-fs
const Content: result of (string, string) := Fs.ReadText('notes.txt');
case Content of
  when Ok(const Text):
    Console.WriteLn(Text);
  when Error(_):
    null;
end case;
```

---

## `function WriteText(Path: string; Text: string): Result of (boolean, string)`

Writes UTF-8 text to `Path`, creating or replacing the file.

```pascal id=example-3 fragment context=body-console-fs
if Fs.WriteText('out.txt', 'hello') is Ok(_) then
  Console.WriteLn('written');
end if;
```

---

## `function WriteTextAtomic(Path: string; Text: string): Result of (boolean, string)`

Writes all UTF-8 bytes to a collision-resistant temporary sibling, flushes that
file, and then atomically replaces `Path`. The previous file remains at `Path`
until the replacement commits. A failure returns `Error(message)`, preserves an
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

## `function CreateDir(Path: string): Result of (boolean, string)`

Creates a single directory at `Path`. Parent directories must already exist. An existing entry at `Path`, including an existing directory, returns `Error(message)`.

```pascal id=example-8 fragment context=body-console-fs
if Fs.CreateDir('build/output') is Ok(_) then
  Console.WriteLn('directory created');
end if;
```

---

## `function CreateDirAll(Path: string): Result of (boolean, string)`

Creates the directory at `Path` together with every missing parent directory and returns `Ok(true)`. The call is idempotent: when `Path` already is a directory, including one created concurrently by another task or process, it also returns `Ok(true)`. A path component that exists but is not a directory, permission failures, and other OS errors return `Error(message)`. Directories created before a failure are not removed.

```pascal id=example-9 fragment context=body-console-fs
case Fs.CreateDirAll('build/output/logs') of
  when Ok(const Created):
    Console.WriteLn('directory ready');
  when Error(const Message):
    Console.WriteLn('cannot create directory: ' + Message);
end case;
```

---

## `function Glob(Pattern: string): Result of (array of string, string)`

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
- Invalid pattern syntax or filesystem failures return `Error(message)`.
- Patterns that match more than `1_000_000` files return `Error(message)`.
- Returned paths use `/` separators for deterministic cross-platform ordering.

Platform notes: `Glob` follows the host OS filesystem and the Rust `glob` crate. On Windows, drive-relative patterns and separator normalization follow the same rules as the project loader and `fpas fmt` glob expansion.

---

## `function ReadDir(Path: string): Result of (array of string, string)`

Returns all immediate entry paths, including files, empty directories, and links,
in stable lexical order with `/` separators. Relative inputs produce relative
entry paths. The operation does not recurse or follow child directory links.
An empty directory returns `Ok([])`. Missing directories, file paths, read errors,
and non-UTF-8 entry paths return `Error(message)` rather than an empty result.
A call is limited to 1,000,000 entries. `Glob` retains its file-only behavior.
The call blocks its executing thread and may be run in a `go` task.

---

## `function ReadDirAll(Path: string): Result of (array of string, string)`

Returns every entry below the directory `Path`, recursively, including files,
directories, and links, in stable lexical order with `/` separators. Directory
links are listed but not followed. The same limits and errors as `ReadDir`
apply; the 1,000,000-entry limit covers the whole tree.

## `function CopyDirAll(Source: string; Target: string): Result of (boolean, string)`

Copies the directory tree `Source` into `Target` and returns `Ok(true)`.
Missing target directories are created, existing files are replaced, and
other existing entries in `Target` are kept. A `Source` that is not a
directory, a `Target` inside `Source`, a symbolic link in the tree, and OS
errors return `Error(message)`; entries copied before a failure remain.

---

## Metadata

### `function FileSize(Path: string): Result of (integer, string)`

Returns the size in bytes of the regular file at `Path`. Directories, missing
paths, and OS errors return `Error(message)`.

### `function ModifiedTime(Path: string): Result of (integer, string)`

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

### `function CurrentDir(): Result of (string, string)`

Returns the absolute current working directory of the process with `/`
separators. Relative paths in every `Std.Fs` call resolve against it.

### `function SetCurrentDir(Path: string): Result of (boolean, string)`

Changes the current working directory and returns `Ok(true)`. The working
directory belongs to the whole process: the change affects all tasks and every
later relative path. Prefer absolute paths built with `Std.Path` in concurrent
programs.

### `function TempDir(): string`

Returns the host temporary directory with `/` separators. The directory is not
created or checked.

### `function CreateTempDir(): Result of (string, string)`

Creates a new, empty, uniquely named directory inside `TempDir()` and returns
its path. The caller owns it and removes it, for example with `DeleteDirAll`.

### `function CreateTempFile(): Result of (string, string)`

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
| Runtime dispatch | [`fs.rs`](../../../../crates/fpas-std/src/fs.rs) |
| Atomic publication | [`fs/publication.rs`](../../../../crates/fpas-std/src/fs/publication.rs) |
| Bounded reads | [`fs/read.rs`](../../../../crates/fpas-std/src/fs/read.rs) |
| Directory enumeration | [`fs/directory.rs`](../../../../crates/fpas-std/src/fs/directory.rs) |
| Glob expansion | [`fs/glob.rs`](../../../../crates/fpas-std/src/fs/glob.rs) |
| Appending | [`fs/append.rs`](../../../../crates/fpas-std/src/fs/append.rs) |
| Metadata queries | [`fs/metadata.rs`](../../../../crates/fpas-std/src/fs/metadata.rs) |
| Temporary locations | [`fs/temp.rs`](../../../../crates/fpas-std/src/fs/temp.rs) |
| Recursive listing and copy | [`fs/tree.rs`](../../../../crates/fpas-std/src/fs/tree.rs) |
| Compiler intrinsic catalog | [`intrinsic_catalog.rs`](../../../../crates/fpas-compiler/src/intrinsic_catalog.rs) |
| Registration | [`std_registry/loaded/fs.rs`](../../../../crates/fpas-sema/src/std_registry/loaded/fs.rs) |
| Intrinsic ids | [`intrinsic/fs.rs`](../../../../crates/fpas-bytecode/src/intrinsic/fs.rs) |

## See also

- [Host I/O index](README.md)
- [Standard library index](../README.md)
