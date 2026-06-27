# TUI architecture review: path to a Turbo Vision-style toolkit

**Review date:** 2026-06-19

**Scope:** `fpas-std` TUI runtime and widgets, `fpas-vm` hosted dispatch, public `Std.Tui`
API, IDE usage, tests, and the adjacent [window/dialog plan](README.md).

**Compatibility:** not required. Prefer a coherent replacement over preserving transitional APIs.

## Executive verdict

At the 2026-06-19 review date, the TUI was a useful **host foundation**, but not yet a Turbo
Vision-style toolkit. It already provided terminal ownership, a buffered CRT screen, differential
presentation, damage tracking, a basic view tree, modal scopes, commands, menu/status widgets, and
strong headless test hooks. Those parts were retained.

The window/dialog plan helped materially with frame geometry, scrolling, transforms, paint order,
and owned modal roots. Implemented unchanged, however, it would have produced Turbo Vision-looking
frames on top of an event, focus, and painting model that could not yet support Turbo Vision-like
interaction. The implementation therefore added the foundation phase, retained scene routing, and
broader toolkit coverage tracked below.

| Goal | Current system | Window/dialog plan | Verdict |
| --- | --- | --- | --- |
| Classic visual chrome | Menu/status palette only | Strong frame presets | Achievable |
| Overlapping movable windows | Root z-order + window manager helpers | Move/resize/zoom/cascade/tile via frame roots | **Met** — painted chrome, subtree damage, occlusion FPAS tests |
| Nested clipped view groups | Parent-relative rectangles + resolved clips | Adds transforms/clips | **Mostly done** — resolved clips + subtree damage; `DamageRegion::clip_rect` replaces widget duplicates |
| Focused controls and tab order | Retained focus path + tab traversal | Assumes child-first input | **Mostly done** |
| Modal dialogs | Scope, owned root, focus restore, results | Adds atomic framed root | **Met** — `ShowFramedDialog` + IDE/example integration |
| Commands and broadcasts | Sourced `CommandEvent` + reserved frame ids | Separate frame callback | **Met** — frame chrome paint + reserved-command tests |
| Standard controls | Label/button/input/checkbox/radio/list/scroll/memo + anchor/grow layout | Controls and layout are implemented in retained views | **Met** — retained controls, memo, scroll widgets, and resize layout have FPAS/Rust coverage |
| Deterministic testing | Good headless APIs | Adds frame tests | Strong reusable base |

## Reusable foundations

1. [`TuiSession`](../../../crates/fpas-std/src/tui/session/mod.rs#L15) correctly owns raw mode,
   alternate-screen, mouse, and headless lifecycle state.
2. [`Console::render_screen`](../../../crates/fpas-std/src/console/render.rs#L33) provides a retained
   cell buffer and differential terminal presentation. This is the right rendering substrate.
3. [`DamageTracker`](../../../crates/fpas-std/src/tui/damage.rs#L22) provides correct coarse dirty
   region accumulation. It can evolve from one union rectangle to a small region set later.
4. [`ViewRegistry`](../../../crates/fpas-std/src/tui/view/mod.rs#L84) already has opaque handles,
   ownership through a tree, sibling z-order, subtree removal, and modal scoping primitives.
5. The menu bar has separated geometry, input, paint, and tests under
   [`widget/menu_bar/`](../../../crates/fpas-std/src/tui/widget/menu_bar/).
6. Native headless APIs and `tests/tui/` make screen, event, modal, and widget behavior testable
   without a real terminal. Preserve this as a first-class design constraint.

## Critical findings

### C1. Painting is not a safe retained-mode compositor

The dispatcher previously ran global `OnPaint`, then global passes for widget underlays, Pascal
view handlers, and widget overlays. Retained redraw now traverses root views depth-first, paints
native widget underlays, Pascal local handlers, children, and overlays in node order, and collects
scene overlays such as menu pull-downs above the retained scene.

Pascal paint handlers receive a rectangle but no enforced clip. Direct `Std.Console` writes can
modify any screen cell. Partial damage makes this unsafe: a global handler may clear or repaint the
whole buffer while only widgets intersecting the requested damage are repainted.

Global `OnPaint` now remains a separate global-paint path for applications without retained roots.
`Application.Run` rejects global `OnPaint` when retained roots already exist, and retained
applications paint through root-local `OnViewPaint` handlers or native host widgets. Local view
handlers and widgets clip through `begin_tui_view_paint`.

**Required change:**

- Replace global/widget/handler passes with one depth-first scene traversal.
- Give every node explicit `underlay`, local content, children, and `overlay` phases.
- Enforce origin and effective clip at the console buffer boundary for the entire callback, not
  only by passing a `Rect` value.
- Replace unrestricted global `OnPaint` in retained mode with a root view paint handler or a
  clipped `PaintContext`. Backward compatibility is not needed, so do not retain two competing
  paint models.
- Make overlays part of scene ownership. A menu popup may use a dedicated overlay layer, but it
  must still obey modal scope, z-order, clipping, and damage.

### C2. There is no general consumable event router

Key routing now runs Tab traversal, retained controls, native widget keys, scoped commands, and then
global `OnKeyPressed`. The boolean returned by `OnKeyPressed` is preserved in the process tag, but
default routing still runs before the callback. Pointer routing resolves retained targets, routes
native widget pointer/wheel behavior through the widget-dispatch layer, and focuses selectable
views.

The frame plan specifies child-first bubbling, but no reusable widget or view event protocol exists
to implement it. Adding frame-specific branches to `process.rs` would repeat the menu-bar design
problem and make every new control modify the VM dispatcher.

**Required change:**

- Introduce a typed `TuiEvent` routed to a target `ViewId` with capture/target/bubble/default phases.
- Standardize `EventOutcome` as `Ignored`, `Consumed`, `Command`, `RequestFocus`, `CapturePointer`,
  `ReleasePointer`, and damage requests.
- Route focused keys to the focused view and then ancestors. Route pointer events to the topmost
  clipped hit, then ancestors. Run application fallback only when unconsumed.
- Keep event mechanics in `fpas-std`; the VM should only invoke Pascal callbacks produced by the
  routing outcome.

### C3. The view model cannot represent control or group state

`ViewEntry` stores only id, rectangle, parent, and children
([`view/mod.rs:68-73`](../../../crates/fpas-std/src/tui/view/mod.rs#L68-L73)). Focus is a separate
global list and index ([`view/mod.rs:84-89`](../../../crates/fpas-std/src/tui/view/mod.rs#L84-L89)).
There is no visible, enabled, selectable, focused, active, modal, or exposed state. Groups do not
own a current child, and explicit `HostPushChildView` can diverge from tree order and state.

Global `OnActivate` / `OnDeactivate` callbacks do not carry the old or new `ViewId`, so controls
cannot reliably update focus visuals or state. Clicking a selectable view does not move focus.

**Required change:**

- Add `ViewState` (`Visible`, `Enabled`, `Focused`, `Active`, `Exposed`) and `ViewOptions`
  (`Selectable`, `TabStop`, `PreProcess`, `PostProcess`, layout flags).
- Derive focus traversal from tree order plus state; remove `HostPushChildView`.
- Track a current child per group and a focused leaf for the active root.
- Emit typed focus transitions containing both previous and current ids.
- Focus selectable controls on pointer-down according to widget policy.

### C4. Modal state does not preserve interaction context

Modal frames store root, ownership, extra scope, and command bindings, but not previous focus,
default/cancel actions, validation, or a result. `ShowModal` attempts to focus the first scoped
entry ([`modal.rs:11`](../../../crates/fpas-vm/src/vm/execute/io/tui/views/modal.rs#L11)); if the
scope has no focusable entry, the old outside focus remains and modal command dispatch can be
blocked. Closing simply pops and optionally unregisters the root
([`modal.rs:108`](../../../crates/fpas-vm/src/vm/execute/io/tui/views/modal.rs#L108)); it does not
restore the exact previous focus.

**Required change:**

- Store previous active root and focused view in every modal frame.
- On entry, focus an explicit initial control, then the first eligible descendant, then the modal
  root as an active non-selectable fallback. Never retain outside focus.
- On close, validate the requested result, remove owned views, then restore the saved focus if it
  still exists or choose the next eligible view.
- Model `Accept`, `Cancel`, and application-defined modal results. Enter and Escape should resolve
  default/cancel controls through the modal root, not require manual key binding in every dialog.

## High-priority findings

### H1. A minimal desktop/window manager is required

The registry can raise a view internally, but there is no public window activation or desktop
policy. There is no active-window root, click-to-front behavior, constrained move/resize, window
numbering, next-window command, inactive frame state, or pointer capture. The plan currently defers
inactive color and move/resize and lists a full desktop as out of scope
([`README.md:390-415`](README.md#L390-L415)).

Turbo Vision similarity requires a **minimal desktop manager** in the frame MVP:

- desktop work area below menu bar and above status bar;
- active window tracking and click-to-front;
- one active descendant focus path;
- bounds/minimum-size constraints;
- pointer capture for drag, resize, and scrollbar thumb operations;
- active/inactive frame palette from the first interactive window release;
- optional shadow with correct damage and hit-test behavior.

Advanced MDI, tiling, cascading, and a window list may remain later work.

### H2. Commands need source, state, and one protocol

Commands currently carry only an integer to one global callback
([`command.rs:28`](../../../crates/fpas-vm/src/vm/execute/io/tui/host/process/command.rs#L28)). The
frame plan avoids id collisions by adding `FrameAction` and a separate per-frame callback
([`README.md:257-327`](README.md#L257-L327)). That solves source identity for frames but fragments
the event model before buttons, scrollbars, and dialogs are added.

**Required change:** replace both surfaces with one sourced command event:

```text
CommandEvent {
    id: CommandId,
    source: Option<ViewId>,
    kind: Application | Close | Zoom | ZoomBack | Accept | Cancel,
}
```

The command registry must also expose enabled/disabled state so menus, buttons, and status hints
stay synchronized. Built-in frame actions should run host defaults and optionally bubble a sourced
notification. Remove `HostRegisterOnFrameAction` from the plan.

### H3. Geometry, clipping, and damage must be one registry contract

The frame plan correctly calls for composable transforms and clips. Extend that requirement to a
single resolved node record containing screen rectangle, content origin, effective clip, and
visibility. Painting, hit-testing, focus damage, modal checks, queries, and reparenting must consume
that same result.

The Phase 0 correction makes `HostSetViewRect` invalidate the resolved child screen rectangle
([`tree.rs:53`](../../../crates/fpas-vm/src/vm/execute/io/tui/views/tree.rs#L53)). Frame moves and
resizes call `request_frame_subtree_damage` so descendants extending outside the parent rectangle
are repainted.

`ViewRect` exposes `intersection`, `union`, and emptiness checks; widget paint paths
now clip through [`DamageRegion::clip_rect`](../../../crates/fpas-std/src/tui/damage.rs) instead of
private duplicates.

Progress 2026-06-27: subtree damage snapshots now consume
[`ViewRegistry::resolved_subtree`](../../../crates/fpas-std/src/tui/view/geometry.rs), so damage
uses the same resolved node data as paint, hit-testing, and queries. The snapshot intentionally
keeps hidden or fully clipped nodes with their absolute rectangles for visibility and clipping
transitions. `ResolvedView` now also carries the parent content origin used for direct children,
including scrolled frame viewports, and reparenting consumes that resolved value instead of
parallel frame-specific origin logic.

### H4. Pointer capture is a prerequisite, not interaction polish

Move, resize, scrollbar arrows, and thumb dragging need to keep receiving move/up events after the
pointer leaves the initial hit rectangle. Current routing performs a fresh hit-test for every mouse
event and stores no pressed/captured view. Implement capture, pressed-button tracking, cancel on
terminal focus loss, and release on view removal before frame Phase 2/4 input work.

Progress 2026-06-27: retained routing now has pointer capture, frame move/resize and frame
scroll-thumb drags use it, captured routes survive outside the initial hit rectangle, and
`ViewRegistry::cancel_pointer_interactions` clears capture plus host-owned frame drag state on
terminal focus loss. View removal already releases capture and frame-root interaction state.

Progress 2026-06-27: buttons now use retained press/release matching. Left-button down captures the
pressed button, left-button up releases capture, and the command fires only when the release remains
inside that same button. Releasing outside cancels activation.

### H5. Terminal cell width is incorrect for general Unicode

Implemented. The TUI now uses Unicode display width through
[`cell_width.rs`](../../../crates/fpas-std/src/text/cell_width.rs) and documents the policy in
[`cell-width.md`](../../pascal/std/tui/cell-width.md). Console painting, frame titles, menu and
status geometry, labels, buttons, list box rows, input-line cursor/scroll placement, and memo
cursor placement consume the shared helpers. Wide characters reserve a continuation cell and
combining marks do not advance layout on their own.

### H6. Standard controls and layout are part of the goal

Implemented. The retained toolkit now includes static labels with accelerators, buttons with
default styling and command IDs, input lines with cursor/paste/horizontal scroll behavior,
checkboxes, radio groups, list boxes, standalone scroll bars, scroll views, memo editing, and
anchor/grow `ViewLayout` relayout.

List boxes, standalone scroll bars, scroll views, and frame chrome share the same scroll model
policy. Public contracts live in [`controls.md`](../../pascal/std/tui/app/controls.md) and
[`views.md`](../../pascal/std/tui/app/views.md); coverage lives in `tests/tui/controls/`,
`tests/tui/scene/tui_view_layout_test.fpas`, and Rust `ViewRegistry` layout tests.

## Current correctness defects to fix first

**Implementation status (2026-06-19): complete.** The six defects below now have focused Rust
regression coverage; the broader retained-engine work remains in the later phases.

1. Widget mouse target selection now skips views without widgets instead of terminating the scan.
2. Menu keyboard routing runs before modal keyboard suppression and is selected globally
   ([`key.rs:62`](../../../crates/fpas-vm/src/vm/execute/io/tui/host/process/key.rs#L62)).
   Target selection now filters both keyboard and mouse widgets through the active modal scope.
3. Popup navigation treats every non-separator row as selectable
   ([`menu_popup.rs:28-36`](../../../crates/fpas-std/src/tui/widget/menu_popup.rs#L28-L36)), while the
   public spec says disabled rows are skipped. Disabled rows are now excluded.
4. `OnKeyPressed` consumption is now reflected by process tags `1` (`true`) and `22` (`false`)
   instead of discarding the handler result.
5. A widget and Pascal paint handler can now coexist on one view; widget bases paint first, local
   handlers second, and widget overlays last.
6. `HostSetViewRect` now invalidates the resolved screen rectangle for child views.

## Structural findings

**Implementation status (2026-06-20): complete.** The oversized bridge files are split by
ownership; the largest resulting file is `menu_bar_model/decode.rs` at 271 LOC.

| Previous file | Replacement modules |
| --- | --- |
| `fpas-std/src/tui/session.rs` | `session/{mod,lifecycle,input,redraw}.rs` |
| `fpas-vm/.../tui/views.rs` | `views/{mod,handles,tree,modal,commands,widgets}.rs` |
| `fpas-vm/.../tui/menu_bar_model.rs` | `menu_bar_model/{mod,decode,dispatch}.rs` |
| `fpas-vm/.../tui/host/process.rs` | `host/process/{mod,callbacks,key,pointer,focus,command}.rs` |

Widget input and rendering policy should move out of `fpas-vm` into `fpas-std`. The VM bridge
should decode Pascal values, store Pascal callbacks, and translate outcomes only.

**Paint progress (2026-06-26):** retained widget paint no longer clones widget snapshots.
`redraw.rs` temporarily removes a widget from the retained registry, synchronizes resolved view
state, paints by reference, and restores the widget before invoking Pascal handlers or continuing
scene traversal. `StatusBarWidget::paint` now takes `&self`, so `ViewWidget::paint` delegates
without a status-bar clone.

**Input progress (2026-06-26):** widget input routing no longer clones native widgets. Target
selection returns only `ViewId` and `ViewRect`; menu, control, and frame scroll dispatch mutate the
stored widget under the TUI lock, restore it before command callbacks, and pass only lightweight
outcomes across the Pascal boundary.

`ViewRegistry::entry` is a linear scan, while recursive paint order and rectangle resolution are
rebuilt repeatedly. This is acceptable for the current demo but scales poorly with windows and
controls.

**View lookup progress (2026-06-26):** `ViewRegistry` now keeps a `HashMap<ViewId, usize>` index
beside the registration-order `Vec<ViewEntry>`. The root and child sibling vectors still define
explicit paint and hit-test order, while `entry()` and `entry_mut()` use indexed lookup. Unregister
rebuilds the index after compacting the entry vector.

## Required changes to the window/dialog plan

### Keep

- one frame implementation with window/dialog style presets;
- fixed-point scrollbar visibility and minimum geometry;
- composable content transforms and effective clips;
- underlay/content/children/overlay traversal;
- atomic creation of an owned framed modal root;
- capability defaults enabled only with implemented behavior;
- headless geometry, routing, ownership, and screen tests.

### Change

1. Add **Phase 0: retained view engine** before frame geometry: resolved nodes, enforced paint clip,
   view state/options, focus manager, event router, pointer capture, and sourced commands.
2. Replace `FrameAction`/`HostRegisterOnFrameAction` with the unified command event.
3. Include minimal desktop activation and active/inactive frame state in the first interactive
   window phase. Only advanced MDI remains out of scope.
4. Make pointer capture a prerequisite for scrollbar and move/resize interaction.
5. Extend modal work with focus save/restore, result, default/cancel, and validation.
6. Share a `ScrollModel` and scrollbar geometry with future list/editor controls.
7. Add anchor/grow layout before examples depend on manual resize handlers.
8. Define Unicode cell-width policy before title truncation and scrollbar geometry are finalized.
9. Remove unrestricted global `OnPaint` from the retained toolkit path.
10. Keep new bridge behavior in the focused `host/process/` and `views/` ownership modules; do not
    rebuild monolithic dispatch files.

## Recommended implementation order

### Phase 0 - Correctness baseline

- [x] Fix the six current defects above and add regression tests.
- [x] Centralize rectangle operations and validate non-positive geometry.
- [x] Split oversized TUI modules without changing behavior.

### Phase 1 - Retained view engine

**Implementation status (2026-06-20): complete.** Existing Pascal `Host*` calls are adapters over
the retained Rust contracts; bridge tags are encoded only at the intrinsic boundary.

- [x] Add resolved transforms/clips and an enforced paint context.
- [x] Add view state/options, group current child, focus path, and click focus.
- [x] Add typed event routing, pointer capture, sourced commands, and command enabled state.
- [x] Replace integer `HostProcessNext` tags with an internal enum; expose only stable high-level APIs.

### Phase 2 - Desktop and frames

**Implementation status (2026-06-22): partial.** Geometry, desktop constraints, active-root
primitives, atomic frame-root creation, owned framed-dialog registration, public painted
`HostCreateFrameView`, `ShowFramedDialog`, palette presets, inner viewport clipping, and overlay
chrome are complete. Frame-integrated scroll chrome is implemented; examples remain open — see
[Remaining work](#remaining-work-2026-06-22).

- [x] Add active-root tracking and raise/activate (click-to-front) in the retained registry.
- [x] Add desktop work area, active/inactive palette state, constraints, and shadow geometry.
- [x] Add static frame geometry.
- [x] Add atomic framed-window root creation.
- [x] Add atomic owned framed-dialog modal root creation.
- [x] Verify overlap, occlusion repair, clipping, focus activation, and nested frames.

### Phase 3 - Dialog controls

**Implementation status (2026-06-21): complete** for dialog-control foundations, public FPAS
bindings, input dispatch, state queries, and example integration. The retained modal stack now carries the interaction context that closing dialogs
need: each frame stores the previously active window root
and focused leaf, default (Enter) and cancel (Escape) action commands, and a resolved
[`ModalResult`](../../../crates/fpas-std/src/tui/modal/context.rs) (`Accept`, `Cancel`, or an
application-defined `Command`). `leave_with_context` returns the full
[`ModalClose`](../../../crates/fpas-std/src/tui/modal/context.rs) record so the host can unregister
owned roots and restore the exact prior window/focus, including for nested modals. Retained
[`LabelWidget`](../../../crates/fpas-std/src/tui/widget/control/label.rs) and
[`ButtonWidget`](../../../crates/fpas-std/src/tui/widget/control/button.rs) controls now provide the
first dialog-control building blocks with focused unit coverage. A retained
[`InputLineWidget`](../../../crates/fpas-std/src/tui/widget/control/input_line.rs) now adds a
single-line text model with cursor movement, insert/paste, delete/backspace, horizontal cursor
scrolling, and focused cursor painting. Retained
[`CheckBoxWidget`](../../../crates/fpas-std/src/tui/widget/control/checkbox.rs) and
[`RadioGroupWidget`](../../../crates/fpas-std/src/tui/widget/control/radio.rs) controls now complete
the first Rust-internal dialog-control set. The VM modal bridge now stores each modal's return
context on entry and restores the saved focus/window root on close, including nested modals and
owned dialog-root removal. `Application.HostSetActiveModalResult` now validates modal result codes
through the VM bridge (`1` Accept, `2` Cancel, or application-defined command results `>= 1000`).
[`show_dialog.fpas`](../../../examples/pascal/tui/show_dialog.fpas) now demonstrates owned dialogs
with host widgets and modal results, and the IDE shell exposes a Help / About dialog through
[`Ide.Dialog`](../../../apps/ide/src/dialog.fpas).

- [x] Add labels, buttons, input line, checkbox/radio controls.
  - [x] Add retained label and button widgets.
  - [x] Add retained input line widget.
  - [x] Add retained checkbox/radio controls.
- [x] Add modal result, default/cancel actions, and saved return-focus context (retained side).
- [x] Restore saved focus/window root on close through the VM bridge.
- [x] Validate modal results through the VM bridge.
- [x] Convert the manual dialog example and add a realistic IDE dialog.

### Phase 4 - Scrolling controls

**Implementation status (2026-06-21): complete.** A shared one-dimensional `ScrollModel`,
public retained `ListBox`, standalone `ScrollBar`, and integrated `ScrollView` now provide clamped
offsets, key navigation, mouse-wheel scrolling, scroll-bar arrow/track clicks, captured thumb
dragging, and FPAS state queries.

- [x] Add shared scroll model and list box.
- [x] Add integrated and standalone scrollbars and scroll view.
- [x] Complete wheel, key, track, and captured thumb interaction.

### Phase 5 - Window interaction and editor path

**Implementation status (2026-06-21): complete** for window interaction helpers. Frame roots retain
metadata and geometry on the view registry. Next-window activation, captured title-bar move, border
resize, zoom/restore, and cascade/tile layout are exposed through `docs/pascal/std/tui/app/frames.md`
host calls and VM chrome dispatch. Memo/editor primitives remain deferred.

- [x] Store frame root metadata and refresh geometry after move or resize.
- [x] Add next-window root activation cycling.
- [x] Add captured title-bar move and border resize interaction.
- [x] Add zoom/restore state for zoomable frame roots.
- [x] Wire frame chrome dispatch through the VM bridge and public FPAS APIs.
- [x] Add optional cascade/tile layout helpers.
- Add memo/editor primitives only after cursor, selection, Unicode width, and scrolling contracts
  are stable.

## Remaining work (2026-06-22)

Phases 0–7 of the [recommended order](#recommended-implementation-order) are complete. The original
[window/dialog plan](README.md) is largely realized: frame roots expose painted chrome, inner viewport
clipping, interaction, scroll chrome, and owned framed dialogs. Current spec for implemented behavior:
[`docs/pascal/std/tui/app/frames.md`](../../pascal/std/tui/app/frames.md).

### Phase 6 — Frame rendering and chrome widget

- [x] `FrameWidget` retained widget + `ViewWidget::Frame` registration.
- [x] `chrome.rs` — double-line border, title text, and `▲` / `▼` cells from geometry slots.
- [x] `style.rs` + `kind.rs` palettes — active/inactive Window (blue) and Dialog (gray) at paint time.
- [x] `FrameWidget::paint` — client fill and chrome wired into depth-first underlay/overlay dispatch.
- [x] Clip child geometry/input to the inner frame viewport rather than only protecting chrome with
  the overlay pass.
- [x] Public `Application.HostCreateFrameView`, consolidated from `HostCreateFrameRootView`.
- [x] Public `Application.ShowFramedDialog` VM bridge over existing `register_framed_dialog_root`.
- [x] Enable `Closable` capability — title-bar close hit-test and sourced close command (`CommandId` `-4`).
- [x] Frame-integrated scroll chrome (`scroll.rs`) — offset state, `▲█▼` / `◄█►` paint, wheel and
  track/thumb input on frame borders (distinct from standalone `ScrollView` / `ScrollBar` controls).
- [x] `HostSetFrameContentSize`, `HostScrollFrame`, `HostSetFrameScrollOffset`, `QueryFrameScrollState`.
- [x] Auto `content_size` from child bounds for scroll-bar visibility.
- [x] Examples: `examples/pascal/tui/framed_window.fpas`, `framed_dialog.fpas`.
- [x] FPAS test: painted frame chrome, palettes, title truncation, overlay ordering, and view kind.
- [x] FPAS + VM tests: owned framed-dialog cleanup and atomic invalid-geometry rejection.
- [x] FPAS tests: close and zoom chrome clicks.
- [x] FPAS tests: frame scroll offset queries.

### Phase 7 — Editor, layout, and polish

- [x] Memo/text editor control (multi-line cursor, selection, paste, vertical scroll).
- [x] Unicode terminal cell-width policy ([H5](#h5-terminal-cell-width-is-incorrect-for-general-unicode))
  — display width for titles, labels, input, and editor.
- [x] Anchor/grow layout flags on views ([H6](#h6-standard-controls-and-layout-are-part-of-the-goal))
  so menu, desktop, status, and frame children survive terminal resize without manual handlers.
- [x] Parent move invalidates descendant damage outside the parent rectangle ([H3](#h3-geometry-clipping-and-damage-must-be-one-registry-contract)).
- [x] Window list / MDI conveniences (optional; cascade/tile helpers are done).

### Architecture and performance debt

These original findings are closed for the retained TUI implementation:

| Finding | Status | Remaining |
| --- | --- | --- |
| [C1](#c1-painting-is-not-a-safe-retained-mode-compositor) Paint compositor | Done | Retained `Application.Run` rejects global `OnPaint`; retained apps use root-local paint or widgets; global `OnPaint` remains only for explicit non-retained paint apps and low-level dispatch samples |
| [C2](#c2-there-is-no-general-consumable-event-router) Event router | Done | Native control, menu, and frame key/pointer outcomes are reached through retained route/widget dispatch; VM process files no longer carry frame-specific key/pointer stages |
| [C3](#c3-the-view-model-cannot-represent-control-or-group-state) View state | Done | — |
| [C4](#c4-modal-state-does-not-preserve-interaction-context) Modal context | Done | — |
| [H3](#h3-geometry-clipping-and-damage-must-be-one-registry-contract) Geometry contract | Done | Damage, child origins, and reparenting now consume resolved-node data |
| [H4](#h4-pointer-capture-is-a-prerequisite-not-interaction-polish) Pointer capture | Done | Capture, frame drag, scroll-thumb drag, focus-loss cancel, removal cleanup, and button press/release tracking are done |
| [H5](#h5-terminal-cell-width-is-incorrect-for-general-unicode) Cell width | Done | Shared Unicode display-width helpers drive console paint and TUI widget geometry/cursors |
| [H6](#h6-standard-controls-and-layout-are-part-of-the-goal) Controls and layout | Done | Retained controls, memo editing, shared scroll behavior, and anchor/grow layout are implemented and covered |
| Structural ([§](#structural-findings)) | Done | Widget paint/input no longer clone widgets; retained view entries have indexed lookup while preserving sibling order vectors |

### Acceptance criteria status

| # | Scenario | Status |
| --- | --- | --- |
| 1 | Overlapping windows: raise, move, resize, zoom, restore, **close** without stale cells | **Met** — [`tui_frame_occlusion_test.fpas`](../../../tests/tui/frames/tui_frame_occlusion_test.fpas), [`tui_frame_occlusion_move_test.fpas`](../../../tests/tui/frames/tui_frame_occlusion_move_test.fpas), [`tui_frame_occlusion_zoom_test.fpas`](../../../tests/tui/frames/tui_frame_occlusion_zoom_test.fpas), [`tui_frame_occlusion_resize_test.fpas`](../../../tests/tui/frames/tui_frame_occlusion_resize_test.fpas), [`tui_frame_occlusion_raise_test.fpas`](../../../tests/tui/frames/tui_frame_occlusion_raise_test.fpas) |
| 2 | Click and Tab focus within active group | **Met** — [`tui_controls_test.fpas`](../../../tests/tui/controls/tui_controls_test.fpas), [`tui_tab_focus_test.fpas`](../../../tests/tui/controls/tui_tab_focus_test.fpas) |
| 3 | Nested modal focus restore and results | **Met** — Phase 3 + modal VM tests |
| 4 | Sourced commands from menu, button, shortcut, frame chrome | **Met** — chrome clicks + [`tui_frame_reserved_commands_test.fpas`](../../../tests/tui/frames/tui_frame_reserved_commands_test.fpas) |
| 5 | Clipped paint/input through nested groups and scroll transforms | **Met** — [`tui_view_clip_test.fpas`](../../../tests/tui/scene/tui_view_clip_test.fpas), [`tui_frame_scroll_clip_test.fpas`](../../../tests/tui/frames/tui_frame_scroll_clip_test.fpas), [`tui_nested_frame_clip_test.fpas`](../../../tests/tui/frames/tui_nested_frame_clip_test.fpas), [`tui_frame_scroll_input_clip_test.fpas`](../../../tests/tui/frames/tui_frame_scroll_input_clip_test.fpas) |
| 6 | Pointer capture for drag, resize, scrollbar thumb | **Met** — frame move/resize + scroll thumb tests |
| 7 | Resize layout for menu, desktop, status, frames, anchored controls | **Met** — `ViewLayout` + auto relayout on resize |
| 8 | Cell-width tests (ASCII, box, wide, combining, truncation, cursor) | **Met** — Rust policy tests + [`tui_cell_width_test.fpas`](../../../tests/tui/controls/tui_cell_width_test.fpas) |
| 9 | Unit + VM + FPAS workflow coverage | **Met** — frame geometry/interaction VM tests + painted-frame FPAS workflows ([`tui_framed_dialog_controls_test.fpas`](../../../tests/tui/modals/tui_framed_dialog_controls_test.fpas), [`tui_frame_occlusion_test.fpas`](../../../tests/tui/frames/tui_frame_occlusion_test.fpas), existing chrome/scroll/window tests) |

### Integration still using pre-frame patterns

- [`README.md`](README.md) implementation-phase checklists — superseded by this file; kept as design reference only.

`show_dialog.fpas` and the IDE About dialog now use `ShowFramedDialog` with host labels and buttons. `ShowDialog` remains available for plain owned modal roots (`tests/tui/modals/tui_show_dialog_test.fpas`).

**C1 progress (2026-06-26):** [`tui_menu_overlay_frame_test.fpas`](../../../tests/tui/menu/tui_menu_overlay_frame_test.fpas)
protects the compositor step by asserting that an open menu pull-down paints above frame chrome and
that closing the pull-down repaints the obscured frame/client cells. The VM host now collects menu
pull-downs during retained subtree traversal and paints them as scene overlays, removing the old
post-pass registry scan. `ApplicationHandlers.OnPaint` is now optional, so widget-only apps can
omit the previous no-op global paint handler; explicit global-paint apps continue to use
`OnPaint := Some(OnPaint)`.

**C1 progress (2026-06-27):** retained `Application.Run` now rejects global `OnPaint` when retained
roots already exist, and `HostDispatchRedraw` runs global `OnPaint` only for non-retained
global-paint applications. Interactive retained examples now register a full-screen root view with a
local `OnViewPaint` handler or rely on host widgets, and the public `Std.Tui` docs describe global
`OnPaint` as outside the retained scene.

**C2 progress (2026-06-27):** native key routing now treats frame key scrolling as part of the
general widget-key dispatch layer instead of as a separate `host/process/key.rs` branch. Pointer
routing likewise reaches frame move, resize, scrollbar, and wheel behavior through native
widget-mouse/wheel dispatch rather than frame-specific `host/process/pointer.rs` stages. The
`tui_frame_scroll_test.fpas`, `tui_frame_occlusion_move_test.fpas`, and
`tui_frame_occlusion_resize_test.fpas` regressions cover those paths.

**Structural progress (2026-06-26):** retained widget paint now paints by reference instead of
cloning widgets into redraw snapshots. Widget input target selection now returns IDs and rectangles
only, while menu/control/frame dispatch mutate the retained widget in place. `ViewRegistry` keeps an
indexed entry lookup map while preserving the existing registration vector plus explicit root/child
sibling order vectors. This closes the structural finding from this review.


The project should not claim this goal based on frame appearance alone. A minimum credible result
must pass these headless scenarios:

1. Two overlapping windows can be clicked, raised, activated, moved, resized, zoomed, restored,
   and closed without stale cells.
2. Focus follows click and Tab/Shift+Tab within the active group; hidden and disabled controls are
   skipped.
3. Opening nested dialogs moves focus inside each dialog; Accept/Cancel returns a result and closing
   restores the exact prior focus and active window.
4. Menu, button, keyboard shortcut, and frame actions all produce sourced command events and obey
   command enabled state.
5. Child painting and input are clipped through nested groups and scrolling transforms.
6. Drag and scrollbar thumb interaction continue outside the original hit rectangle through
   pointer capture and cancel safely on focus loss or view removal.
7. Resize layout keeps menu, desktop, status, frames, and anchored controls valid at small terminal
   sizes.
8. Cell-width tests cover ASCII, box drawing, wide characters, combining input, truncation, and
   cursor placement according to the chosen policy.
9. Unit tests cover geometry and state transitions; VM tests cover callback translation and
   cleanup; FPAS tests cover complete user workflows through `OpenForTest` and `TestPump`.

## Final assessment

**Can the existing implementation reach the goal?** Yes. Phases 0–7 delivered the retained engine,
controls, scrolling, modal lifecycle, frame chrome, cell-width policy, and workflow tests on top of
the terminal buffer and damage substrate.

**Does the current window/dialog plan help?** Yes — it is now largely implemented. The prior C1
global-paint conflict and structural widget-clone debt are closed; remaining work is incremental
toolkit breadth and continued consolidation around the retained event/router contracts.
