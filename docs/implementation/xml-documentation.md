# XML API documentation

The compiler implements the existing [comment and Markdown rules](../pascal/language/comments.md)
and [MSBuild documentation output](../build/msbuild.md#documentation-output)
for its supported declarations. No additional comment syntax is introduced.

## Source documentation

A contiguous standalone // block immediately before a declaration documents it.
Blank source lines detach a block; a // line within the block is a Markdown
paragraph break. Inline comments remain ordinary comments. Documentation belongs
before public/private/async/static modifiers. Comments within those modifiers
do not replace the leading documentation.

Program/unit headers, routines, records, record fields, union cases, enums,
enum fields, aliases, and scalar domains retain
their attached source comments. Private members and local bindings also retain
comments, but do not appear in the public XML. Aliases have no CLR type ID;
method IDs use their target types. Public global values export F: field IDs;
private globals retain comments without appearing in the XML. Arbitrary source
attributes remain outside the implemented language subset.

The compiler reads Parameters, Type parameters, Returns, and Remarks Markdown
sections. Names are case-sensitive source spellings, including @ escapes; XML
uses the actual unescaped metadata names. Routine type-parameter sections describe
only that routine's own generic parameters. Returns sections apply to functions,
including Task-returning functions. Missing documentation does not warn.

Invalid section placement, repeated sections, unknown/duplicate names, malformed
parameter items, and missing parameter descriptions produce FP6001 warnings
when XML generation is enabled. The first valid entries are retained; invalid
material remains readable in remarks. Paragraphs, code spans/fences, lists, and
links become ordinary .NET XML elements. Raw HTML/XML is escaped text and never
loads external content. There is no cref resolution or inherited documentation.

## Compiler and CLI

CompilationRequest.EmitDocumentation opts into generation. The compiler returns
UTF-8 XML in CompiledProgram.XmlDocumentation. DiagnosticSeverity distinguishes
warnings from errors; a successful result can include warnings. DiagnosticOptions
supports TreatWarningsAsErrors, WarningsAsErrors, WarningsNotAsErrors, and NoWarn.
Turning documentation on or off does not change the emitted PE/PDB bytes.

Use --generate-documentation to write a conventional XML file beside the DLL,
or --documentation path.xml for a custom destination. Generation defaults off.
The CLI also accepts --warn-as-error, --warnings-as-errors, --warnings-not-as-errors,
and --no-warn. Code lists use commas/semicolons and accept FP6001 or 6001.
JSON diagnostics include an explicit error/warning severity. Promoted errors
prevent publication. Output failure restores prior assembly/configuration/PDB/XML
files, including documentation in a separate directory or volume. A direct CLI
invocation without documentation enabled leaves unrelated XML files alone.

IDs use resolved CLR signatures: nested types, generic arity and parameter slots,
constructed types, nullable values, arrays, byrefs, overloads, and record properties.
Headers document their container type. Union cases document public factory methods
or fieldless-case properties; generated constructors/helpers/case classes receive
no copied comments. Entries are sorted by ID with ordinal comparison.

## Project SDK

The SDK honors GenerateDocumentationFile and DocumentationFile. Generation
defaults off; a nonempty DocumentationFile enables it unless explicitly disabled.
Requested files must have an .xml extension. Relative paths resolve against the
project directory. Compilation writes a separate intermediate XML file, then
publishes the requested path and the conventional assembly-named sidecar.

DocFileItem/FinalDocFile and ordinary .NET reference handling make the sidecar
available to referencing C# projects. Compiler fingerprints include generation
options, paths, comments, and warning policy, and verify all generated XML files.
Missing or changed documentation invalidates compilation. Cached documentation
warnings are reported on unchanged builds. Paths recorded by a successful build
are removed when replaced or disabled; no XML wildcard cleanup is used. FileWrites
also records intermediate, conventional, and custom outputs. Before CoreClean,
the SDK removes the XML paths from its compiler manifest, including custom paths
outside bin/obj which ordinary MSBuild FileWrites cleanup excludes.

## Verification

Compiler tests cover attachment with LF/CRLF/CR, Markdown sections/rendering,
escaped names, public export boundaries, warning policy, deterministic XML, and
unchanged PE output when generation is toggled. XML is parsed with a standard
parser. A mixed-project test compares FPAS documentation IDs with IDs emitted
by the C# compiler for an equivalent API.

CLI tests cover structured warnings, promoted-error output preservation, custom
paths, input protection, and rollback on XML output failure. SDK tests exercise
Debug/Release, default/custom destinations, C# sidecar copying, comment-only edits,
missing/corrupt outputs, unchanged builds, warning settings, generation toggles,
and Clean while retaining unrelated XML. The same solution is built and tested
on Windows and Linux.
