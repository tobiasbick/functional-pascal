# AGENTS

FPAS is a functional-first Pascal language with selected Ada conventions.
Read the [agent reference](docs/pascal/agents/README.md) for the language rules.

## Code

- Keep files focused on one concern and usually below 500 lines.
- Group related files in directories by topic.
- Write all code comments in English.
- Before adding, moving, splitting or renaming a public `Std.*` unit, or naming
  one in a plan, follow the binding [namespace specification](docs/specs/std-namespaces.md).
  Update that specification in the same change when revising an allocation.

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
- Use Fat-LTO for release, benchmarks, and dist; use `--profile dev-fast` with Thin-LTO for development builds.

## Before every commit

- Inspect the exact staged changes, file names and full contents of every file
  being committed for secrets, credentials, personal data and machine-specific
  information. Include API/private keys, tokens, passwords, personal contact
  details, host/user names, private addresses and machine-specific paths or IDs.
- If any such data is found, do not commit. Remove or anonymize it and repeat
  the complete review before committing. Never print sensitive values in reports
  or logs.

## Dogfooding

- Write helper scripts in FPAS instead of Python, JavaScript or shell logic.
- Debug FPAS programs with [`fpas debug`](docs/pascal/tools/debugger-batch.md):
  one run to a breakpoint or failure, one JSON result.
- Investigate performance with [`fpas run --profile`](docs/pascal/tools/profiler.md)
  before writing external samplers or benchmark variants.
- Record gaps that force a fallback or make FPAS awkward in
  [dogfooding-gaps.md](docs/future/dogfooding-gaps.md).
