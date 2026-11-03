# Documentation

Functional Pascal targets the CLR directly. Start with the handbook to write
programs, the agent reference for focused authoring guidance, or the development
workflow to work on the compiler/toolchain.

| Area | Entry point |
| --- | --- |
| Language, projects, interop, libraries, and user tools | [Pascal handbook](pascal/README.md) |
| Compact authoring guidance and verified examples | [FPAS for agents](pascal/agents/README.md) |
| Repository task skills | [Authoring](../.agents/skills/fpas-authoring/SKILL.md), [projects](../.agents/skills/fpas-projects/SKILL.md), [performance](../.agents/skills/fpas-bench/SKILL.md) |
| Repository commands and application SDK bootstrap | [Development workflow](build/development.md) |
| Project boundaries, compilation, closures, async, and symbols | [Compiler architecture](architecture/README.md) |
| Test organization and verified behavior | [Tests](../tests/README.md) and [coverage map](../tests/coverage.md) |
| Language/CLR/CLI/MSBuild requirements compared with implementation and proof | [Baseline comparison](baseline-comparison.md) |
| Parser-facing grammar | [Syntax specification](specs/syntax.md) |
| Public CLR identities and language lowering | [CLR mapping](specs/clr-mapping.md) |
| Project, dependency, diagnostic, incremental, and output contracts | [MSBuild specification](build/msbuild.md) |
| Missing baseline work and deferred proposals | [Language gaps](future/language-gaps.md) |
| Practical limits encountered during maintenance | [Dogfooding gaps](future/dogfooding-gaps.md) |
| Missing practical guides and verified walkthroughs | [Documentation gaps](future/documentation-gaps.md) |

Each rule has an owner: the handbook describes language behavior, the grammar
defines syntax, and the CLR/MSBuild specifications define emitted and build
contracts. Architecture explains implementation choices; the coverage map points
to verification. Examples illustrate those contracts and do not add features.
