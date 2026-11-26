# AGENTS

FPAS is a function-first Pascal language with selected Ada conventions, targeting
the CLR. Start with the [agent reference](docs/pascal/agents/README.md) and load
the topics needed for the task.

## Approach

- This is a hobby project. Avoid backward compatibility and omit
  implementation history.
- Prefer clear rewrites over layers of patches. Keep solutions small.
- Make architecture and implementation decisions autonomously. Fix errors you
  encounter.
- Changes to FPAS syntax require the user's explicit approval before implementation.
- Prefer one canonical spelling per construct. Close agreed baseline gaps
  before extending the language.
- Remove completed work from plans and gap lists. Preserve acceptance checkmarks
  in the CUI widget inventory.
- Keep temporary data under the git-ignored `.temp-data/` in this checkout:
  scripts, logs, patches, measurements, build trees, worktrees and clones. Do
  not use the system temp directory or a session scratchpad; MSBuild rejects
  intermediate directories below `%TEMP%`. Linux checks under WSL are the
  exception and use a clone below `~/work/`, because builds on `/mnt/` are slow.

## Code

- Keep files focused, usually below 500 lines. Group files by topic in matching
  directories and namespaces, and review those boundaries when editing.
- Write comments in English. Put a relative Markdown `Documentation` link in
  each first-party source's first-line comment and link back from the owning
  page. Keep required shebangs and XML declarations first. Leave a blank line
  after the comment in FPAS.
- Document public declarations, members, parameters, type parameters and
  results. Use `//` and Markdown for FPAS API documentation.
- Use the `fpas-docs-ste` skill when creating, editing, or reviewing Markdown
  files.
- Use `requires` and `ensures` only on FPAS entry routines. Use actual .NET APIs
  and keep interop and CLR mapping specifications current for public CLR changes.
- Use `Fpas.Ui.Mvu.Testing.EffectReplies` when UI behavior depends on reply order
  or timing. Release replies individually and verify behavior before and after
  each release.
- Keep the agent reference and CLI index current. Evict repackaged SDK versions
  from the target NuGet cache and isolate caches for concurrent tests.

## Dogfooding

- Write repository helpers in FPAS whenever the required CLR APIs are supported.
- Use `fpas debug` in batch mode to capture a breakpoint or unhandled exception
  in one JSON report.
- Use `fpas profile` or the documented .NET diagnostic tools before writing
  samplers or benchmark variants.
- Record gaps found through use under `docs/future/`, including any fallback and
  its verification limits.

## CUI contributions

- Follow the CUI workflow in the agent reference. Identify whether a defect or
  gap belongs to CUI or its FPAS integration.
- Ask whether to open an upstream issue and prepare a fix. Get explicit approval
  before publishing an issue or pull request.

## Before every commit

- Run `fpas fmt` and `fpas fmt --check` on all new or changed FPAS files in the
  commit. Repeat after further edits.
- Review exact staged changes, file names and full contents for secrets,
  credentials, personal data and machine-specific information. Remove or
  anonymize findings and repeat the review before committing. Keep sensitive
  values out of reports and logs.
