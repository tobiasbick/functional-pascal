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

Phases 0–8 are complete on branch `turbo-vision-4-rust`. Continue with deferred Phase 5 widgets in this order:

1. **TextViewer** — same checklist as other widgets after file dialog lands. Maps to `turbo_vision::views::text_viewer::TextViewer` (read-only scrolling text; not `Memo`).

Before starting, read [implementation phases](04-implementation-phases.md) and `.agents/skills/fpas-change-checklist/SKILL.md`.

## Progress Update Rule

Whenever a phase item is completed:

- update [implementation phases](04-implementation-phases.md);
- add short notes for commands run and failures found;
- keep the next unchecked item obvious.

Do not leave progress only in chat history.
