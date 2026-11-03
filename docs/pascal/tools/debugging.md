# Debugging CLR programs

Use a .NET debugger supported by the current environment. FPAS emits matching
portable PDBs with source documents, statement sequence points, lexical locals,
closure locations, and async yield/resume metadata.

## Prepare an executable

Build through the [project SDK](../program-structure/projects.md) or
[compiler CLI](../program-structure/cli.md), retaining portable symbols. Keep
the generated assembly/PDB pair together and make its source files available.
Use a bounded, reproducible input that reaches the breakpoint or failure.

The generated program has a normal CLR entry point; an async program has a bridge
to its task-returning body. Free routines live on their program/unit container.
Captured variables and async locals use generated CLR storage with debug metadata.
Hidden compiler plumbing should not appear as ordinary source statements.

Inspect the actual exception and source stack before changing code. Distinguish
a contract violation, a nullable/array failure, a library exception, and a failed
cleanup operation. For rethrow, use raise; inside its catch to preserve the stack.

## Scope

Portable symbol emission is implemented. Automatic FPAS IDE project loading,
language-service integration, and debugger front ends have separate scope.
Select an installed compatible .NET debugger rather than assuming a language-
specific debug command exists.

## See also

- [Compiled programs](../program-structure/compiled-programs.md)
- [Exceptions](../language/error-handling/exceptions.md)
- [Portable symbol implementation](../../architecture/debug-information.md)
- [.NET debugging tools](https://learn.microsoft.com/en-us/dotnet/core/diagnostics/)
