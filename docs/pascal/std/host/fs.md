# `Std.Fs`

Basic blocking filesystem operations for hosted FPAS programs. This page is the full API for the unit.

```pascal
program Example;
uses Std.Fs, Std.Result, Std.Task;

begin
  var ReadJob: task := go ReadText('input.txt');
  var Text: string := Std.Result.Unwrap(Std.Task.Wait(ReadJob))
end.
```

`Std.Fs` reads and writes host files. Calls are blocking and may run on worker threads when invoked from `go`, but the runtime uses thread-safe Rust filesystem APIs.

Text reads and writes use UTF-8.


## Importing and names

After `uses Std.Fs;` use **`ReadText`**, **`WriteText`**, **`Exists`**, **`IsFile`**, **`IsDir`**, **`CreateDir`**, or the fully qualified forms such as **`Std.Fs.ReadText`**.

---

## Quick reference

Requires `uses Std.Fs;`.

| Kind | Name | Notes |
|------|------|-------|
| function | `ReadText(Path: string): Result of string, string` | reads UTF-8 text |
| function | `WriteText(Path: string; Text: string): Result of boolean, string` | writes UTF-8 text, returns `Ok(true)` |
| function | `Exists(Path: string): boolean` | `true` when the path exists |
| function | `IsFile(Path: string): boolean` | `true` for a regular file |
| function | `IsDir(Path: string): boolean` | `true` for a directory |
| function | `CreateDir(Path: string): Result of boolean, string` | creates one directory, returns `Ok(true)` |

Fallible operations return `Error(message)` with a host error string instead of raising a runtime panic.

---

## Blocking and concurrency

Filesystem calls block the thread that executes them. When a call runs inside `go`, it blocks that worker thread only. Combine `go ReadText(...)` or `go WriteText(...)` with `Std.Task.Wait` for task-based file workflows.

---

## `function ReadText(Path: string): Result of string, string`

Reads the entire file at `Path` as UTF-8 text.

```pascal
var Content: Result of string, string := ReadText('notes.txt');
if Std.Result.IsOk(Content) then
  WriteLn(Std.Result.Unwrap(Content))
```

---

## `function WriteText(Path: string; Text: string): Result of boolean, string`

Writes UTF-8 text to `Path`, creating or replacing the file.

```pascal
if Std.Result.IsOk(WriteText('out.txt', 'hello')) then
  WriteLn('written')
```

---

## `function Exists(Path: string): boolean`

Returns `true` when the host filesystem reports that `Path` exists.

```pascal
if Exists('config.json') then
  WriteLn('config is present')
```

---

## `function IsFile(Path: string): boolean`

Returns `true` when `Path` exists and is a regular file.

```pascal
WriteLn(IsFile('data.txt'))
```

---

## `function IsDir(Path: string): boolean`

Returns `true` when `Path` exists and is a directory.

```pascal
WriteLn(IsDir('src'))
```

---

## `function CreateDir(Path: string): Result of boolean, string`

Creates a single directory at `Path`. Parent directories must already exist.

```pascal
if Std.Result.IsOk(CreateDir('build/output')) then
  WriteLn('directory created')
```

---

## Implementation (contributors)

| Concern | Location |
|---------|----------|
| Runtime execution | [`fs.rs`](../../../../crates/fpas-std/src/fs.rs) |
| Call lowering | [`std_calls/fs.rs`](../../../../crates/fpas-compiler/src/compiler/std_calls/fs.rs) |
| Registration | [`std_registry/loaded/fs.rs`](../../../../crates/fpas-sema/src/std_registry/loaded/fs.rs) |
| Intrinsic ids | [`intrinsic/fs.rs`](../../../../crates/fpas-bytecode/src/intrinsic/fs.rs) |

## See also

- [Host I/O index](README.md)
- [Standard library index](../README.md)
