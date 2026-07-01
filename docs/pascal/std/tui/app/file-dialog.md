# Std.Tui file dialog

`Application.RunFileDialog` shows a modal Turbo Vision file selection dialog backed by upstream `FileDialog`.

| Symbol | Description |
| --- | --- |
| `Application.RunFileDialog(App, Bounds, Title, Wildcard, StartPath): Option of string` | Open the dialog and return the selected file path, or `None` when canceled. |
| `Application.TestSetFileDialogResult(App, Result)` | Headless tests only: queue the result for the next `RunFileDialog` call. |

## Parameters

| Parameter | Meaning |
| --- | --- |
| `Bounds` | Dialog rectangle in terminal cells. |
| `Title` | Dialog title text. |
| `Wildcard` | File filter. Use `'*'` for all files or patterns such as `'*.fpas'`. Directories are always listed. |
| `StartPath` | `Some(directory)` to open in that folder, or `None` for the process current directory. |

## Wildcards

| Pattern | Effect |
| --- | --- |
| `'*'` | All files |
| `'*.fpas'` | Files with the `.fpas` extension |
| `'test'` | Files whose names contain `test` |

## Headless tests

Interactive terminal code calls upstream `FileDialog::execute`. Headless `Application.OpenForTest` sessions do not open a real modal loop; call `Application.TestSetFileDialogResult` before `Application.RunFileDialog` to assert accept or cancel behavior.

## See Also

- [Application](README.md)
- [Native testing](testing.md)
- [Dialogs and windows](modals.md)
