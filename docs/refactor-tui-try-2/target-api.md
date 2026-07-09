# Target API (public Pascal surface)

Planned `Std.Tui` API after try-2. This becomes the spec under `docs/pascal/std/tui/` when implemented.

## Unit structure

```pascal
uses Std.Tui;
```

Single unit exporting:

- Core: `Application`, geometry types, `CommandId`, `CM_*` constants
- Views: `Dialog`, `Window`, `Button`, … each with `New` and mutation methods
- Modals: `Application.ExecView`, `MessageBox`, `FileDialog`
- Optional: `ApplicationHandlers` (later; current branch uses explicit callback registration / callback parameters)

## Core types

### `Application`

```pascal
type Application = record
  id: integer;
end;
```

| Symbol | Signature | Notes |
| --- | --- | --- |
| `Application.New` | `(): Application` | Logical session; no terminal yet |
| `Application.Open` | `(): Application` | Alias for `New` (pick one; avoid both long-term) |
| `Application.OpenForTest` | `(Width, Height: integer): Application` | Headless session |
| `Application.Close` | `(App: Application)` | Tear down session |
| `Application.Size` | `(App: Application): Size` | Terminal size |
| `Application.Run` | `(App: Application; OnCommand: procedure (Application, CommandId))` | Blocking event loop |
| `Application.Quit` | `(App: Application)` | Request loop exit |
| `Application.ExecView` | `(App: Application; View: Dialog): CommandId` | Modal; also accepts `Window` if modeless exec added later |
| `Application.MessageBox` | `(App: Application; Message: string; Options: integer): CommandId` | Upstream msgbox |
| `Application.RunFileDialog` | `(App: Application; Bounds: Rect; Title, Wildcard: string; StartPath: option of string): option of string` | Modal file picker |
| `Desktop.Add` | `(App: Application; Win: Window)` | Add modeless window |
| `Application.SetMenuBar` | `(App: Application; Bar: MenuBar)` | |
| `Application.SetStatusLine` | `(App: Application; Line: StatusLine)` | |

### Geometry

Unchanged from try-1:

```pascal
type
  Rect = record x, y, width, height: integer; end;
  Point = record x, y: integer; end;
  Size = record width, height: integer; end;
```

Helper `Bounds(X, Y, Width, Height): Rect` can live in examples or a small `Std.Tui.Geometry` internal helper — not required in std if users inline `record`.

### `CommandId` and `CM_*`

```pascal
type CommandId = integer;
```

Export upstream constants with Borland names (from `turbo_vision::core::command`):

```pascal
const
  CM_QUIT = 1;
  CM_CLOSE = 4;
  CM_OK = 10;
  CM_CANCEL = 11;
  CM_ABOUT = 100;
  { … full set from upstream prelude … }
  CM_USER = 4096;  { suggested base for app-private commands }
```

**Remove:** `Command.Quit`, `Command.Accept`, `Command.Cancel`, `Command.Close` aliases — use `CM_*` directly. Document that `CM_OK` is the OK button id (FPAS keyword `Ok` blocks a `Command.Ok` name).

## View records and methods

Each view is a record with an opaque `id` field (not visible to users if we use empty records + sema opaque types — prefer **opaque types** like try-1 where the record fields are internal).

### `Dialog`

Current branch status: `Dialog.NewModal` and `Dialog.Add` are landed. Plain modeless `Dialog.New` is target API only until a caller needs it.

```pascal
type Dialog = … opaque …

function Dialog.New(Bounds: Rect; Title: string): Dialog;  { target, not landed }
procedure Dialog.Add(Dlg: Dialog; Child: Button);
procedure Dialog.Add(Dlg: Dialog; Child: StaticText);
procedure Dialog.Add(Dlg: Dialog; Child: InputLine);
{ … overloads per child type in sema … }
procedure Dialog.SetTitle(Dlg: Dialog; Title: string);
```

Sema implements `Add` as overloaded procedures by child type (FPAS has no inheritance).

Modal factory:

```pascal
function Dialog.NewModal(Bounds: Rect; Title: string): Dialog;
```

Maps to `Dialog::new_modal` when SF_MODAL is required before `ExecView`.

### `Window`

```pascal
function Window.New(Bounds: Rect; Title: string): Window;
procedure Window.Add(Win: Window; Child: …);
procedure Window.SetTitle(Win: Window; Title: string);
```

### `Button`

```pascal
function Button.New(Bounds: Rect; Text: string; Command: CommandId; IsDefault: boolean): Button;
procedure Button.SetText(Btn: Button; Text: string);
```

### `StaticText`

```pascal
function StaticText.New(Bounds: Rect; Text: string): StaticText;
procedure StaticText.SetText(Txt: StaticText; Text: string);
```

### `InputLine`

```pascal
function InputLine.New(Bounds: Rect; Text: string; MaxLength: integer): InputLine;
function InputLine.Text(Line: InputLine): string;
procedure InputLine.SetText(Line: InputLine; Text: string);
```

### `Memo` / `TextViewer`

```pascal
function Memo.New(Bounds: Rect; Text: string): Memo;
procedure Memo.SetText(M: Memo; Text: string);

function TextViewer.New(Bounds: Rect; Text: string): TextViewer;
procedure TextViewer.SetText(V: TextViewer; Text: string);
```

### `ListBox`

```pascal
function ListBox.New(Bounds: Rect; Items: array of string; Command: CommandId): ListBox;
function ListBox.Selection(Lb: ListBox): integer;
procedure ListBox.SetItems(Lb: ListBox; Items: array of string);
```

### `Outline`

Current branch status: outline remains on the try-1 `Application.CreateOutline` API until Phase 7 or a dedicated try-2 outline pass.

```pascal
type OutlineNode = record text: string; children: array of OutlineNode; end;

function Outline.New(Bounds: Rect; Roots: array of OutlineNode): Outline;
function Outline.Selection(O: Outline): integer;
function Outline.SelectedText(O: Outline): string;
procedure Outline.SetNodes(O: Outline; Roots: array of OutlineNode);
```

### `CheckBox` / `RadioButton`

```pascal
function CheckBox.New(Bounds: Rect; Text: string; Checked: boolean): CheckBox;
function CheckBox.Checked(Cb: CheckBox): boolean;
procedure CheckBox.SetChecked(Cb: CheckBox; Checked: boolean);

function RadioButton.New(Bounds: Rect; Text: string; GroupId: integer; Selected: boolean): RadioButton;
function RadioButton.Selected(Rb: RadioButton): boolean;
procedure RadioButton.SetSelected(Rb: RadioButton; Selected: boolean);
```

`GroupId` remains FPAS-side for radio mutual exclusion unless upstream cluster API is bound instead (evaluate during implementation).

### `MenuBar` / `StatusLine`

Current branch status: `MenuBar.New`, `StatusLine.New`, `Application.SetMenuBar`, and `Application.SetStatusLine` are landed. `MenuBar.SetMenus` and `StatusLine.SetItems` are target API only.

Data records unchanged from try-1:

```pascal
type
  MenuItem = record text: string; commandId: CommandId; end;
  Menu = record title: string; items: array of MenuItem; end;
  StatusItem = record text: string; keyCode: integer; commandId: CommandId; end;

function MenuBar.New(Bounds: Rect; Menus: array of Menu): MenuBar;
procedure MenuBar.SetMenus(Bar: MenuBar; Menus: array of Menu);  { target, not landed }

function StatusLine.New(Bounds: Rect; Items: array of StatusItem): StatusLine;
procedure StatusLine.SetItems(Line: StatusLine; Items: array of StatusItem);  { target, not landed }
```

## Event handlers

### Primary: callback parameter to `Run`

```pascal
procedure OnCommand(App: Application; Cmd: CommandId);
begin
  if Cmd = CM_QUIT then Application.Quit(App)
end;

Application.Run(App, OnCommand);
```

### Optional: `OnKey` / `OnMouse`

```pascal
Application.OnKey(App, OnKey);
Application.OnMouse(App, OnMouse);
Application.Run(App, OnCommand);
```

The branch has `Application.OnKey` and `Application.OnMouse` registration. `Application.Run(App, OnCommand)` is the landed callback-parameter form for commands.

### Optional: hosted dispatch

```pascal
type ApplicationHandlers = record
  onCommand: procedure (Application, CommandId);
  onKey: option of function (Application, Std.Console.KeyEvent): boolean;
  onMouse: option of procedure (Application, Std.Console.Event);
end;

Application.Configure(App, Handlers);
Application.Run(App);
```

Aligns with `Std.Graph`. Defer until core `Run` works.

## Message box options

Keep `MessageBoxOption.*` flags compatible with upstream `msgbox` (same integer values as try-1).

## Full program examples

### Minimal modal

```pascal
program ModalDemo;

uses Std.Tui;

begin
  var App := Application.New();
  var Dlg := Dialog.NewModal(Bounds(10, 5, 40, 12), 'Quit?');
  Dlg.Add(StaticText.New(Bounds(2, 2, 30, 1), 'Exit now?'));
  Dlg.Add(Button.New(Bounds(8, 5, 16, 2), 'Yes', CM_OK, true));
  Dlg.Add(Button.New(Bounds(20, 5, 30, 2), 'No', CM_CANCEL, false));
  var Cmd := Application.ExecView(App, Dlg);
  Application.Close(App)
end.
```

### Desktop + menu (IDE-style)

```pascal
procedure OnCommand(App: Application; Cmd: CommandId);
begin
  if Cmd = CM_QUIT then Application.Quit(App)
end;

begin
  var App := Application.New();
  Application.SetMenuBar(App, MenuBar.New(Bounds(0, 0, 80, 1), [FileMenu()]));
  Application.SetStatusLine(App, StatusLine.New(Bounds(0, 24, 80, 1), [QuitItem()]));
  var Win := Window.New(Bounds(5, 2, 70, 20), 'FPAS IDE');
  Desktop.Add(App, Win);
  Application.Run(App, OnCommand);
  Application.Close(App)
end.
```

## API naming decisions (resolved on branch)

| Topic | Decision |
| --- | --- |
| `New` vs `Create` | **`New`** — `Dialog.NewModal`, `Button.New` landed |
| `Application.Open` vs `New` | **Both** — `Application.New` aliases `ApplicationOpen` |
| `Desktop.Add` vs `Application.AddWindow` | **`Desktop.Add`** — landed in phase 3 |
| `ExecView` vs `ExecDialog` | **`ExecView`** — intrinsic 475 landed |

## Sema and compiler impact

- Register view types as opaque records in `fpas-sema/.../tui/`.
- Split `application_api.rs` into `views/dialog.rs`, `views/button.rs`, etc.
- Each `Add` overload → distinct intrinsic or shared intrinsic with type tag.
- Compiler: `std_calls/tui/views/*.rs` mirroring sema split.
- Bytecode: new intrinsics replace `TuiCreate*` / `TuiAddChild` family (breaking opcode change acceptable).

## Symbols removed (breaking)

See [deletion-checklist.md](deletion-checklist.md) for the full list. Notable user-facing removals:

- `Application.CreateDialog`, `CreateButton`, … entire `Create*` family
- `Application.AddChild`
- `Application.AddWindow` → `Desktop.Add`
- `Application.ExecDialog` → `Application.ExecView`
- `Application.InputText` → `InputLine.Text`
- `Application.Checked` → `CheckBox.Checked`
- `Application.Selected` → `RadioButton.Selected`
- `Application.ListSelection` → `ListBox.Selection`
- `Command.*` → `CM_*`
- `Application.Pump`, `TestSetDialogResult`, … replaced by event injection API
