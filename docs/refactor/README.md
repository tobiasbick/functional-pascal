# Refactor plans

Structured refactor backlogs for Functional Pascal. Use this directory when work spans multiple sessions or agents — each item is a self-contained checklist you can finish and mark done without losing context.

**Rules**

- One concern per file. Mark items `[x]` only when implemented **and** verified (`cargo test`, relevant `fpas test`).
- Do not describe unimplemented behavior in `docs/pascal/`. Plans live here until done; then move facts to `docs/pascal/` and archive or delete the plan item.
- Add new items when you discover more debt. Prefer documenting **what** and **why** over prescriptive code dumps.

## Ready for implementation

_No items currently blocked on planning only._

## Status overview

| ID | Topic | Status |
| --- | --- | --- |
| [tui-bridge/00-context.md](tui-bridge/00-context.md) | TUI bridge — current architecture (reference) | Reference (keep updated when bridge changes) |
| [tui-bridge/done/02-single-tv-session.md](tui-bridge/done/02-single-tv-session.md) | One `TurboVisionApplication` per FPAS session | Done |
| [tui-bridge/done/03-about-message-box.md](tui-bridge/done/03-about-message-box.md) | About / simple dialogs via upstream `message_box` | Done |
| [tui-bridge/done/04-headless-test-util.md](tui-bridge/done/04-headless-test-util.md) | Headless tests via TV `draw` + in-memory backend | Done |
| [tui-bridge/04-command-map-sync.md](tui-bridge/04-command-map-sync.md) | Keep reserved `CM_*` list aligned with upstream | Ongoing (bump checklist + `reserved_list_matches_upstream` test) |
| [tui-bridge/05-reduce-reconcile-rebuild.md](tui-bridge/05-reduce-reconcile-rebuild.md) | Incremental view updates instead of full desktop rebuild | Done (Phase B optional) |
| [tui-bridge/06-review-bridged-widgets.md](tui-bridge/06-review-bridged-widgets.md) | Re-evaluate `Bridged*` wrappers after TV 2.0 | Done (headless mouse follow-up open) |
| [tui-bridge/done/07-pascal-message-box-api.md](tui-bridge/done/07-pascal-message-box-api.md) | Public `Std.Tui` message box API | Done |
| [tui-bridge/done/01-turbo-vision-2-upgrade.md](tui-bridge/done/01-turbo-vision-2-upgrade.md) | Upgrade to turbo-vision 2.0 + Borland command ids | Done |

## Themes

| Directory | Scope |
| --- | --- |
| [tui-bridge/](tui-bridge/) | `Std.Tui` VM bridge over [turbo-vision-4-rust](https://github.com/aovestdipaperino/turbo-vision-4-rust) |

Add sibling directories here when other areas need the same treatment (compiler, project loader, IDE shell, …).

## Suggested order (TUI bridge)

_No pending bridge items. On every `turbo-vision` bump: run [04-command-map-sync](tui-bridge/04-command-map-sync.md) (`cargo test -p fpas-vm reserved_list_matches_upstream`)._

Open follow-ups: headless `TestClickMouse` via `push_event` ([06-review-bridged-widgets.md](tui-bridge/06-review-bridged-widgets.md)); optional reconcile Phase B ([05-reduce-reconcile-rebuild.md](tui-bridge/05-reduce-reconcile-rebuild.md)).

## See also

- [docs/pascal/std/tui/app/vm-bridge.md](../pascal/std/tui/app/vm-bridge.md) — implemented bridge map (contributors)
- [docs/future/](../future/) — language/stdlib product roadmap (not bridge internals)
- [.agents/skills/turbo-vision-4-rust/SKILL.md](../../.agents/skills/turbo-vision-4-rust/SKILL.md) — integration procedure for agents
