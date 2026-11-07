# CLR TUI library gaps

Build a CLR library with a TUI core written in FPAS and usable from FPAS, C# and
F#. Carry forward Model–Update–View and selective updates from `main`. Design
the CLR library and terminal host anew.

## Project layout and namespaces

Place the `Fpas.Tui` library project in `src/Fpas.Tui/`. Reserve `Fpas.Tui` as
the CLR namespace root and give each FPAS unit a name below it. Use
`Fpas.Tui.Application` for public host entry points such as `Run` and
`Fpas.Tui.Elements` for the public element tree and factories.

Group implementation units by topic under `Fpas.Tui.Internal.Layout.*`,
`Fpas.Tui.Internal.Rendering.*`, `Fpas.Tui.Internal.Headless.*` and
`Fpas.Tui.Internal.Terminal.*`, with matching source directories.

Define the remaining CLR signatures, package identity, XML documentation and
dependencies. Follow the [CLR mapping](../specs/clr-mapping.md) and
[interface rules](../pascal/interop/interfaces.md).

## Application interface and effects

Start the interactive host with `Run(initialModel, Update, View)`. Headless
sessions take the same three inputs. `Update` takes a model and an application
message and returns the next model and a list of commands. `View` takes the model
and deterministically builds an immutable `Element<Msg>` tree. An element
describes which typed message an interaction produces. A button's `onActivate`
message applies to both keyboard and pointer activation.

The host routes input and effect results as typed messages and runs `Update`
serially. It owns layout, input routing, terminal output and asynchronous work,
including timers and subscriptions. Define the queue capacity, command dispatch
and failure delivery. Keep I/O in effects and keep the model and render buffers
on the host's serial execution path.

Return `Exit<TMsg>()` in a transition's command list to end the session normally.
The host closes queue admission, cancels and joins owned work and runs terminal
cleanup.

When the host processes a `StartBackground` command, it starts a new run for
its ID and replaces any previous run with that ID. Replacement and
`CancelBackground` cancel the old work and invalidate its run. Discard messages
from an invalidated run when dequeuing them for `Update`, including messages
already in the queue when the run was cancelled or replaced.

## Host scheduling

Use one bounded FIFO queue per session for normalized input and independent
application messages. Finish each entry's processing and settle its frame
outcome before starting the next entry. The host runs all `Update` calls
directly caused by the entry, registers their commands and publishes at most
one frame. A background command result enters the queue as a separate message
and gets its own turn.

In manual headless mode, `step` processes exactly one queued entry. It returns
`idle` immediately if the queue is empty. A full queue makes `send` return
`QueueFull` immediately. Assign an input ID only after successful admission.
Pause terminal reads until space is available. Background producers wait for
space with cancellation outside the host loop. Admit already waiting producers
in arrival order as slots become free.

`wait` passively observes the session through the control connection outside the
work queue. Every wait has a deadline. In manual headless mode `step` advances
the host. Send an input, step the host and then wait for its input ID. If that
step starts background work, wait for the result to enter the queue and process
it in a later step in queue order.

## Interaction state and overlays

The application model owns overlay visibility and content, input values and
application selection. The host owns focus, hover, pointer capture and visual
scroll positions. It associates this state with stable control IDs.

`Screen` holds the base content and the overlay layers. Setting `modal = true`
gives an `Overlay` its own focus and input scope.

```text
Msg = OpenOptions | CloseOptions

Update(model, OpenOptions)  → (model with OptionsOpen = true,  [])
Update(model, CloseOptions) → (model with OptionsOpen = false, [])

View(model) =
  Screen(
    content = Button(id = "options", text = "Options", onActivate = OpenOptions),
    overlays = if model.OptionsOpen:
      [Overlay(
        id = "options-overlay",
        modal = true,
        initialFocus = "close",
        onEscape = CloseOptions,
        content = Button(id = "close", text = "Close", onActivate = CloseOptions)
      )]
    else []
  )
```

When the modal overlay appears, the host remembers the previously focused control.
It focuses `initialFocus` if it names a focusable control in the overlay, such as
`close` in the example. Otherwise it focuses the first focusable control in the
overlay. With no focusable control, the overlay stays the active input scope
with focus unset.

Tab and Shift+Tab cycle within the modal scope, which consumes pointer input
outside the overlay. Escape and the close button both dispatch `CloseOptions`.
`Update` changes the model, and the next `View` omits the overlay.

When the overlay disappears, the host restores the previously focused control
if it is still focusable in the active scope. If `options` had focus before the
overlay opened, it regains focus on close. If the saved control has disappeared
or is disabled, focus moves to the first focusable control in the active scope.

## Input events and shortcut routing

| Event | Meaning |
| --- | --- |
| `KeyPress(Key, Modifiers)` | A logical key such as Enter, Escape or Left Arrow, with modifiers when the terminal can distinguish them. |
| `TextInput(Text)` | Entered Unicode text. Shift+A produces the text `A`. |
| `Paste(Text)` | A paste identified by the terminal. The host delivers the entire payload as one event, including any line breaks. |

Route `TextInput` and `Paste` to the focused control in the active scope when
it accepts text. The single-line `TextInput` element replaces each pasted line
break with one space, treating CRLF as one break, and inserts the result as one
operation. Only `KeyPress` participates in shortcut matching.

Handle a key press at the first applicable step in this order.

1. Host navigation with Tab or Shift+Tab, or Escape when the active modal
   overlay has an `onEscape` binding.
2. Key handling by the focused control, such as arrow keys or Ctrl+A in a text
   field.
3. Shortcuts bound in the active input scope.
4. Global application shortcuts. A modal overlay suspends this step.

Bind any shortcut that should work inside a modal overlay in the overlay's
scope. For example, an options overlay can bind Ctrl+S to its own save message.

## Views and cached subviews

Compose `View` from ordinary view functions. A cached subview declares its typed
dependencies, a builder and an equivalence comparison. It builds on first use
or when its dependencies change. Equivalent dependencies must produce the same
appearance and behavior, including identities, actions, editing state and focus.
Use immutable projections or explicit revisions for dependencies drawn from
mutable state. Builders and comparisons are pure.

Keep cache entries per application instance with bounded retention. Define how
entries are scoped, when unused entries are released, how nested subviews behave
and how the host cleans up after failed builds and shutdown.

Use stable keys for dynamic siblings such as items in a sortable list. A reorder
preserves the identity of a surviving item and lets the host keep focus on it
where the focus rules allow. Define key scope and duplicate-key handling. Compare
successive element trees to distinguish changes to layout, painting and input
routing. An action change can require new routing even when every cell looks the
same.

## Layout, drawing and presentation

Use CSS-inspired flex layout. `Row` and `Column` select the main axis of the
same layout algorithm. Views declare gap, padding, alignment, starting size
(`basis`), minimum and maximum sizes, and grow and shrink factors as typed
layout values.

Measure and arrange in whole terminal cells. Distribute surplus or deficit
along the main axis according to grow or shrink factors and size constraints.
Assign leftover cells deterministically. If children cannot fit at their
minimum sizes, apply the container's declared overflow policy.

Define clipping, scrolling, overlay placement, resize, semantic styles and themes.
Text measurement and editing must account for grapheme boundaries, display width
and wide-glyph continuation cells.

Give view construction, layout, painting and terminal presentation independent
reuse rules. Resizing can require new layout and paint while the same subview
tree stays valid. With stable layout, repaint affected regions in stacking order.
Compare the resulting cells with the last presented frame and send changed cell
runs. Define when each stage falls back to full work. Incremental frames must
agree with full frames in both cells and input behavior. Choose the cell-buffer
representation and invalidation granularity using allocation and frame costs.

## Version 1 elements

Compose the first set of widgets with `Screen`, `Row`, `Column` and `Overlay`.

| Element | Version 1 behavior |
| --- | --- |
| `Text` | Styled display text with wrapping and truncation. |
| `Button`, `Checkbox` | Keyboard and pointer interaction with typed activation and value-change messages. |
| `TextInput` | Single-line editing with a caret, selection and paste. The application model owns the value. |
| `List` | Scrollable items with stable keys, model-controlled single selection and activation by Enter or pointer. |
| `ProgressBar` | Display a progress value, such as progress reported by a background command. |
| `Border` | Draw a frame with an optional title around any content. |
| `ScrollView` | A scrollable viewport for longer content and forms. |
| `CellCanvas` | Draw custom cells inside the element's arranged, clipped region. The element can participate in input routing through application messages. |

Place each TUI example in its own subdirectory under `examples/tui/`. The first
end-to-end example uses a `TextInput` to filter a `List`, a button to open a
modal options `Overlay` with a `Checkbox`, and a `ProgressBar` driven by
background messages. Use the example to exercise focus, paste, resize and
background updates together.

Tabs, menus, tables and a multiline editor are follow-up work.

### FPAS contracts for the example

The FPAS fragments use `App` for `Fpas.Tui.Application` and `E` for
`Fpas.Tui.Elements`. `Sys`, `Threading` and `Tasks` alias `System`,
`System.Threading` and `System.Threading.Tasks`.

Declare the transition, commands and headless start in
`Fpas.Tui.Application`.

```text
public type Transition<TModel, TMsg> = record
  public Model: TModel;
  public Commands: array of Command<TMsg>;
end record;

public function StartBackground<TMsg>(
  Id: string;
  Work: function(function(TMsg): Tasks.Task, Threading.CancellationToken): Tasks.Task;
  OnFailure: function(Sys.Exception): TMsg
): Command<TMsg>;

public function CancelBackground<TMsg>(Id: string): Command<TMsg>;

public function Exit<TMsg>(): Command<TMsg>;

public function StartHeadless<TModel, TMsg>(
  InitialModel: TModel;
  Update: function(TModel, TMsg): Transition<TModel, TMsg>;
  View: function(TModel): E.Element<TMsg>;
  Columns: int;
  Rows: int
): HeadlessSession;
```

When `Update` handles `Msg.Refresh`, it returns the refresh command in its
transition.

```text
const Refresh: App.Command<Msg> := App.StartBackground<Msg>(
  'refresh',
  RefreshItems,
  function(Error: Sys.Exception): Msg => Msg.RefreshFailed(Error.Message)
);
```

`RefreshItems` receives a `Post` function and a cancellation token. It reports
progress with `await Post(Msg.ProgressChanged(Percent))`. That await completes
on admission to the FIFO queue. A separate host turn handles the message and
updates the model value shown by `ProgressBar`.

Declare shortcut bindings and cached subviews in `Fpas.Tui.Elements`.

```text
public function Shortcut<TMsg>(
  PressedKey: Key;
  Modifiers: KeyModifiers;
  Message: TMsg
): ShortcutBinding<TMsg>;

public function CachedSubview<TDeps, TMsg>(
  Id: string;
  Dependencies: TDeps;
  Build: function(TDeps): Element<TMsg>;
  Equal: function(TDeps, TDeps): bool
): Element<TMsg>;
```

```text
const Results: E.Element<Msg> := E.CachedSubview<ListDeps, Msg>(
  'results', ProjectList(State), BuildFilteredList, SameListDeps
);

const RefreshKey: E.ShortcutBinding<Msg> := E.Shortcut<Msg>(
  E.Key.R, E.KeyModifiers.Ctrl, Msg.Refresh
);
```

`ProjectList` selects the filter, immutable item data and its revision, the
selection and the option for hidden items. `BuildFilteredList` builds the list
from those dependencies. A progress-only update can reuse `Results`.

Put `RefreshKey` in `Screen.Shortcuts` to bind Ctrl+R globally. The options
overlay binds the same keys to `Msg.ResetOptions` in `Overlay.Shortcuts`.

Start the same application headless with an explicit initial size.

```text
using const Session: App.HeadlessSession :=
  App.StartHeadless<AppModel, Msg>(InitialModel, Update, View, 80, 24);
```

`StartHeadless` publishes the initial frame and returns an `IDisposable`
session. Send text to the filter, step one entry and inspect the filtered list
in the snapshot. A later progress message gets its own step.

## Input completion, frames and failures

An input is processed once the host has routed it, run `Update` for every
message produced directly by that routing, and registered the resulting
commands. Background results arrive later as separate messages. Waiting on an
input also waits for its frame outcome. Report the new frame, the unchanged last
frame, or the stage that failed.

Build the next frame in staging state. It is complete when `View`, layout,
painting and presentation to the output target succeed. Publish its ID, size,
cells and input routing together. A routing change alone publishes a new frame
ID, with no cell writes needed if the cells are unchanged.

If `View`, layout or painting fails, discard the staged frame, report the failed
stage and end the session. The last complete frame remains available for
inspection.

A terminal output error can leave the screen partially updated. Invalidate the
baseline for cell diffs. If the error is recoverable, try one full redraw from
the current model. Run each `Update` and register each command only once across
both attempts. If the terminal is lost or the redraw fails, end the session.
When a failure ends a session, cancel and join owned work and run terminal cleanup.

## Terminal host

Choose Windows and Linux adapters for raw keyboard and pointer input, resize,
terminal capabilities, alternate screen and cursor state. Define ownership of
native resources. Attempt to restore terminal state on every exit path. Use the
existing [controlled terminal runner](../pascal/tools/terminal-testing.md) and
`Fpas.Processes.Terminal` for private PTY/ConPTY sessions.

## DAP launch in a private terminal

Extend the existing DAP launch with a private-terminal mode.

| Role | Responsibility |
| --- | --- |
| Debugger | Launch and supervise the target. Keep the existing startup gate for debugger configuration and clean up the target on exit. |
| Launch worker | Create the private terminal, bind it to the target at launch and hold the native handles on behalf of the controller. Expose terminal input, output and resize, then release the handles when the session closes. |
| Controller | Own the session. Send terminal input, read output and request resize through the worker. Close the terminal session after target cleanup. |

On Windows the worker creates a ConPTY and binds the target to it at process
creation. On Linux the worker creates a PTY pair, gives the target the slave as
its controlling terminal and connects the target's standard streams to it. The
worker uses the master for input, output and resize. On both platforms the
terminal is connected before the debugger releases application code.

Keep DAP traffic on its existing connection. Terminal bytes and resize requests
go through the launch worker. Structured input and snapshots use the local TUI
control connection.

## TUI inspection and control

Both headless sessions and interactive debugger sessions must expose the TUI
for inspection and control. The headless host draws into an in-memory cell
target. In an interactive debugger session the controller also renders a screen
view from the actual terminal output stream.

Use an explicitly enabled local control connection with machine-readable
commands and responses. Both modes expose `snapshot`, `send`, `wait` and
`resize`. Manual headless sessions also expose `step`.

| Operation | Behavior |
| --- | --- |
| `snapshot` | Read the last completed frame as text and styled cells, with its ID, size and semantic control tree. Control descriptors include stable IDs, roles, labels, bounds, focus, enabled state and exposed values. |
| `send` | Queue `KeyPress`, `TextInput`, `Paste` or pointer events in the host's normal input queue. Return an input ID on admission or `QueueFull` when the queue is full. A click by control ID resolves against the current completed frame and uses ordinary hit testing, focus and disabled-state checks. |
| `wait` | Observe input completion and its frame outcome, a condition in a completed frame, queued work or available queue capacity. For input completion, report the processed input ID, frame outcome and latest completed frame ID when available. |
| `resize` | Set columns and rows. Headless queues a resize of the virtual cell target through the normal resize path. Interactive control resizes the private terminal itself, so the app receives a native resize event. Wait for a completed frame at the requested size to inspect the new layout. |
| `step` | Advance one queued entry in a manual headless session. |

The interactive controller can also write input directly to the terminal to
exercise decoding and routing together. Define control request cancellation and
disconnect behavior. Snapshots are versioned and detached from live state. Keep
application model summaries separate from generic control descriptors and expose
bounded event history, pending work and failures.

The external controller retains the latest complete frame, including after a
session fails. At a breakpoint, `snapshot` returns that frame with its original
size and ID, even if the debugger stopped the app during the next frame. The
controller can resize the terminal while stopped. The app processes the new
size and any pending input after `Continue`. Report paused or pending work
separately from processed input. Use the source debugger for breakpoints,
variables and source stepping. [Debugger reference](../pascal/tools/debugging.md)

## Acceptance

Use the same message loop, routing, view construction, layout and painting in
headless and interactive hosts.

| Level | Required evidence |
| --- | --- |
| Model and effects | Message transitions, command results, controlled timing, failures and discarding queued messages from cancelled or replaced runs. |
| Application exit | End the session through `Exit`, cancel and join owned work, and restore terminal modes. |
| Host scheduling | Step one queued entry at a time, process background results in later turns, report `QueueFull` on rejected input and verify that a passive wait reaches its deadline without advancing the host. |
| Headless UI | Keyboard, pointer and resize routing, focus through a keyed reorder, isolated caches, and equivalent cells and routing with reuse enabled. |
| Overlay focus | Check initial focus, fallback for an invalid target and an overlay with no focusable controls. Cycle focus with Tab and Shift+Tab, contain outside clicks, close by button or Escape, and restore focus with a fallback when the previous control is unavailable. |
| Input routing | Deliver typed text and multiline paste as text payloads. Replace pasted line breaks with spaces in single-line input. Verify key handling order, overlay shortcuts and suspension of global shortcuts while the modal overlay is active. |
| Flex layout | Grow and shrink within size constraints, deterministic cell allocation and explicit overflow. Resize invalidates layout independently of subview construction. |
| Rendering | A progress update reuses the cached subview of a sibling list. Custom drawing obeys clipping and Unicode cell geometry. |
| Frame completion | Verify unchanged and routing-only frame outcomes, retained snapshots after drawing failures, and one full redraw after a recoverable partial write. Confirm that recovery processes `Update` and commands once. |
| Terminal | Changed-cell output, real interaction and terminal restoration in controlled Windows and Linux sessions. |
| Private-terminal DAP launch | On Windows and Linux, launch a small app that shows its terminal size. Stop at a breakpoint, resize the private terminal from the controller, continue and observe the new size in the terminal output. |
| TUI control | Send input, wait for its completion, resize and inspect the resulting frame in both modes. In an interactive debug session, inspect the actual terminal output, resize at a breakpoint, read the last complete frame and observe the resized frame after `Continue`. |
| CLR consumers | Independent FPAS, C# and F# programs can run a view through the packaged library. |
