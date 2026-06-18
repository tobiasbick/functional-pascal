# Standard library reference (`Std.*`)

Built-in units under the reserved `Std` namespace. Import them via `uses` and refer to symbols by short name or fully qualified name:

```pascal
program Hello;

uses
  Std.Console,
  Std.Math;

begin
  WriteLn(Sqrt(16.0));                           { short name }
  Std.Console.WriteLn(Std.Math.Sqrt(16.0));      { fully qualified }
end.
```

See [Units](../program-structure/units.md) for `uses` rules, short name ambiguity, and the reserved `Std` namespace.

Each unit page is a **self-contained handbook**: importing and short vs qualified names, a **quick reference** table, then **every** routine (and types where applicable) with parameters, behavior, edge cases, and a **small example**. All units are opt-in through `uses`.

## Unit overview

| Unit | Purpose | Key symbols | Details |
|------|---------|-------------|---------|
| [`Std.Console`](console.md) | Console I/O, CRT screen control, terminal events | `Write`, `WriteLn`, `ReadLn`, `ClrScr`, `GotoXY`, `TextColorRGB`, `TextColor256`, `ReadEvent`, … | [console.md](console.md) |
| [`Std.Args`](args.md) | Process arguments after `--` | `ParamCount`, `ParamStr` | [args.md](args.md) |
| [`Std.Env`](env.md) | Process environment access | `Get`, `Exists` | [env.md](env.md) |
| [`Std.Proc`](proc.md) | Blocking host process execution | `Run` | [proc.md](proc.md) |
| [`Std.Path`](path.md) | Pure path manipulation | `Join`, `BaseName`, `DirName`, `Extension`, `Normalize` | [path.md](path.md) |
| [`Std.Fs`](fs.md) | Blocking filesystem I/O | `ReadText`, `WriteText`, `Exists`, `IsFile`, `IsDir`, `CreateDir` | [fs.md](fs.md) |
| [`Std.Time`](time.md) | Wall-clock and monotonic time | `TimestampMillis`, `MonotonicMillis`, `ElapsedMillis`, `Sleep` | [time.md](time.md) |
| [`Std.Graph`](graph.md) | Native windowed graphics | `Application.Open`, `Application.UploadFrame`, `Application.DrawLine`, `Application.DrawText`, `Event`, `EventKind`, … — hosted dispatch in [graph-app.md](graph-app.md) | [graph.md](graph.md) |
| [`Std.Tui`](tui.md) | Terminal application structure | `Application.Open`, `Application.Configure`, `Application.Run`, `On*` handlers, host widgets — [tui-app.md](tui-app.md) (VM intrinsics **256**–**347**; see [TUI framework](../../future/tui-application-framework.md)) | [tui.md](tui.md) |
| [`Std.Str`](str.md) | String operations | `Length`, `ToUpper`, `Contains`, `Split`, `Join`, `PadLeft`, `CharAt`, `Ord`, `Chr`, `Format`, … | [str.md](str.md) |
| [`Std.Conv`](conv.md) | Type conversions | `IntToStr`, `StrToInt`, `RealToStr`, `BoolToStr`, `IntToHex`, … | [conv.md](conv.md) |
| [`Std.Parse`](parse.md) | Structured text parsing | `TryInt`, `TryReal`, `TryBool` | [parse.md](parse.md) |
| [`Std.Json`](json.md) | JSON parse and stringify | `JsonValue`, `Parse`, `Stringify` | [json.md](json.md) |
| [`Std.Math`](math.md) | Mathematical functions | `Pi`, `Abs`, `Sqrt`, `Pow`, `Sin`, `Cos`, `Tan`, `Log`, `Exp`, `Clamp`, … | [math.md](math.md) |
| [`Std.Random`](random.md) | Pseudo-random numbers | `Random`, `RandomInt`, `Randomize` | [random.md](random.md) |
| [`Std.Array`](array.md) | Array helpers | `Length`, `Push`, `Pop`, `Sort`, `Slice`, `Map`, `Filter`, `Reduce`, `Find`, `Any`, `All`, … | [array.md](array.md) |
| [`Std.Dict`](dict.md) | Dictionary helpers | `Length`, `ContainsKey`, `Keys`, `Values`, `Remove`, `Get`, `Merge`, `Map`, `Filter` | [dict.md](dict.md) |
| [`Std.Result`](result.md) | Result helpers | `Unwrap`, `UnwrapOr`, `IsOk`, `IsError`, `Map`, `AndThen`, `OrElse` | [result.md](result.md) |
| [`Std.Option`](option.md) | Option helpers | `Unwrap`, `UnwrapOr`, `IsSome`, `IsNone`, `Map`, `AndThen`, `OrElse` | [option.md](option.md) |
| [`Std.Task`](task.md) | Task synchronization | `Wait`, `WaitAll` | [task.md](task.md) |
| [`Std.Test`](test.md) | Test assertions | `AssertTrue`, `AssertFalse`, `AssertEquals`, `Fail`, `Skip` | [test.md](test.md) |

### Hosted application pages

| Topic | Page |
|-------|------|
| TUI hosted dispatch (`Application.Configure`, `Application.Run`, `On*`, host widgets) | [tui-app.md](tui-app.md) |
| Graph hosted dispatch (same hosted model as TUI) | [graph-app.md](graph-app.md) |
| TUI real-terminal manual smoke tests | [tui-terminal-checklist.md](tui-terminal-checklist.md) |

## Quick examples

### Console I/O

```pascal
uses Std.Console;

WriteLn('Hello!');
TextColorRGB(255, 160, 0);
WriteLn('Accent text');
NormVideo();
WriteLn('Value: ', 42, ' Flag: ', true);
```

### TUI application shell

```pascal
uses Std.Console, Std.Tui;

procedure OnPaint(App: Application);
begin
  ClrScr();
  WriteLn('Press Escape to exit')
end;

function OnKeyPressed(App: Application; Key: Std.Console.KeyEvent): boolean;
begin
  if Key.kind = KeyKind.Escape then
  begin
    Application.HostRequestQuit(App);
    return true
  end;
  return false
end;

begin
  var App: Application := Application.Open();
  Application.Configure(App, record
    OnPaint := OnPaint;
    OnKeyPressed := Some(OnKeyPressed)
  end);
  Application.Run(App)
end.
```

### Native graphics shell

```pascal
uses Std.Console, Std.Graph;

procedure OnPaint(App: Application);
begin
  Application.Clear(App, $00000020);
  Application.DrawText(App, 8, 8, 'FPAS', $00FFFFFF);
  Application.Present(App)
end;

begin
  var App: Application := Application.Open(320, 200, 'Graph');
  Application.Configure(App, record OnPaint := OnPaint end);
  Application.Run(App)
end.
```

### Strings and conversions

```pascal
uses Std.Str, Std.Conv;

var Upper: string := ToUpper('hello');
var N: integer := StrToInt('42');
var Msg: string := Format('value=%d (%s)', N, Upper);
```

### Math

```pascal
uses Std.Math;

var Root: real := Sqrt(16.0);
var A: integer := Abs(-5);
```

### Random

```pascal
uses Std.Random;

Randomize();
var Die: integer := RandomInt(1, 6);
```

### Arrays

```pascal
uses Std.Array;

mutable var Nums: array of integer := [3, 1, 4];
Push(Nums, 9);
var Sorted: array of integer := Sort(Nums);
```

### Error handling (Result and Option)

See [Error handling](../language/error-handling/README.md) for the `Result of T, E` and `Option of T` types, constructors (`Ok`, `Error`, `Some`, `None`), the `try` operator, and `case` destructuring.

```pascal
uses Std.Result, Std.Option;

var R: Result of integer, string := Ok(42);
WriteLn(Std.Result.Unwrap(R));

var O: Option of integer := Some(7);
WriteLn(Std.Option.UnwrapOr(O, -1));
```

## Shared implementation touchpoints

When changing a `Std.*` API, update both docs and implementation wiring.
These files are the usual integration points:

- Intrinsic opcodes: [`crates/fpas-bytecode/src/intrinsic/mod.rs`](../../../crates/fpas-bytecode/src/intrinsic/mod.rs)
- Intrinsic dispatch (non-console): [`crates/fpas-std/src/intrinsics.rs`](../../../crates/fpas-std/src/intrinsics.rs)
- Pascal types and `uses` registration: [`crates/fpas-sema/src/std_registry/`](../../../crates/fpas-sema/src/std_registry/mod.rs)

## See also

- [Units](../program-structure/units.md)
- [Concurrency](../language/concurrency/README.md) — `go` and `Std.Task`
- [Test framework](../../future/test-framework/README.md) — `fpas test` runner
