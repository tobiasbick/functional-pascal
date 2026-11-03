# Documentation

Functional Pascal targets the CLR directly. Start with the handbook to write
programs, the agent reference for focused authoring guidance, or the development
workflow to work on the compiler/toolchain.

| Area | Entry point |
| --- | --- |
| Language, projects, interop, libraries, and user tools | [Functional Pascal handbook](pascal/README.md) |
| Compact language guidance and verified examples | [FPAS for agents](pascal/agents/README.md) |
| Repository build/test commands and local SDK bootstrap | [Development workflow](build/development.md) |
| Implemented features, verification, and exact limits | [Implementation reference](implementation/README.md) |
| C# projects, compiler boundaries, SDK files, tests, and examples | [Repository layout](architecture/project-layout.md) |
| Parser-facing grammar | [Syntax specification](specs/syntax.md) |
| Public CLR identities and language lowering | [CLR mapping](specs/clr-mapping.md) |
| Project, dependency, diagnostic, incremental, and output contracts | [MSBuild specification](build/msbuild.md) |
| Missing baseline work and deferred proposals | [Language gaps](future/language-gaps.md) |
| Practical limits encountered during maintenance | [Dogfooding gaps](future/dogfooding-gaps.md) |

Handbook articles describe behavior. The grammar defines surface syntax and the
CLR mapping defines emitted representations. Implementation documents identify
what is verified. Examples illustrate those contracts and do not add features.
