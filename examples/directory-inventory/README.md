# Directory inventory

A small text-file inventory using actual .NET APIs. The program takes a directory
and an optional search pattern, recursively filters paths with LINQ Where<string>,
materializes them in List<string>, sorts them, and uses Select<string, string>
to read and format each file with StreamReader. Files whose basename starts with
a dot are skipped. The row lambda captures the root directory and a shared mutable
total. Rows are enumerated once, so each selected file is read once. Sources and streams are evaluated/opened
once; using const releases each reader and enumeration releases its enumerator.

After building and packaging the local SDK (see the
[bootstrap instructions](../../docs/build/development.md#build-an-fpas-application)):

```sh
dotnet restore examples/directory-inventory/DirectoryInventory.fpasproj
dotnet build examples/directory-inventory/DirectoryInventory.fpasproj --no-restore
dotnet examples/directory-inventory/bin/Debug/net10.0/DirectoryInventory.dll examples/units '*.fpas'
```

Quote paths and shell wildcard patterns. args contains only the supplied program
arguments. The default pattern is '*'; pass a text-file pattern when scanning a
repository containing binaries. Output lists relative paths with line counts,
then file and line totals. Missing/extra arguments return exit code 2; I/O or
access failures return 1. The inventory is a live traversal without a filesystem
snapshot; concurrent changes can fail with an I/O error. Counts use checked int.
