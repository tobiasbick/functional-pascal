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
- Before changing public CLR names, namespaces, signatures, or representations,
  follow the owning interop and CLR mapping specifications. Update them in the
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
- Finish the agreed language/build baseline before adding extensions. Distinguish
  missing baseline work from deferred proposals in the language-gaps document.
- Fix errors and code organization issues when you encounter them. Do not
  dismiss them as pre-existing.
- Keep the agent reference aligned with implemented language and toolchain behavior.

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
  writing external samplers or benchmark variants.
- During compiler implementation, record concrete language/tooling gaps that
  force a fallback in focused documents under `docs/future/`. Keep dogfooding
  gaps aligned with implemented features.
