# Implemented multiple sources and units

The compiler/CLI and Project SDK accept multiple FPAS source files in one
compilation. Each file declares one qualified program or unit. An executable
requires exactly one program and any number of units; a library requires one or
more units and no program. The output is one ordinary CLR assembly, not one DLL
per source. See the [Units example](../../examples/units/README.md) for a complete
three-file project.

## Binding and visibility

All source container names and routine signatures are collected before binding
any routine body. Imports can therefore target a unit appearing later in the
source list, and public routines can call each other across cyclic unit imports.
Names are case-sensitive. The unit/program header defines identity; filenames
and source-list ordering do not define dependencies or metadata ownership.

`uses Demo.Numbers as Numbers;` imports a unit from the current compilation or
the corresponding public CLR type from a supplied reference. Source imports do
not acquire files, projects, or packages. Call `Numbers.SumTo(10)` explicitly;
an import does not expose its routines as unqualified names.

Routines are private unless marked public. Private routines can be called within
their own unit, including through a self-unit alias, and cannot be called from
another source unit/program. Alias names are local to the importing file and
cannot be shadowed there. Same-named routines in different units are independent.
Programs cannot be imported as units. Duplicate qualified source containers and
source-container identities colliding with referenced CLR types are diagnosed.

The emitted assembly has a static CLR container per program/unit, with the
existing namespace/name mapping and public/private method visibility. Empty
units emit empty containers. Calls between source units target the correct
MethodDef; referenced-assembly calls use ordinary metadata references. Unit
containers each carry non-null string context for .NET consumers.

Sources are ordered by qualified container name for binding/emission. Identical
source texts and references produce identical PE bytes even if file paths and
source/reference input order change. Diagnostics retain the path/line/column of
the source containing the error, including reference lookups in helper files.
Reference input failures use the first source path in ordinal path order.

## CLI and SDK

After building the toolchain and supplying the installed net10.0 reference path:

```sh
dotnet run --project src/Fpas.Cli --no-build -- compile examples/units/main.fpas examples/units/lib/calculations.fpas examples/units/lib/messages.fpas --output .temp-data/units/Units.dll --reference-dir "$FPAS_REFERENCE_DIR"
dotnet .temp-data/units/Units.dll
```

One `-` may read source from stdin alongside files. Duplicate canonical source
paths or multiple stdin entries fail as CLI argument errors. No selected source
may coincide with the assembly/runtime-configuration output. JSON response files
retain the same argument contract for long source/reference lists. UTF-8 decoding
and compilation failures preserve existing output files.

For `.fpasproj`, standard Compile items and default `**/*.fpas` globs select all
sources, including subdirectories. The task passes the complete selected list
through a response file. Its fingerprint tracks paths/content as well as the
unit set, so editing, adding, or removing a selected file invalidates compilation.
No project-per-unit or explicit source ordering is required.

New diagnostics are FP2005 for duplicate source containers, FP3011 for an
inaccessible private routine, and FP3012 for importing a program. FP2004 now
checks the entire compilation's program count; FP4002 also diagnoses a source
container conflicting with referenced metadata. Existing name/type/call and
structured MSBuild diagnostics are preserved.

## Verification and remaining scope

Unit tests inspect method ownership and empty containers, deterministic images,
program counts, source/reference conflicts, visibility, and helper-file locations.
CLI tests execute mutually recursive units with a stdin program in different
orders, check secondary-source errors/encoding, and protect inputs/old outputs.
SDK tests build the checked-in Units project and a multi-unit library in the
C# → FPAS → C# fixture, plus an FPAS consumer of that library through CLR metadata.
They verify edits, added/removed units, and a missing
import after deleting a required source. Existing arithmetic, Mandelbrot, clean/
rebuild, package-reference, and mixed-project tests remain applicable.

This implements file-level units and routines. [CLR object support](clr-objects-and-resources.md)
also permits namespace aliases and imported/nullable types in routine signatures.
Arrays and closed CLR generic types are supported; see [collections](arrays-and-collections.md).
[Callable values, closures, and explicit generic CLR calls](functions-and-closures.md) also work.
[Records](records.md), [unions](unions-and-matching.md), and
[generic source routines](generic-routines.md) also work across units/projects.
Global fields/initializers, generic source type declarations, and
nested routines still await their compiler implementations. There are no unit
initialization/finalization blocks or per-unit compilation caches.
