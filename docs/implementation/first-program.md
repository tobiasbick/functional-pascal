# First executable compiler milestone

**Implemented:** a synchronous FPAS program with units, or units-only library,
can be compiled by the C# CLI to a managed DLL. Programs include runtime configuration and execute through
a normal .NET 10 host. Units export public routines for .NET consumers.
This is a small implemented subset of the [language baseline](../pascal/README.md),
not implementation of the complete specification.

## Run the example

Build with a stable .NET 10 SDK:

```sh
dotnet restore Fpas.slnx
dotnet build Fpas.slnx --no-restore
```

Set FPAS_REFERENCE_DIR to the installed .NET 10 framework reference directory.
It has the layout `<dotnet-install>/packs/Microsoft.NETCore.App.Ref/<pack-version>/ref/net10.0`
and contains System.Runtime.dll and System.Console.dll. The compiler receives
explicit metadata inputs; it does not inspect arbitrary assemblies loaded in
its own process. The [implemented MSBuild task](msbuild.md) supplies resolved
references automatically when building a `.fpasproj`.

From the repository root, with that variable set in a POSIX shell:

```sh
dotnet run --project src/Fpas.Cli --no-build -- compile examples/hello/main.fpas --output .temp-data/hello/Hello.dll --reference-dir "$FPAS_REFERENCE_DIR"
dotnet .temp-data/hello/Hello.dll
```

The program prints `Hello from FPAS`. See the checked-in
[source](../../examples/hello/main.fpas). On PowerShell, use
`$env:FPAS_REFERENCE_DIR` for the environment variable.

## Implemented language subset

- UTF-8 source, optional initial BOM, LF/CRLF/CR, `//` comments, and case-sensitive
  keywords/ASCII identifiers. Escaped identifiers use `@`.
- Exactly one `program Qualified.Name; ... begin ... end; end program;` source,
  including an empty body, plus any number of `unit Qualified.Name; ... end unit;`
  sources; library output has one or more units and no program.
  The entry body's `end;` is separate from the program's `end program;`.
- Top-level functions/procedures with explicit parameter and result types, local
  bindings, and `return`. Unit routines may be `public` or `private`; omitted
  visibility is private. Source calls may reference later routine declarations.
  Public routines emit ordinary static CLR methods with named parameters and
  nullable reference metadata where annotated. Source overloads and
  [named nested routines](nested-routines.md) are supported.
- [Synchronous contracts and CLR parameters](contracts-and-parameters.md): callee
  preconditions/postconditions, ref/out/in storage, optional/params parameters,
  and named calls with ordinary CLR metadata.
- Explicit aliases importing public, non-nested CLR types from supplied metadata,
  such as `uses System.Console as Console;`, namespaces such as `uses System.IO as IO;`,
  or units from the same compilation. See [multiple sources](multiple-sources.md)
  for file selection, visibility, and declaration-order rules.
- Local immutable `const Name: T := Expression;` and mutable `var Name: T := Expression;`
  declarations with mandatory type and initializer. Types currently implemented
  include all specified CLR primitives and [scalar domains](scalar-types.md), and supported imported CLR
  types with nullable reference/value forms. See [CLR objects](clr-objects-and-resources.md).
- Single-line string literals with doubled apostrophes, Unicode string contents,
  decimal/hexadecimal integral literals with L/U/UL suffixes, and real F/D/M
  literals with fractional parts/exponents and digit separators.
- Reads of parameters and already initialized locals, parenthesized expressions,
  named routines, constructors, static/instance CLR calls, and fields/properties.
- Overload selection for supported signatures, preferring identity and permitting
  numeric widening, reference conversion, and boxing. Reference results retain
  their nullability metadata; oblivious results are conservatively nullable.
  Void calls may be statements; returned values must be consumed.
- Arithmetic, numeric casts, comparisons, Boolean/bitwise operators, string
  concatenation/equality, lexical blocks, assignments, if statements/expressions,
  while/repeat loops, integral/subrange counting loops, break/continue, and discard. See the
  [expression/control-flow milestone](expressions-and-control-flow.md) for rules
  and verification.
- Nullable validation/coalescing, try/catch/finally/raise and filters, and blockless
  using const with CLR IDisposable cleanup on all exits. See the CLR milestone
  for implemented behavior and remaining signature restrictions.
- Duplicate bindings/imports, unknown names, invalid type assignments, missing
  annotations, unsupported calls, and lexical/syntactic errors are diagnosed.

The complete language remains specified in the topic documents. Broader CLR
signature forms remain later steps. [Typed loops](loops.md) now include repeat
and all specified integral/subrange counters.
[Arrays, args, closed generic CLR types, indexers, and enumeration](arrays-and-collections.md)
are implemented, as are [callable values, closures, and explicit generic CLR calls](functions-and-closures.md)
and [nongeneric records, defaults, methods, equality, and with updates](records.md).
[Nongeneric unions and exhaustive case matching](unions-and-matching.md) are
also implemented, with nested/guarded patterns and CLR type tests.
[Generic records and unions](generic-types.md) add CLR generic data models and recursive payloads.
[Generic source routines](generic-routines.md) add explicit type arguments,
where constraints, generic record methods, and typed generic closures.
[Type argument inference](type-argument-inference.md) now closes routine/CLR calls
from inputs and union constructors from payloads and expected types.
[Overloads, interfaces, debugging, and async](overloads-interfaces-debugging-and-async.md)
add source overload groups, records implementing imported CLR interfaces,
portable PDBs, and async functions/lambdas/programs with awaited cleanup.
[Source documentation extraction/XML export](xml-documentation.md) is implemented
for the supported declarations.
[Primitive numbers, aliases, enums, distinct values, and subranges](scalar-types.md)
are implemented with cross-assembly CLR contracts and valid default domain values.
[Global const/var fields and CLR type initialization](global-values.md) are
implemented, including shared state across source units and CLR consumers.
The compiler currently stops at the first diagnostic.

[Imported CLR events](events.md) support subscribe/unsubscribe with typed handlers,
ordinary accessor dispatch, and receiver/handler evaluation in written order.
[Runtime type expressions](typeof.md) support typeof(T) with actual CLR type
identity, including closed generics and in-scope generic parameters.
[Source CLR attributes](attributes.md) now bind typed constructor/member constants
and emit metadata for supported type, routine, and record-property targets.

## Compiler and output

Fpas.Compiler owns scanning, parsing, binding, reference metadata, and emission.
It reads referenced assemblies through PEReader/MetadataReader without executing
them. Nullable return annotations are read from the ordinary CLR attributes.
Only supported signatures are selected; unsupported metadata is not erased into
an unchecked call.

The selected initial binary writer is .NET's built-in System.Reflection.Metadata
and ManagedPEBuilder. Typed bound nodes produce CIL locals, constants, calls,
named methods, arithmetic, branches, returns, and an executable entry point.
Assembly references/signatures come from the actual
supplied metadata. The compiler generates neither C# nor an FPAS VM image.
Identical inputs produce identical PE bytes independent of reference-list order.
Record/union/domain declarations and public aliases reference Fpas.Core for metadata attributes; other emitted
programs retain their ordinary framework/project dependencies. See [records](records.md)
for SDK distribution and direct CLI reference/deployment requirements.

The CLI writes `<name>.dll`, a portable `<name>.pdb` by default, and, for
executable output, `<name>.runtimeconfig.json`. Use --debug none to disable symbols.
Optional --generate-documentation or --documentation path.xml writes .NET XML documentation.
Library output has no entry point or runtime configuration. Switching an existing
output to a library removes its former compiler-owned runtime configuration.
Failed compilation leaves existing outputs unchanged. Publication stages outputs and restores
previous files if a replacement fails; unrecoverable rollback retains backups
and reports their location. Parallel callers must use distinct output paths.

## CLI contract

`compile --help` documents references, stdin, and output. Supply one or more source
paths; one `-` may read a source from stdin alongside files. --output names a DLL. Repeat --reference to
supply individual assemblies, or --reference-dir to supply a directory's DLLs.
References must include the net10.0 System.Runtime assembly.

`--target exe` (the default) requires exactly one program plus optional units;
`--target library` requires units with no program.
`compile --response-file path.json` reads a JSON string array of compile
arguments instead of inline arguments; nested response files are unsupported.

Add `--diagnostics json` for one stdout JSON object containing success, output,
and diagnostics. Diagnostics carry a code, message, and source path/line/column/
length. Normal mode writes diagnostics to stderr and the output path to stdout.
Exit codes are 0 for success/help, 1 for compilation/input/output failure, and 2
for invalid arguments.

| Codes | Meaning |
| --- | --- |
| FP0001 / FP0002 | Invalid CLI arguments / input-output failure |
| FP1001–FP1004 | Invalid character, string, separator, or unsupported numeric form |
| FP2001–FP2005 | Syntax/terminator error, missing type annotation, unsupported form, source/output-kind mismatch, or duplicate container |
| FP3001–FP3008 | Name, type, overload, binding, unsupported type, unused result, missing return, or unreachable statement |
| FP3009 / FP3010 | Non-writable assignment target / loop exit outside a loop |
| FP3011 / FP3012 | Private routine/member/constructor access / importing a program |
| FP3013 / FP3015–FP3017 | Nullable safety / property accessors, disposal, or exception-flow errors |
| FP3018 / FP3019 | Generic constraint failure / unsupported or unresolved callable signature |
| FP3021 / FP3022 | Invalid record declaration or visibility / invalid record construction or update |
| FP3023–FP3025 | Invalid union declaration, pattern, or coverage/unreachable case arm |
| FP3026 | Invalid/unsupported source generic declaration or explicit type-argument arity |
| FP3027 / FP3028 | Invalid contract / parameter modes, defaults, storage, or out flow |
| FP3029 / FP3030 | Invalid record interface implementation / async or await reference-lifetime error |
| FP3031 | Invalid alias, enum, distinct declaration, subrange bounds, or domain construction |
| FP3032 | Invalid global name or public global exposing private types |
| FP3033 | Nested routine environment created before captured bindings initialize |
| FP3034 | Invalid or ambiguous event selection/access, or event used as a delegate value |
| FP3035 | Invalid attribute class, target, duplicate, constant value, named member, or structural metadata |
| FP6001 | Documentation warning; severity follows the configured warning policy |
| FP4001 / FP4002 | Invalid/missing reference metadata / ambiguous reference identity |

## Verification

```sh
dotnet test Fpas.slnx --no-build --no-restore
```

xUnit checks source diagnostics and locations, emitted entry point/namespace/CLR
call signatures, reference ordering and deterministic output. Integration tests
invoke the actual CLI and dotnet host, including the checked-in Hello example,
stdin, nested calls, Unicode/quotes, Int32 output, JSON/help/exit status, invalid
UTF-8, and preservation/rollback of existing output files. Test reference paths
are obtained from the test project's resolved framework references.

The [MSBuild milestone](msbuild.md) describes the implemented Project SDK, its
bootstrap, mixed-project tests, and remaining limits separately from these
direct-compiler checks.
