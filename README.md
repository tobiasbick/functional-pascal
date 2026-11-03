# Functional Pascal for .NET

A function-first language with Pascal-style syntax and selected Ada conventions,
compiling directly to the Common Language Runtime through CIL and assembly metadata.

Use functions, immutable records, discriminated unions, checked scalar domains,
closures, contracts, and Task-based asynchronous code with actual .NET APIs.
Projects use `.fpasproj` files and can share `.slnx` solutions with C#. The local
Fpas.NET.Sdk resolves framework/project/package references and builds managed
executables and libraries for net10.0/AnyCPU.

| Task | Start here |
| --- | --- |
| Learn the language or find a rule | [Pascal handbook](docs/pascal/README.md) |
| Write FPAS with an agent | [Compact agent reference](docs/pascal/agents/README.md) |
| Run a complete program | [Handbook examples](docs/pascal/examples/README.md) and [SDK examples](examples/README.md) |
| Build/test the toolchain or bootstrap an application | [Development workflow](docs/build/development.md) |
| Work on the compiler, closures, async, or symbols | [Architecture](docs/architecture/README.md) |
| Check grammar or emitted public contracts | [Syntax](docs/specs/syntax.md), [CLR mapping](docs/specs/clr-mapping.md), and [MSBuild](docs/build/msbuild.md) |
| Select verification | [Tests](tests/README.md) and [coverage map](tests/coverage.md) |
| Check missing baseline work or proposals | [Language gaps](docs/future/language-gaps.md) |

The compiler emits managed code directly; it does not generate C# or run an FPAS
VM. The CLR supplies memory management, exceptions, and tasks. Fpas.Core carries
language metadata rather than a second standard library.

Use a stable .NET 10 SDK satisfying global.json. Package versions are centralized
in Directory.Packages.props. Repository commands and SDK package-cache handling
belong in the development workflow. The [documentation hub](docs/README.md)
routes to detailed user and contributor references.
