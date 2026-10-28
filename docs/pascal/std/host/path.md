# `Std.Path`

Pure path manipulation without filesystem access. This page is the full API for the unit.

```pascal id=example-1
program Example;

uses Std.Console as Console, Std.Path as Path;

begin
  Console.WriteLn(Path.BaseName(Path.Normalize('dir/nested/../file.txt')));
end.
```

`Std.Path` works on path strings only. It does not read the filesystem, resolve the current working directory, or check whether paths exist.


## Importing and names

Import with `uses Std.Path as Path;` and access public symbols as `Path.Name`. User enum variants require `Path.Type.Variant`.

---

## Quick reference

Requires `uses Std.Path as Path;`.

| Kind | Name | Notes |
|------|------|-------|
| function | `Join(Segments: array of string): string` | joins segments with the platform path separator |
| function | `BaseName(Path: string): string` | returns the final path component |
| function | `DirName(Path: string): string` | returns the parent path without the final component |
| function | `Extension(Path: string): string` | returns the final extension without a leading dot |
| function | `Normalize(Path: string): string` | normalizes separators and `.` / `..` components |
| function | `IsAbsolute(Path: string): boolean` | tests whether the path is fully absolute under host rules |
| function | `RelativeTo(Target: string; Base: string): Option of string` | returns the lexical path from `Base` to `Target`, or `None` when indeterminate |
| function | `ChangeExtension(Path: string; Extension: string): Option of string` | replaces the final extension, or `None` for an invalid extension or absent filename |

---

## Platform behavior

- Separator normalization follows the host platform. On Windows, `\` is the primary separator; `/` is also accepted in many paths. On Unix, `/` is used.
- `Normalize` does not access the filesystem. It only rewrites the path string.
- `Join` with an empty array returns `''`.
- If a later `Join` segment is an absolute path (host rules), it replaces the path built so far — the same behavior as Rust `PathBuf::push`. Prefer relative segments when concatenating under a root.
- `BaseName`, `DirName`, and `Extension` follow the same parsing rules as Rust's `std::path::Path` on the host platform.
- `IsAbsolute`, `RelativeTo`, and `ChangeExtension` use those host parsing rules too. On Unix, a backslash is an ordinary filename character.
- Component comparisons follow host Rust path equality. These operations do not apply filesystem-wide case folding or resolve symlinks.

---

## `function Join(Segments: array of string): string`

Joins path segments in order using the platform separator.

An absolute segment replaces earlier segments (host `PathBuf::push` semantics):

```pascal id=example-2 fragment context=body-console-path
// Unix example: result is '/etc/hosts', not 'home/etc/hosts'
Console.WriteLn(Path.Join(['home', '/etc/hosts']));
```

```pascal id=example-3 fragment context=body-console-path
const Parts: array of string := ['src', 'main', 'app.txt'];
Console.WriteLn(Path.BaseName(Path.Join(Parts)));
```

---

## `function BaseName(Path: string): string`

Returns the final component of `Path`.

```pascal id=example-4 fragment context=body-console-path
Console.WriteLn(Path.BaseName('dir/nested/file.txt')); // file.txt
```

Trailing separators follow host `std::path::Path` rules and may differ between Windows and Unix.

---

## `function DirName(Path: string): string`

Returns the parent path without the final component.

```pascal id=example-5 fragment context=body-console-path
Console.WriteLn(Path.DirName('dir/nested/file.txt')); // dir/nested
Console.WriteLn(Path.DirName('file.txt')); // ''
```

---

## `function Extension(Path: string): string`

Returns the final extension without a leading dot.

```pascal id=example-6 fragment context=body-console-path
Console.WriteLn(Path.Extension('archive.tar.gz')); // gz
Console.WriteLn(Path.Extension('README')); // ''
```

---

## `function Normalize(Path: string): string`

Normalizes separators and collapses `.` and `..` components without touching the filesystem.

Normalization preserves the path's root and prefix. On Windows, an absolute drive path
keeps the separator after its drive letter; a drive-relative path stays drive-relative.
Parent components cannot climb above a rooted path's root, including a Windows UNC share.

```pascal id=example-7 fragment context=body-console-path
// Windows examples:
Console.WriteLn(Path.Normalize('D:/projects/demo')); // D:\projects\demo
Console.WriteLn(Path.Normalize('D:\projects\..\demo')); // D:\demo
Console.WriteLn(Path.Normalize('D:projects\..\demo')); // D:demo (drive-relative)
```

```pascal id=example-8 fragment context=body-console-path
Console.WriteLn(Path.Normalize('a/b/../c'));
Console.WriteLn(Path.BaseName(Path.Normalize('dir/nested/../file.txt')));
```

---

## `function IsAbsolute(Path: string): boolean`

Tests whether `Path` is fully absolute on the host platform. Empty strings,
`.` and ordinary relative paths return `false`.

On Unix, a leading `/` makes a path absolute. On Windows, an absolute drive
path such as `D:\projects` or a UNC path such as `\\server\share\projects`
is absolute. A drive-relative path such as `D:projects`, or a root-relative
path such as `\projects`, is not fully absolute.

## `function RelativeTo(Target: string; Base: string): Option of string`

Returns a path from `Base` to `Target`. The target is the first argument.
Both paths are normalized lexically using `Normalize` rules before comparing
their components. Empty paths are treated as `.` for this calculation.
Identical determinate paths produce `Some('.')`.

The result uses the host separator and may contain `..` components. Neither
argument needs to exist. No current directory, filesystem lookup, or symlink
resolution is involved.

`None` means the relationship cannot be established under these rules:

- One argument is absolute and the other relative, or their roots/prefixes are incompatible.
- After matching the common component prefix, an unresolved `..` remains in the base.
- On Windows, either argument is drive-relative or root-relative, including identical paths of those forms. Fully absolute paths on the same drive or UNC share and ordinary relative paths are supported.

Shared leading parent components cancel lexically:
`RelativeTo('../a', '../a')` returns `Some('.')`, and on Unix
`RelativeTo('../file', '../dir')` returns `Some('../file')`. An unmatched
parent component in the target is allowed, so on Unix
`RelativeTo('../file', 'dir')` returns `Some('../../file')`. An unmatched
parent in the base is indeterminate: `RelativeTo('file', '../dir')` is `None`.
Components retain the host's existing case comparison behavior; this does not
ask the filesystem whether differently spelled names refer to the same object.

## `function ChangeExtension(Path: string; Extension: string): Option of string`

Replaces only the final extension. The extension argument has no leading dot:
`ChangeExtension('archive.tar.gz', 'zip')` returns `Some('archive.tar.zip')`.
An empty extension removes the final extension. Multipart extensions such as
`'tar.gz'` are accepted. A dotfile such as `.env` has no existing extension,
so changing it to `'txt'` produces `Some('.env.txt')`.

Returns `None` when the path has no final filename, including an empty path,
root, `.` or `..`. An extension starting with `.`, containing NUL, or containing
a host path separator also returns `None`. Both `/` and `\` are separators on
Windows; only `/` is a separator on Unix.

Final-component parsing follows the same rules as `BaseName`, including
trailing separators: `ChangeExtension('folder/', 'txt')` produces
`Some('folder.txt')`. Other components are not resolved against a current
directory or checked on disk.

The following assertions run on either platform:

```pascal id=example-9 run
program PathHelpers;

uses Std.Path as Path, Std.Test as Test;

begin
  Test.AssertTrue(not Path.IsAbsolute('docs/guide.txt'));
  Test.AssertEquals('guide.txt', Path.RelativeTo('docs/guide.txt', 'docs').Unwrap());
  Test.AssertEquals('.', Path.RelativeTo('docs/./guide.txt', 'docs/guide.txt').Unwrap());
  Test.AssertEquals('.', Path.RelativeTo('', '.').Unwrap());
  Test.AssertEquals('.', Path.RelativeTo('../docs', '../docs').Unwrap());
  Test.AssertTrue(Path.RelativeTo('docs', '../docs') is None);
  Test.AssertEquals('archive.tar.zip', Path.ChangeExtension('archive.tar.gz', 'zip').Unwrap());
  Test.AssertEquals('archive.tar', Path.ChangeExtension('archive.tar.gz', '').Unwrap());
  Test.AssertEquals('.env.txt', Path.ChangeExtension('.env', 'txt').Unwrap());
  Test.AssertTrue(Path.ChangeExtension('notes.txt', '.md') is None);
  Test.AssertTrue(Path.ChangeExtension('.', 'txt') is None);
end.
```

---

## Implementation (contributors)

| Concern | Location |
|---------|----------|
| Runtime execution | [`path.rs`](../../../../crates/fpas-runtime/src/stdlib/path.rs) |
| Compiler intrinsic catalog | [`intrinsic_catalog.rs`](../../../../crates/fpas-compiler/src/intrinsic_catalog.rs) |
| Registration | [`std_registry/loaded/path.rs`](../../../../crates/fpas-sema/src/std_registry/loaded/path.rs) |
| Intrinsic ids | [`intrinsic/path.rs`](../../../../crates/fpas-code/src/intrinsic/path.rs) |

## See also

- [Host I/O index](README.md)
- [Standard library index](../README.md)
