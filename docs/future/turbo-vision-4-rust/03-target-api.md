# Target API

This is a planning sketch, not current specification. Move pieces into `docs/pascal/std/tui/` only after implementation.

## Shape

Use host-owned handles for live UI objects:

- `Application`
- `View`
- `Window`
- `Dialog`
- `Button`
- `InputLine`
- `MenuBar`
- `StatusLine`

Use records for value types:

- `Point`
- `Size`
- `Rect`
- `Event`
- `KeyEvent`
- `MouseEvent`

Use command constants or a simple command type:

- `Command.Ok`
- `Command.Cancel`
- `Command.Close`
- `Command.Quit`
- application-defined command IDs

## Naming Rules

- Do not use `Host*` for public names.
- Do not expose Rust type names when a Pascal name is clearer.
- Do not expose Rust traits, boxes, builders, or ownership concepts.
- Prefer `Application.CreateX` and `Application.AddChild` for handle-based construction.
- Prefer `Window.*` and `Dialog.*` only if method syntax is already well supported and reads better.

## Minimal Spike API

The first implementation should be this small:

```pascal
uses Std.Tui;

procedure OnCommand(App: Application; CommandId: integer);
begin
  if CommandId = Command.Quit then
    Application.Quit(App)
end;

var App: Application := Application.Open();
var Win: Window := Application.CreateWindow(App, Rect.Create(5, 3, 50, 15), 'Demo');
var Btn: Button := Application.CreateButton(App, Rect.Create(18, 8, 30, 10), 'Quit', Command.Quit);

Application.AddChild(App, Win, Btn);
Application.AddWindow(App, Win);
Application.OnCommand(App, OnCommand);
Application.Run(App);
```

Accept alternate exact spelling during implementation if it better fits existing FPAS type and method rules.

## Core Application API

Planned surface:

- `Application.Open(): Application`
- `Application.Close(App: Application)`
- `Application.Run(App: Application)`
- `Application.Quit(App: Application)`
- `Application.Size(App: Application): Size`
- `Application.OnCommand(App: Application; Handler: procedure(App: Application; CommandId: integer))`
- `Application.OnEvent(App: Application; Handler: function(App: Application; Event: Event): boolean)` if needed after command routing works

## View Construction API

Planned surface:

- `Application.CreateWindow(App, Bounds, Title): Window`
- `Application.CreateDialog(App, Bounds, Title): Dialog`
- `Application.CreateButton(App, Bounds, Text, CommandId): Button`
- `Application.CreateInputLine(App, Bounds, Text): InputLine`
- `Application.CreateStaticText(App, Bounds, Text): View`
- `Application.CreateMenuBar(App, Bounds, Items): MenuBar`
- `Application.CreateStatusLine(App, Bounds, Items): StatusLine`
- `Application.AddWindow(App, Window)`
- `Application.AddChild(App, Parent, Child)`
- `Application.Remove(App, View)`

## Dialog API

Planned surface:

- `Application.RunDialog(App, Dialog): integer`
- `Application.EndDialog(App, Dialog, CommandId)`
- `Command.Ok`
- `Command.Cancel`

Do not keep `ShowFramedDialog` as the main API. Turbo Vision dialogs are already framed.

## Testing API

Planned surface depends on the spike:

- `Application.OpenForTest(Width, Height): Application`
- `Application.InjectKey(App, Key)`
- `Application.InjectMouse(App, Event)`
- `Application.Pump(App)`
- `Application.QueryScreenLine(App, Y): string`

Keep test APIs visibly test-oriented. Do not force production authors to understand the event pump.
