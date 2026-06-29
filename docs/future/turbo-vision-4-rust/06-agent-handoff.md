# Agent Handoff

Use this file when continuing after context loss.

## First Steps

1. Confirm branch:

   ```text
   git status --short --branch
   ```

   Expected branch: `turbo-vision-4-rust`.

2. Read these files in order:

   ```text
   docs/future/turbo-vision-4-rust/README.md
   docs/future/turbo-vision-4-rust/01-decision-record.md
   docs/future/turbo-vision-4-rust/04-implementation-phases.md
   ```

3. Check the current phase checklist before editing.

4. If implementing behavior, read `.agents/skills/fpas-change-checklist/SKILL.md`.

## Assumptions

- Backward compatibility is not required.
- The old `Std.Tui` public API can be deleted.
- `docs/pascal/` must describe only implemented behavior.
- Planned behavior stays in `docs/future/`.
- Do not add GitHub Actions, Dependabot, or CI automation.
- Keep files focused; split files that grow large.

## Do Not Do

- Do not create a compatibility wrapper for all old `Application.Host*` calls.
- Do not mirror the Rust API one-to-one.
- Do not expose Rust traits, builders, or `Box<dyn View>` concepts to FPAS.
- Do not keep old retained-view tests that assert deleted internals.
- Do not rewrite unrelated standard library APIs during this work.

## Good Next Task

If no implementation has started, do Phase 1 from [implementation phases](04-implementation-phases.md):

- add the dependency in the narrowest crate that needs it;
- build;
- inspect `cargo tree -i crossterm`;
- update the checklist with results.

If Phase 1 is done, do Phase 2:

- implement only the minimal FPAS callback spike;
- avoid deleting old code until the command callback and headless test path are proven.

## Progress Update Rule

Whenever a phase item is completed:

- update [implementation phases](04-implementation-phases.md);
- add short notes for commands run and failures found;
- keep the next unchecked item obvious.

Do not leave progress only in chat history.
