# `Std.Path`

Pure path manipulation without filesystem access. This page is the full API for the unit.

```pascal
program Example;

uses Std.Console as Console;
uses Std.Path as Path;

begin
  Console.WriteLn(Path.BaseName(Path.Normalize('dir/nested/../file.txt')));
end program;
```

`Std.Path` works on path strings only. It does not read the filesystem, resolve the current working directory, or check whether paths exist.


## Importing and names

Import with `uses Std.Path as Path;`. Access every exported member through `Path`, for example `Path.Join(...)`. Imports open no short names.

---

## Quick reference

Requires `uses Std.Path as Path;`.

| Kind | Name | Notes |
|------|------|-------|
| function | `Join(Segments: array of (string)): string` | joins segments with the platform path separator |
| function | `BaseName(Path: string): string` | returns the final path component |
| function | `DirName(Path: string): string` | returns the parent path without the final component |
| function | `Extension(Path: string): string` | returns the final extension without a leading dot |
| function | `Normalize(Path: string): string` | normalizes separators and `.` / `..` components |

---

## Platform behavior

- Separator normalization follows the host platform. On Windows, `\` is the primary separator; `/` is also accepted in many paths. On Unix, `/` is used.
- `Normalize` does not access the filesystem. It only rewrites the path string.
- `Join` with an empty array returns `''`.
- If a later `Join` segment is an absolute path (host rules), it replaces the path built so far — the same behavior as Rust `PathBuf::push`. Prefer relative segments when concatenating under a root.
- `BaseName`, `DirName`, and `Extension` follow the same parsing rules as Rust's `std::path::Path` on the host platform.

---

## `function Join(Segments: array of (string)): string`

Joins path segments in order using the platform separator.

An absolute segment replaces earlier segments (host `PathBuf::push` semantics):

```pascal
uses Std.Console as Console;
uses Std.Path as Path;

// Unix example: result is '/etc/hosts', not 'home/etc/hosts'
Console.WriteLn(Path.Join(['home', '/etc/hosts']));
```

```pascal
uses Std.Console as Console;
uses Std.Path as Path;

const Parts: array of (string) := ['src', 'main', 'app.txt'];
Console.WriteLn(Path.BaseName(Path.Join(Parts)));
```

---

## `function BaseName(Path: string): string`

Returns the final component of `Path`.

```pascal
uses Std.Console as Console;
uses Std.Path as Path;

Console.WriteLn(Path.BaseName('dir/nested/file.txt')); // file.txt
```

Trailing separators follow host `std::path::Path` rules and may differ between Windows and Unix.

---

## `function DirName(Path: string): string`

Returns the parent path without the final component.

```pascal
uses Std.Console as Console;
uses Std.Path as Path;

Console.WriteLn(Path.DirName('dir/nested/file.txt'));  // dir/nested
Console.WriteLn(Path.DirName('file.txt'));             // ''
```

---

## `function Extension(Path: string): string`

Returns the final extension without a leading dot.

```pascal
uses Std.Console as Console;
uses Std.Path as Path;

Console.WriteLn(Path.Extension('archive.tar.gz'));  // gz
Console.WriteLn(Path.Extension('README'));            // ''
```

---

## `function Normalize(Path: string): string`

Normalizes separators and collapses `.` and `..` components without touching the filesystem.

Normalization preserves the path's root and prefix. On Windows, an absolute drive path
keeps the separator after its drive letter; a drive-relative path stays drive-relative.
Parent components cannot climb above a rooted path's root, including a Windows UNC share.

```pascal
uses Std.Console as Console;
uses Std.Path as Path;

// Windows examples:
Console.WriteLn(Path.Normalize('D:/projects/demo'));       // D:\projects\demo
Console.WriteLn(Path.Normalize('D:\projects\..\demo'));   // D:\demo
Console.WriteLn(Path.Normalize('D:projects\..\demo'));     // D:demo (drive-relative)
```

```pascal
uses Std.Console as Console;
uses Std.Path as Path;

Console.WriteLn(Path.Normalize('a/b/../c'));
Console.WriteLn(Path.BaseName(Path.Normalize('dir/nested/../file.txt')));
```

---

## Implementation (contributors)

| Concern | Location |
|---------|----------|
| Runtime execution | [`path.rs`](../../../../crates/fpas-std/src/path.rs) |
| Compiler intrinsic catalog | [`intrinsic_catalog.rs`](../../../../crates/fpas-compiler/src/intrinsic_catalog.rs) |
| Registration | [`std_registry/loaded/path.rs`](../../../../crates/fpas-sema/src/std_registry/loaded/path.rs) |
| Intrinsic ids | [`intrinsic/path.rs`](../../../../crates/fpas-bytecode/src/intrinsic/path.rs) |

## See also

- [Host I/O index](README.md)
- [Standard library index](../README.md)
