# `Std.Path`

Pure path manipulation without filesystem access. This page is the full API for the unit.

```pascal
program Example;
uses Std.Console, Std.Path;
begin
  WriteLn(BaseName(Normalize('dir/nested/../file.txt')))
end.
```

`Std.Path` works on path strings only. It does not read the filesystem, resolve the current working directory, or check whether paths exist.


## Importing and names

After `uses Std.Path;` use **`Join`**, **`BaseName`**, **`DirName`**, **`Extension`**, **`Normalize`**, or the fully qualified forms such as **`Std.Path.Join`**.

---

## Quick reference

Requires `uses Std.Path;`.

| Kind | Name | Notes |
|------|------|-------|
| function | `Join(Segments: array of string): string` | joins segments with the platform path separator |
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

## `function Join(Segments: array of string): string`

Joins path segments in order using the platform separator.

An absolute segment replaces earlier segments (host `PathBuf::push` semantics):

```pascal
{ Unix example: result is '/etc/hosts', not 'home/etc/hosts' }
WriteLn(Join(['home', '/etc/hosts']))
```

```pascal
var Parts: array of string := ['src', 'main', 'app.txt'];
WriteLn(BaseName(Join(Parts)))
```

---

## `function BaseName(Path: string): string`

Returns the final component of `Path`.

```pascal
WriteLn(BaseName('dir/nested/file.txt'))  { file.txt }
```

Trailing separators follow host `std::path::Path` rules and may differ between Windows and Unix.

---

## `function DirName(Path: string): string`

Returns the parent path without the final component.

```pascal
WriteLn(DirName('dir/nested/file.txt'))  { dir/nested }
WriteLn(DirName('file.txt'))             { '' }
```

---

## `function Extension(Path: string): string`

Returns the final extension without a leading dot.

```pascal
WriteLn(Extension('archive.tar.gz'))  { gz }
WriteLn(Extension('README'))            { '' }
```

---

## `function Normalize(Path: string): string`

Normalizes separators and collapses `.` and `..` components without touching the filesystem.

```pascal
WriteLn(Normalize('a/b/../c'))
WriteLn(BaseName(Normalize('dir/nested/../file.txt')))
```

---

## Implementation (contributors)

| Concern | Location |
|---------|----------|
| Runtime execution | [`path.rs`](../../../../crates/fpas-std/src/path.rs) |
| Call lowering | [`std_calls/path.rs`](../../../../crates/fpas-compiler/src/compiler/std_calls/path.rs) |
| Registration | [`std_registry/loaded/path.rs`](../../../../crates/fpas-sema/src/std_registry/loaded/path.rs) |
| Intrinsic ids | [`intrinsic/path.rs`](../../../../crates/fpas-bytecode/src/intrinsic/path.rs) |

## See also

- [Host I/O index](README.md)
- [Standard library index](../README.md)
