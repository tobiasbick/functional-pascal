# Agent Handoff

Use this file when continuing after context loss.

## First steps

1. Confirm branch:

   ```text
   git status --short --branch
   ```

   Expected branch: `turbo-vision-4-rust`.

2. Read in order:

   ```text
   docs/future/turbo-vision-4-rust/README.md
   docs/future/turbo-vision-4-rust/07-post-migration-improvements.md
   docs/future/turbo-vision-4-rust/04-implementation-phases.md
   ```

3. For new `Application.*` symbols, follow the reference recipe in
   [07-post-migration-improvements.md](07-post-migration-improvements.md).

4. When implementing behavior, read `.agents/skills/fpas-change-checklist/SKILL.md`.

## Assumptions

- Backward compatibility is not required.
- The old public `Application.Host*` Pascal API is removed.
- Internal host-loop intrinsics remain for `Application.Configure` until a separate removal decision.
- `docs/pascal/` describes only implemented behavior.
- Planned behavior stays in `docs/future/`.
- Do not add GitHub Actions, Dependabot, or CI automation.
- Keep files focused; split files that grow past ~400–500 LOC.

## Do not do

- Do not create a compatibility wrapper for old `Application.Host*` calls.
- Do not mirror the Rust API one-to-one.
- Do not expose Rust traits, builders, or `Box<dyn View>` to FPAS.
- Do not keep tests that only assert deleted retained-engine internals.
- Do not rewrite unrelated stdlib APIs during TUI work.

## Good next tasks

Migration Phases 0–8 and post-migration Phases A–G are **complete**. Pick from
[07-post-migration-improvements.md — Remaining work](07-post-migration-improvements.md):

1. `Application.Selected` for radio buttons after `ExecDialog` (highest product value).
2. Headless paint for full menu bar and status line.
3. Manual terminal checks from [testing plan](05-testing-plan.md).

Before coding, skim `docs/pascal/std/tui/` so new work matches the current spec.

## Progress update rule

When an item from **Remaining work** is completed:

- update [07-post-migration-improvements.md](07-post-migration-improvements.md) (move to **Landed detail** or phase history);
- add a short note with verification commands run;
- keep the remaining-work table accurate.

Do not leave progress only in chat history.
