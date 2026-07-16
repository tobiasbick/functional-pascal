# Std.Version

`Std.Version` provides the versions published with the installed Functional Pascal standard library.

```pascal
program ShowVersion;

uses Std.Console, Std.Version;

begin
  WriteLn(CompilerVersion);
  WriteLn(LibraryVersion)
end.
```

## Quick reference

| Name | Type | Description |
|---|---|---|
| `CompilerVersion` | `string` | Version of the bundled FPAS compiler release. |
| `LibraryVersion` | `string` | Version of the bundled FPAS source standard library. |

## Constants

### `CompilerVersion`

The compiler release version as a string.

### `LibraryVersion`

The version of the installed standard-library sources as a string.

## Implementation (contributors)

The unit is a trusted FPAS source file at `lib/Std/Version.fpas`, exported by `lib/stdlib.fpasprj`. Building `fpas` copies `lib/` beside the executable; the CLI validates and links the selected source manifest when the unit is imported.

## See also

- [Standard library index](README.md)
- [CLI](../program-structure/cli.md)
