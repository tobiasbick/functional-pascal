# Draft: Scripted input (test sidecar)

Format for driving interactive FPAS programs during `fpas test` without a real terminal or graph window. Sidecar files sit next to test sources or are referenced from the project manifest.

**File name:** `<test_basename>.script.toml`  
**Example:** `tui_escape_test.fpas` → `tui_escape_test.script.toml`

---

## Top-level shape

```toml
# Optional defaults for the whole script
[config]
headless_graph = false
# delay_ms between events (Phase 3+: usually 0 for determinism)
step_delay_ms = 0

# Ordered list of input injections before/during run
[[event]]
type = "readln"
line = "Alice"

[[event]]
type = "console_key"
kind = "Escape"
shift = false
ctrl = false
alt = false

[[event]]
type = "console_mouse"
action = "Down"
button = "Left"
x = 10
y = 5

[[event]]
type = "console_resize"
width = 120
height = 40

[[event]]
type = "console_paste"
text = "hello"

[[event]]
type = "graph_key"
kind = "Escape"

[[event]]
type = "graph_mouse"
action = "Down"
button = "Left"
x = 32
y = 48
shift = false
ctrl = false
alt = false
```

Events are applied **in order** before `vm.run()` unless `timing = "before_run" | "inline"` is added in a later revision. Phase 3 applies all events upfront (sufficient for poll/TUI loops that drain the queue).

---

## Event types

### Line-oriented stdin

| Field | Type | Maps to |
|-------|------|---------|
| `type = "readln"` | | |
| `line` | string | `Vm::push_readln_input` |

### Raw character keys (`ReadKey`)

| Field | Type | Maps to |
|-------|------|---------|
| `type = "readkey_chars"` | | |
| `chars` | string | `Vm::push_readkey_input` |

### Console unified events (`ReadEvent`, TUI host)

| `type` | Required fields | Rust mapping |
|--------|-----------------|--------------|
| `console_key` | `kind` (`Escape`, `Enter`, `Character`, …), optional `ch`, modifier booleans | `ConsoleEvent::key(...)` |
| `console_mouse` | `action`, `button`, `x`, `y`, modifiers | `ConsoleEvent::mouse(...)` |
| `console_resize` | `width`, `height` | `ConsoleEvent::resize(...)` |
| `console_paste` | `text` | `ConsoleEvent::paste(...)` |
| `console_focus_gained` | — | `ConsoleEvent::focus_gained()` |
| `console_focus_lost` | — | `ConsoleEvent::focus_lost()` |

`kind`, `action`, and `button` use the same names as [`docs/pascal/std/console.md`](../../pascal/std/console.md) enums (`KeyKind.*`, mouse action/button variants).

### Graph application events

Requires runner to enable headless graph backend for the test run.

| `type` | Fields | Rust mapping |
|--------|--------|--------------|
| `graph_key` | `kind`, `ch`, modifiers | `GraphEvent` key variant |
| `graph_mouse` | `action`, `button`, `x`, `y`, modifiers | `GraphEvent::Mouse` |
| `graph_wheel` | `delta_x`, `delta_y`, modifiers | `GraphEvent` wheel variant |

Exact field set aligns with existing `GraphEvent` in `fpas-std` and compiler test helpers in [`graph.rs`](../../../crates/fpas-compiler/src/tests/std_library/graph.rs).

---

## Example: TUI quit on Escape

**`tui_escape_test.fpas`**

```pascal
program TuiEscapeTest;
uses Std.Console, Std.Tui, Std.Test;

mutable var QuitSeen: boolean := false;

procedure OnPaint(App: Application);
begin
end;

function OnKeyPressed(App: Application; Key: KeyEvent): boolean;
begin
  if Key.kind = KeyKind.Escape then
  begin
    QuitSeen := true;
    Application.HostRequestQuit(App);
    return true
  end;
  return false
end;

begin
  var App: Application := Application.Open();
  var Handlers: ApplicationHandlers := record
    OnPaint := OnPaint;
    OnKeyPressed := Some(OnKeyPressed);
  end;
  Application.Configure(App, Handlers);
  Application.Run(App);
  AssertTrue(QuitSeen)
end.
```

**`tui_escape_test.script.toml`**

```toml
[[event]]
type = "console_key"
kind = "Escape"
```

The runner pushes one Escape key event into the console queue; the hosted TUI loop consumes it and invokes `OnKey`.

---

## Example: ReadLn + assert output

**`greet_test.fpas`** — program under test inlined or imported from app unit.

**`greet_test.script.toml`**

```toml
[[event]]
type = "readln"
line = "World"
```

Runner may additionally support golden stdout file `greet_test.expect.stdout` (Phase 2 runner feature, not sidecar):

```text
Hello, World
Goodbye
```

---

## Validation rules

- Unknown `type` → script parse error before any test runs
- Negative or zero resize dimensions → error (match console validation)
- Graph events without `headless_graph = true` (config or auto-detect) → warning or error
- Empty `[[event]]` list → valid (non-interactive test)

---

## Parser implementation

```text
crates/fpas-cli/src/test_script/
 ├── mod.rs       — public apply_script(vm, path)
 ├── parse.rs     — TOML → Vec<ScriptEvent>
 └── apply.rs     — ScriptEvent → Vm queue methods
```

Reuse `ConsoleEvent` / `GraphEvent` / `ConsoleKeyEvent` builders from `fpas-std`; do not duplicate enum string tables (import `key_kind_index`, etc.).

---

## Future extensions

- `[[event]]` with `at = "after_ms"` for timed injection (needs VM hook to dequeue during run)
- Include/import other script files
- Record/replay from manual session (out of scope initially)
