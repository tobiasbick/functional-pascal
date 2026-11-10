# AGENTS

FPAS is a function-first Pascal language with selected Ada conventions, targeting
the CLR directly through CIL and assembly metadata.
Start with the [agent reference](docs/pascal/agents/README.md). It explains the
language with verified examples and routes to the relevant specifications,
development workflow, build/test commands, and tooling. Load the topics needed
for the task.

## Code

- Follow the repository layout and compiler boundaries described by the reference.
- Keep files focused on one concern and usually below 500 lines.
- Group related files by topic in subdirectories and matching namespaces.
- On every code change, review affected files for size, cohesion, directory
  placement, and namespaces. Split or reorganize them as needed in the same change.
- Write all code comments in English.
- In FPAS, use `//` comments and Markdown sections for API documentation.
  Do not use C#-style `///` comments or source XML documentation tags.
- Document every public declaration in source, including types, routines, and
  members. Explain its purpose and document each parameter, type parameter, and
  return value where applicable. Documentation for private declarations is
  optional.
- When writing FPAS, use `requires` and `ensures` to state caller obligations
  and guarantees on successful return. Put these contracts only on entry
  routines, such as the main routine called by a program's entry body or a
  library operation exposed to callers. Let internal helpers rely on the
  conditions checked at entry.
- Before changing public CLR names, namespaces, signatures, or representations,
  follow the owning interop and CLR mapping specifications. Update them in the
  same change when revising a mapping. Consume actual .NET APIs.

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
- Prefer one canonical spelling per language construct. Justify deviations from
  Pascal or Ada conventions, or explain the intended form with targeted
  diagnostics.
- Resolve confirmed gaps in the agreed language/build baseline before adding
  extensions. Track them separately from deferred proposals in the language-gaps
  document.
- Fix errors and code organization issues when you encounter them. Do not
  dismiss them as pre-existing.
- Keep the agent reference aligned with implemented language and toolchain behavior.
- Keep plans and gap inventories limited to open work. Remove completed entries
  and delete fully completed documents; current contracts and usage belong in
  their owning references.
- When replacing a build under the same `Fpas.NET.Sdk` version, evict that
  version from the target NuGet cache. Use isolated caches for concurrent tests.

## CUI contributions

- Support upstream CUI. When finding a defect or missing capability, establish
  whether it belongs to CUI or the FPAS integration and explain its impact.
- For confirmed upstream findings, ask the user whether to open a CUI issue and
  whether to prepare and contribute a fix. Obtain explicit approval before
  publishing an upstream issue or pull request.

## Before every commit

- Format every new or changed `.fpas` file with `fpas fmt` and verify it with
  `fpas fmt --check` before committing.
- Inspect the exact staged changes, file names and full contents of every file
  being committed for secrets, credentials, personal data and machine-specific
  information. Include API/private keys, tokens, passwords, personal contact
  details, host/user names, private addresses and machine-specific paths or IDs.
- If any such data is found, do not commit. Remove or anonymize it and repeat
  the complete review before committing. Never print sensitive values in reports
  or logs.

## Dogfooding

- Write repository maintenance helpers in FPAS when supported CLR APIs suffice.
  When a missing feature forces a fallback to host tools, record the blocker
  and verification limits under `docs/future/`.
- Debug and profile implemented programs with supported .NET tools before
  writing external samplers or benchmark variants.
