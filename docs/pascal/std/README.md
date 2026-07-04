# Standard library reference (`Std.*`)

Built-in units under the reserved `Std` namespace. Import via `uses` and refer to symbols by short or fully qualified names:

```pascal
program Hello;

uses
  Std.Console,
  Std.Math;

begin
  WriteLn(Sqrt(16.0));
  Std.Console.WriteLn(Std.Math.Sqrt(16.0));
end.
```

See [Units](../program-structure/units.md) for `uses` rules and the reserved `Std` namespace.

Each unit page is a **self-contained handbook**: importing and short vs qualified names, a **quick reference** table, then routines and types with parameters, behavior, edge cases, and examples.

## Areas

| Area | Hub | Units / topics |
|------|-----|----------------|
| Console | [console/](console/README.md) | Text I/O, CRT screen, keyboard, events |
| Host I/O | [host/](host/README.md) | Args, Env, Fs, Path, Proc, Time |
| Text | [text/](text/README.md) | Str, Conv, Parse, Json |
| Collections | [collections/](collections/README.md) | Array, Dict |
| Numeric | [numeric/](numeric/README.md) | Math, Random |
| Result / Option | [result/](result/README.md) | Result, Option helpers |
| Concurrency | [concurrency/](concurrency/README.md) | Task (`Wait`, `WaitAll`) |
| Terminal UI | [tui/](tui/README.md) | Turbo Vision application facade ([upstream](https://github.com/aovestdipaperino/turbo-vision-4-rust)), controls, command dispatch |
| Graphics | [graph/](graph/README.md) | Window, drawing, hosted dispatch |
| Testing | [testing/](testing/README.md) | Std.Test assertions |

## Quick examples

### Console I/O

```pascal
uses Std.Console;

WriteLn('Hello!');
TextColorRGB(255, 160, 0);
WriteLn('Accent text');
NormVideo();
```

### TUI application shell

```pascal
uses Std.Tui;

procedure OnCommand(App: Application; CommandId: integer);
begin
  if CommandId = Command.Quit then
    Application.Quit(App)
end;

begin
  var App: Application := Application.Open();
  var DialogHandle: Dialog := Application.CreateDialog(
    App,
    record x := 10; y := 5; width := 40; height := 10 end,
    'Hello'
  );
  var OkButton: Button := Application.CreateButton(
    App,
    record x := 12; y := 7; width := 10; height := 1 end,
    '~O~k',
    Command.Accept
  );
  Application.AddChild(App, DialogHandle, OkButton);
  Application.OnCommand(App, OnCommand);
  Application.Run(App)
end.
```

See [Application](tui/app/README.md) for the full Turbo Vision API and [Handlers](tui/app/handlers.md) for command callbacks.

### Error handling helpers

```pascal
uses Std.Result, Std.Option;

var R: Result of integer, string := Ok(42);
WriteLn(Std.Result.Unwrap(R));
```

Language rules: [Error handling](../language/error-handling/README.md).

## Shared implementation touchpoints

When changing a `Std.*` API, update docs and:

- Intrinsic opcodes: [`crates/fpas-bytecode/src/intrinsic/mod.rs`](../../../crates/fpas-bytecode/src/intrinsic/mod.rs)
- Intrinsic dispatch: [`crates/fpas-std/src/intrinsics.rs`](../../../crates/fpas-std/src/intrinsics.rs)
- Types and `uses` registration: [`crates/fpas-sema/src/std_registry/`](../../../crates/fpas-sema/src/std_registry/mod.rs)

## See also

- [Units](../program-structure/units.md)
- [Concurrency](../language/concurrency/README.md)
- [Testing](testing/README.md)
