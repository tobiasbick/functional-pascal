# AGENTS

FPAS is a function-first Pascal language with selected Ada conventions, targeting
the CLR directly through CIL and assembly metadata.
Read the [language reference](docs/pascal/README.md) for the language rules.

## Development build

This branch contains a language design and a small executable compiler subset.
Read the [compiler subset](docs/implementation/first-program.md) and
[expressions/control flow](docs/implementation/expressions-and-control-flow.md), plus
[multiple sources/units](docs/implementation/multiple-sources.md) and
[CLR objects/resources](docs/implementation/clr-objects-and-resources.md) and
[arrays/collections](docs/implementation/arrays-and-collections.md) and
[functions/closures](docs/implementation/functions-and-closures.md) and
[records](docs/implementation/records.md) and
[unions/matching](docs/implementation/unions-and-matching.md) and
[generic routines](docs/implementation/generic-routines.md) and
[generic data types](docs/implementation/generic-types.md) and
[type argument inference](docs/implementation/type-argument-inference.md), plus
[implemented MSBuild integration](docs/implementation/msbuild.md) before claiming
feature support. Use a stable .NET 10 SDK and separate test compilation from execution:

```sh
dotnet restore Fpas.slnx
dotnet build Fpas.slnx --no-restore --configuration Debug
dotnet test Fpas.slnx --no-build --no-restore --configuration Debug
```

The tests cover the actual CLI, generated programs, compiler diagnostics/metadata,
and mixed C#/FPAS MSBuild builds using a locally packaged SDK. They do not establish
support for the complete language or [MSBuild plan](docs/build/msbuild.md).
Use Debug for development and Release for release builds and controlled
performance measurements.
Use the .NET SDK's default build parallelism unless the user requests a limit.
Run relevant available tests through the checked-in test projects and runner;
separate test compilation from execution when the runner supports it.

## Code

- Follow the [repository layout plan](docs/architecture/project-layout.md) for C#
  implementation projects, xUnit tests, SDK files, and examples/apps.
- Keep files focused on one concern and usually below 500 lines.
- Group related files in directories by topic.
- Write all code comments in English.
- Before changing public CLR names, namespaces, signatures, or representations,
  follow the [interop rules](docs/pascal/interop/README.md) and
  [CLR mapping](docs/specs/clr-mapping.md). Update their specifications in the
  same change when revising a mapping. Consume actual .NET APIs; do not recreate
  the former `Std.*` wrapper catalog.

## Approach

- Prefer clear rewrites over layers of patches.
- Less is more: keep solutions small and simple. Remove unnecessary code.
- This is a hobby project for fun. Backward compatibility and historical
  implementation notes are not required.
- Make architecture and implementation decisions autonomously throughout
  the project. Internal APIs, data structures, and existing designs may be
  rewritten, reorganized, or replaced.
- Changes to FPAS syntax require the user's explicit approval before
  implementation.
- Fix errors when you encounter them. Do not dismiss errors as pre-existing.

## Before every commit

- Inspect the exact staged changes, file names and full contents of every file
  being committed for secrets, credentials, personal data and machine-specific
  information. Include API/private keys, tokens, passwords, personal contact
  details, host/user names, private addresses and machine-specific paths or IDs.
- If any such data is found, do not commit. Remove or anonymize it and repeat
  the complete review before committing. Never print sensitive values in reports
  or logs.

## Dogfooding

- Write helper scripts in FPAS once the CLR compiler can build and execute them.
  Until then, use available host tools for repository maintenance and identify
  verification limits. The former VM/compiler cannot validate the new language.
- Debug and profile implemented programs with supported .NET tools before
  writing external samplers or benchmark variants. Do not assume the former
  `fpas debug` or VM profiling commands exist here.
- During compiler implementation, record concrete language/tooling gaps that
  force a fallback in focused documents under `docs/future/`. Keep the current
  [dogfooding gaps](docs/future/dogfooding-gaps.md) aligned with implemented features.
