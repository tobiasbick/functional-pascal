# Source comments and API documentation

Use line comments for explanations and attached Markdown for declaration
documentation. The compiler can export that documentation as a standard .NET XML
sidecar. See the [compiler CLI](../../program-structure/cli.md) and
[MSBuild contract](../../../build/msbuild.md#documentation-output) for export
options. Enums, globals,
and [source attributes](../../interop/attributes.md) preserve these rules.

## Comment syntax

A comment starts with `//` and ends at LF, CRLF, bare CR, or end of input. It may
occupy a whole line or follow code. Prefix each line of a multiline comment with
`//`. Comment markers inside string literals are ordinary string contents.

```pascal fragment context=declarations
// Time remaining in the current turn.
// Measured in seconds.
const Remaining: int := 30; // Updated by the sampling loop.
const Address: string := 'https://example.com/';
```

`{ ... }`, `(* ... *)`, and `/* ... */` are not comment syntax. Compiler directives
such as `{$...}` are not supported. `///` has no special meaning: it starts an
ordinary `//` comment whose text begins with another slash.

Comments have no effect on execution, types, overload resolution, or contracts.

## Attachment to declarations

A contiguous block of standalone `//` lines immediately before a declaration is
that declaration's Markdown documentation. The rule includes program/unit
headers, types, routines, record members, union cases, enum cases, and value
bindings. A declaration's leading attributes and visibility/async/static modifiers
belong to that declaration: place its documentation before the first of them.
Comments within the attribute/modifier sequence are ordinary comments.

There must be no blank source line or other intervening construct between the
comment block and the declaration. End-of-line comments never attach. A blank
source line deliberately detaches a preceding comment:

```pascal fragment context=declarations
// Explains the next operation, without documenting its binding.

const Count: int := 1;
```

Remove indentation before `//`, the first `//`, and at most one following ASCII
space from each attached line. Join the remaining text with LF. A line containing
only `//` becomes a blank Markdown line without detaching the block. Preserve
Markdown indentation after the removed prefix, including lists and fenced code.

Comments before statements that do not declare a symbol remain ordinary comments.
Attachment is a source rule; it does not depend on an LSP, formatter, or IDE.

## Structured Markdown

Ordinary Markdown before any structured section becomes the API summary. Plain
prose and lists remain valid without special sections. The following level-two
headings have reserved, case-sensitive meanings outside fenced code blocks:

| Heading | Body | XML output |
| --- | --- | --- |
| `## Parameters` | Items of the form ``- `Name`: Description`` | One `param` element per item |
| `## Type parameters` | Items of the same form for declared generic parameters | One `typeparam` element per item |
| `## Returns` | Markdown describing the function's returned value | `returns` |
| `## Remarks` | Additional explanatory Markdown | `remarks` |

Headings begin at the start of the extracted line. Each may occur at most once;
sections may appear in any order. Other headings remain ordinary Markdown within
the current section. Indented continuation lines belong to the preceding
parameter item; descriptions may contain multiple paragraphs. Parameter and type
parameter names match the documented declaration exactly, including case. Use
the source spelling, including `@` for an escaped name; export removes that escape.
Only generic parameters introduced by that declaration belong in its type-parameter
section. Each entry needs a nonempty description. Missing entries are permitted.

Returns documentation is valid for functions, including Task-returning functions,
but not procedures or other declarations. For an async function it describes the
actual returned Task and, where applicable, its result. It does not change the
function's signature or bind a contract result. Ref/out parameters are documented
under Parameters using their declared names.

Unknown names, duplicate entries/headings, malformed parameter items, and
inapplicable sections produce documentation warnings when XML export is enabled.
Keep the first valid entry/section; preserve invalid or duplicate material as
remarks instead of emitting invalid structural elements. Warnings identify the
source location and follow the normal build warning policy. Undocumented symbols
or omitted parameter sections do not warn by default.

## Complete source example

This is one library source, independent of the other examples. Assume the output
assembly is named Numerics:

```pascal
unit Numerics.Numbers;

// Returns the larger value.
//
// ## Parameters
// - `Left`: First candidate.
// - `Right`: Second candidate.
//
// ## Returns
// The larger candidate, or their common value when equal.
public function Max(Left: int; Right: int): int;
begin
  if Left >= Right then
    return Left;
  else
    return Right;
  end if;
end function;

end unit;
```

With XML documentation enabled, the relevant Numerics.xml output is:

```xml
<?xml version="1.0" encoding="utf-8"?>
<doc>
  <assembly>
    <name>Numerics</name>
  </assembly>
  <members>
    <member name="M:Numerics.Numbers.Max(System.Int32,System.Int32)">
      <summary>Returns the larger value.</summary>
      <param name="Left">First candidate.</param>
      <param name="Right">Second candidate.</param>
      <returns>The larger candidate, or their common value when equal.</returns>
    </member>
  </members>
</doc>
```

## XML export contract

Export a standard UTF-8 .NET XML documentation sidecar, normally named after the
assembly and placed beside its DLL. See the [MSBuild contract](../../../build/msbuild.md)
for GenerateDocumentationFile, DocumentationFile, output tracking, and reference
propagation. Export does not embed documentation into CIL or require a runtime
helper. Comments in source remain Markdown.

Emit entries for documented, externally accessible symbols with actual emitted
CLR identities. Private/local declarations and transparent aliases retain their
source documentation but have no entries in the public sidecar. A documented unit
or program header maps to its container type. Public types use `T:`, routines
`M:`, record fields represented as properties `P:`, and global/CLR enum fields
`F:`. A union case documents its public factory method or fieldless-case property;
its payload labels serve as factory parameters. Compiler-generated helpers,
constructors, and case classes receive no invented copies of source comments.

Use standard .NET documentation IDs from resolved emitted signatures. Account for
overloads, primitive CLR names, nested types, generic arity/parameters, arrays,
and managed byrefs; source display names or reflection FullName strings are not
valid substitutes. Emit parameter names exactly as in metadata. Sort entries by
documentation ID using ordinal ordering for deterministic output.

Convert paragraphs to `para`, inline code to `c`, fenced code to `code`, lists to
standard `list`/`item`/`description` elements, and Markdown links to `see href`.
Unsupported presentation formatting falls back to readable text. Escape XML text
and attribute values correctly, including ampersands and angle brackets. Raw
HTML/XML in Markdown is literal text, never an instruction to inject XML or load
external content. This baseline does not add source XML tags, symbol-link/cref
resolution, include directives, or automatic inherited documentation.

The sidecar enables standard .NET documentation consumers, including C# and VB
projects referencing the assembly. Implementing FPAS hover, completion, generated
documentation sites, or a Visual Studio project system remains separate work.

## Verification

Tests cover comment termination at supported newlines and EOF, string contents,
inline comments, blank-line detachment, attribute attachment, Markdown extraction,
structured sections, escaped/case-sensitive names, generic parameters, and warnings.
Exported XML is parsed and matched to emitted metadata, including overloads,
generic types, properties, unions, arrays, and byrefs. SDK tests exercise XML
escaping, deterministic output, generation on/off, incremental changes, Clean,
and propagation to a C# reference through MSBuild.
The [coverage map](../../../../tests/coverage.md#language-and-metadata) identifies
the compiler, CLI, and SDK documentation tests.

## References

- [.NET XML documentation structure and IDs](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/documentation-comments)
- [XML documentation elements](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/xmldoc/recommended-tags)

## See also

- [Topic overview](README.md)
- [Handbook](../../README.md)
- [Syntax grammar](../../../specs/syntax.md)
