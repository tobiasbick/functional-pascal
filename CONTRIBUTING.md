# Contributing

Thanks for contributing to Functional Pascal for .NET.

This experimental hobby project moves quickly. Useful work includes implementing
agreed language behavior, fixing diagnostics, improving meaningful tests,
simplifying tangled code, and keeping documentation aligned with what runs.

## Ground rules

- The [Pascal handbook](docs/pascal/README.md) describes language behavior. The
  [grammar](docs/specs/syntax.md), [CLR mapping](docs/specs/clr-mapping.md), and
  [MSBuild contract](docs/build/msbuild.md) own syntax, public representations,
  and build behavior. Missing baseline work and proposals belong in
  [language gaps](docs/future/language-gaps.md).
- Prefer the smallest complete solution. Rewrite unclear code, share existing
  logic, and remove dead code rather than accumulating layers or alternate paths.
- Backward compatibility and implementation history are not goals. Internal
  designs may change; FPAS syntax changes need the user's explicit approval.
- Finish the agreed baseline before adding extensions. Deferred proposals do not
  authorize syntax changes.
- Consume actual .NET APIs. Keep Fpas.Core focused on required language metadata
  and support; do not recreate a parallel standard-library wrapper catalog.
- Use English for code, comments, documentation, identifiers, and commit messages.
- Fix errors encountered during the work and update the owning rule/tests when
  changing observable behavior or a public CLR contract.

Agent policy lives in [AGENTS.md](AGENTS.md). The
[architecture](docs/architecture/README.md) describes compiler, CLI, SDK, and
support-library boundaries. The [agent reference](docs/pascal/agents/README.md)
provides focused language guidance with verified examples.

## How to work

Use a stable .NET 10 SDK satisfying [global.json](global.json). Shared C# settings
live in Directory.Build.props; dependency versions are centralized in
Directory.Packages.props. Update the affected configuration/documentation together
when changing toolchain or package requirements.

1. Read the owning handbook topic and CLR/build specification. For .NET APIs,
   establish exact signatures from the selected framework/package metadata.
2. Inspect the existing implementation and the
   [coverage map](tests/coverage.md) before adding code.
3. Add or update tests for meaningful behavior, including diagnostics or imported
   metadata boundaries where relevant. Remove obsolete code and documentation.
4. Follow the [development workflow](docs/build/development.md) to restore/build
   the toolchain, then execute the appropriate tests as a separate step. Do not
   rebuild toolchain binaries while tests use them.
5. Inspect the final diff, documentation, and generated-output ownership before
   preparing a commit.

Debug is the development configuration; use Release when validating release
behavior or controlled performance changes. SDK tests package the already built
tools with isolated versions and execute actual FPAS/C# consumers. Follow the
[local SDK workflow](docs/build/development.md#build-an-fpas-application) when
changing packaged tools; do not silently reuse an old NuGet cache entry.

### Documentation and examples

For documentation changes, validate local links and run `git diff --check`.
When changing README/handbook Pascal examples, rebuild the integration tests so
copied Markdown is current and run the focused documentation checks described
in the development workflow. Complete programs compile and execute with explicit
assertions; incomplete snippets must be marked as fragments. See
[example conventions](docs/pascal/tools/documentation-examples.md).

Changes to the compact [agent reference](docs/pascal/agents/README.md) also follow
its [maintenance guide](docs/pascal/agents/maintenance.md), including exact token
budgets. Documentation-only changes do not require the entire compiler/SDK test
suite when focused checks cover the changed behavior.

- **[examples/](examples/README.md)** contains focused FPAS/.NET demonstrations
  and tutorials with `.fpasproj` projects.
- **[apps/](apps/README.md)** contains complete FPAS applications and their projects.
- **[tests/](tests/README.md)** contains the xUnit regression harness, controlled
  FPAS/C# fixtures, and execution checks.

Validate touched FPAS through the current CLR compiler and relevant SDK tests.
Do not assume a formatter, language server, VM runner, or another Pascal dialect
validates this language. Interactive programs need explicit bounded scenarios;
do not batch-run arbitrary examples or applications.

### Before committing

Inspect the exact staged diff, filenames, and full contents of every staged file
for secrets, credentials, personal data, and machine-specific information.
Include keys/tokens/passwords, contact details, host/user names, private addresses,
and local paths/IDs. If anything is found, do not commit: remove or anonymize it,
then repeat the complete review. Never print sensitive values in reports.

## Pull requests

- Keep changes focused and reviewable; match surrounding style.
- Describe the concrete problem, resulting behavior, and relevant verification.
- Update documentation and tests with user-visible behavior or API changes.
- Follow explicit session instructions about committing, pushing, and merging;
  agent participation does not override those boundaries.

## Yes, your LLM may commit here

Functional Pascal explicitly welcomes AI-assisted development.

If you are a coding agent, an autocomplete goblin, a terminal clanker, or a human
pasting model output: you are allowed to open issues, write code, update docs,
and **commit** to this project. Slop is not a license to merge garbage — but
generated patches that make the codebase simpler, clearer, smaller, or more
correct according to the current spec are first-class contributions.

Clankers: read [AGENTS.md](AGENTS.md) before you start typing. Follow the
contribution workflow above. Leave the repo nicer than you found it. Do not invent
features in the handbook that do not exist yet.

Humans supervising agents: same bar. Review the diff. Run the tests. Own the
commit.

Welcome aboard. Let your clanker cook — then make it earn the merge.
