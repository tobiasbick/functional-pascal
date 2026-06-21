# Types and registration

## Types and signatures

Reuse existing types from `**Std.Tui`** and `**Std.Console`** where possible: `**Application**`, `**Size**`, `**Std.Console.KeyEvent**`.

### `ViewId`

Logical name: `Std.Tui.ViewId`. Short: `ViewId` when `Std.Tui` is imported.

Opaque host-owned handle for one entry in the retained view tree. Sema registers `ViewId` as an empty record; only host routines return values. User code cannot construct literals or pass bare integers where a `ViewId` is expected.

Use `Option of ViewId` when a view may be absent. See [ViewId rules](testing.md#viewid-type-decided).

### `Rect`

Rectangle in terminal cells. `QueryViewRect` returns absolute screen coordinates. During
`OnViewPaint`, `Bounds` is local to the view (`x = 0`, `y = 0`) and Console coordinates use the
same local origin.

| Field | Type | Meaning |
| ----- | ---- | ------- |
| `x` | `integer` | Left edge in terminal cells. |
| `y` | `integer` | Top edge in terminal cells. |
| `width` | `integer` | Width in terminal cells. |
| `height` | `integer` | Height in terminal cells. |

### Scene-graph introspection types

`ViewState` reports the resolved state used by paint, focus, and hit-testing:

| Field | Type | Meaning |
| ----- | ---- | ------- |
| `visible` | `boolean` | The view and all ancestors are visible. |
| `enabled` | `boolean` | The view accepts input and may hold focus. |
| `focused` | `boolean` | This view is the focused leaf. |
| `active` | `boolean` | This view lies on the active focus path. |
| `exposed` | `boolean` | The view has at least one visible cell after clipping. |

`ViewOptions` reports retained behavior flags: `selectable`, `tabStop`, `preProcess`,
`postProcess`, and `clipChildren`.

`ResolvedView` contains `rect: Rect`, `clip: Option of Rect`, `state: ViewState`, and
`options: ViewOptions`. `rect` is absolute and unclipped; `clip` is the effective visible rectangle.

`ViewKind` identifies native content attached to a retained node: `Generic`, `SolidFill`, `MenuBar`,
`StatusBar`, `Label`, `Button`, `InputLine`, `CheckBox`, or `RadioGroup`. `Generic` means no native
widget is attached; a Pascal paint handler may still exist.

`ViewSnapshot` contains `id`, `parent`, direct `children`, `resolved`, and `kind`. Arrays returned by
`QuerySceneGraph` contain these records in back-to-front depth-first paint order.

### Control types

`RadioOption` contains `label: string`, `accelerator: Option of char`, `commandId: Option of integer`,
and `enabled: boolean`. `InputLineState` reports `text`, zero-based `cursor`, and `scrollOffset`.
`CheckBoxState` reports `checked`. `RadioGroupState` reports zero-based `selectedIndex` and
`focusedIndex`, using `-1` when no enabled option exists. See [Retained controls](controls.md).

### `ExitReason`

Enum describing why the hosted loop stopped (`**Std.Tui.ExitReason`**). **Registry:** the type and variants `**UserQuit**`, `**HostStop`**, `**HostAndUserStop**`, `**HostShutdown**` are registered in [`loaded/tui/`](../../../../../crates/fpas-sema/src/std_registry/loaded/tui/mod.rs) and known to the compiler enum tables. **VM:** [`Application.Run`](../../../../../crates/fpas-vm/src/vm/execute/io/tui_run.rs) records `**last_exit_reason**`, invokes the registered `**OnExit**`, and then performs close semantics. The current hosted loop reports `**UserQuit`** when `**Application.HostRequestQuit(App)`** ends the run, `**HostStop`** when low-level code stops the active hosted session during `**Run`**, `**HostAndUserStop`** when both stop signals are present in the same turn, and `**HostShutdown`** when VM global shutdown is requested while the hosted run is active.


| Variant    | Meaning                                                                                                                                                    |
| ---------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `UserQuit` | Normal exit requested by the application (for example Escape handled in `**OnKeyPressed**` calling a host **quit** primitive—Phase 3 names the intrinsic). |
| `HostStop` | Host ended the loop for an internal reason (documented per implementation).                                                                                |
| `HostAndUserStop` | Host stop and user quit were both requested in the same dispatch turn; host stop takes precedence but the combined reason is preserved. |
| `HostShutdown` | The VM entered global shutdown while `Application.Run` was active (for example due to a concurrent task failure). |


Future variants (signals, fatal I/O) may extend this enum; handlers must tolerate unknown variants if the language allows exhaustiveness rules.

### Handler signatures (normative)

All procedures run on the **main VM thread**. Parameters use `**App: Application`** for session context.

```pascal
// Conceptual — final Pascal declarations ship with sema registration.

function OnKeyPressed(App: Application; Key: Std.Console.KeyEvent): boolean;
// Returns true if the key was consumed (no further default processing for this event).

procedure OnResize(App: Application; NewSize: Size);

procedure OnViewPaint(App: Application; ViewId: ViewId; Bounds: Rect);

procedure OnPaint(App: Application);

procedure OnIdle(App: Application);

procedure OnExit(App: Application; Reason: ExitReason);
```

`**OnKeyPressed` return value:** `true` = **consumed**. The host does not promise a second consumer; later phases may use consumption for command routing.

`**OnResize`:** `NewSize` matches `**Application.Size(App)`** after the resize is applied.

---

## See also

- [Views and focus](views.md)
- [Modals and dialogs](modals.md)
- [Handlers](handlers.md)
- [Hosted dispatch overview](README.md)
