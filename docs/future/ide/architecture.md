# IDE architecture

The IDE is an ordinary `Std.Tui` Model-Update-View application. Application state
is immutable at the public boundary; `Update` returns the next model and `View`
builds a fresh element tree.

## Required platform additions

Two capabilities are missing and must be implemented before the application:

### `Std.Proc`

Add these public values:

```pascal
ProcessOutput = record
  ExitCode: integer;
  Stdout: string;
  Stderr: string;
end;

function CurrentExecutable(): result of string, string;
function RunCapture(Command: string; Args: array of string): result of ProcessOutput, string;
```

`CurrentExecutable` lets the IDE invoke the same `fpas` binary that launched it.
`RunCapture` waits for completion and captures both output streams. It has no
stdin, working-directory override, environment override, process handle, or
background mode in this plan.

### `Std.Tui`

Add a controlled multiline element:

```pascal
TuiElement.TextArea(
  Id: TuiControlId;
  Text: string;
  Caret: integer;
  Offset: TuiPoint;
  ChangeAction: TuiAction
)
```

Routing emits `TuiMsg.TextAreaChanged(Source, Action, Text, Caret, Offset)`.
Validation checks caret and non-negative offsets. Measurement allows the control
to fill its assigned slot. Arrangement remains host-owned. Paint clips lines to
the arranged bounds and renders the caret only when focused.

## IDE model

```text
IdeModel
├── Document
│   ├── Path: option of string
│   ├── Text: string
│   ├── SavedText: string
│   ├── Caret: integer
│   └── Offset: TuiPoint
├── Dialog: None | OpenPath | SavePath | ConfirmDirty(PendingCommand)
├── MessageText: string
├── LastExitCode: option of integer
└── Focused: option of TuiControlId
```

Dirty state is derived from `Text <> SavedText`; it is not independently
mutable. File and process failures update `MessageText` and keep the document.
Only a successful read or write updates `SavedText`.

## Target source layout

```text
apps/ide/
├── README.md
├── ide.fpasworkspace
├── ide-core.fpasprj
├── ide.fpasprj
└── src/
    ├── main.fpas                 — argument parsing and TuiApplication.Run
    ├── app/
    │   ├── model.fpas            — IdeModel, dialog and pending-command enums
    │   ├── actions.fpas          — stable control/action constructors
    │   ├── update.fpas           — message and command state transitions
    │   └── view.fpas             — fixed screen and modal element tree
    ├── document/
    │   ├── model.fpas            — document value and dirty calculation
    │   └── io.fpas               — UTF-8 open/save through Std.Fs
    └── process/
        └── runner.fpas           — check/run argument construction and output formatting
```

No file may combine filesystem I/O, process execution, and view construction.
Keep each file below roughly 400 lines and split before it reaches 500 lines.

## Dependency direction

```text
main → app/update → document/io
                  → process/runner
main → app/view   → app/model
                  → app/actions
document/io → document/model
all UI code → public Std.Tui only
```

Application units must not import private `Std.Tui.*` units. `ide-core.fpasprj`
exports the pure model, update, view, document, and process units needed by IDE
tests. `ide.fpasprj` contains only `src/main.fpas` and depends on `ide-core`.

## Test ownership

- Generic `TextArea` behavior belongs under `tests/stdlib/tui/`.
- `Std.Proc` behavior belongs in its owning Rust tests plus FPAS stdlib tests.
- IDE model/view/document tests belong under `tests/ide/`.
- `tests/ide/ide-tests.fpasprj` is a `kind = "test"` project with
  `dependencies.projects = ["../../apps/ide/ide-core.fpasprj"]`.
- Add `tests/ide/**/*_test.fpas` and the same `ide-core` project dependency to
  `tests/suite.fpasprj`, so the repository-wide test bundle includes the IDE.
- Add a Cargo suite shard that runs `tests/ide/ide-tests.fpasprj`, not the bare
  directory, so targeted tests receive their project dependency.
