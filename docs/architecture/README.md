# Compiler and toolchain architecture

FPAS compiles directly from typed source to CIL and CLR metadata. The compiler,
CLI, MSBuild adapter, and small metadata support library have distinct owners;
the CLR provides execution, garbage collection, exceptions, and tasks.

| Topic | Responsibility |
| --- | --- |
| [Repository layout](project-layout.md) | Projects, dependencies, SDK packaging, fixtures, and applications |
| [Compiler pipeline](compiler.md) | Source/reference inputs, binding, emission, compiler API, and output publication |
| [Closures](closures.md) | Captured values/cells, lexical environments, and generic parameter translation |
| [Async and cleanup](async-and-cleanup.md) | Resumable control flow, reference lifetimes, contracts, and disposal |
| [Debug information](debug-information.md) | Portable symbols, source scopes, generated methods, and async stepping |

Language behavior belongs in the [handbook](../pascal/README.md) and
[grammar](../specs/syntax.md). The [CLR mapping](../specs/clr-mapping.md) owns public
representations and versioned metadata contracts; the
[MSBuild specification](../build/msbuild.md) owns the project/build contract.
These architecture articles explain how the implementation meets those contracts.

## See also

- [Development workflow](../build/development.md)
- [Tests and coverage](../../tests/README.md)
- [Open language requirements](../future/language-gaps.md)
