# Layout inspection

Read a GUI session snapshot to inspect the last successful presentation.
The snapshot contains the drawable size and the bounds of registered elements.

## Name a container

Set `Layout.Flow.Id` on a row, column or layered stack.
For a grid, set the ID on `Grids.Options.Flow`.
Use a nonempty, case-sensitive ID that is unique in the view.
A standalone control uses the stable ID supplied to its constructor.

## Read the bounds

1. Call `Session.Step()` to present a frame.
2. Call `Session.Snapshot()` on the GUI owner thread.
3. Find the desired ID in `Frame.Controls`.
4. Read `Control.Bounds` for the resolved X, Y, width and height in logical
   units. Read `Frame.Size` for the drawable area.

The host reports named container roles as `row`, `column`, `layers` and `grid`.
It reports a scroll view as `scroll`. Its value contains the vertical offset.
A control can have layout bounds while a scroll viewport clips its paint.
Before the first successful presentation, `Snapshot()` throws.
After a successful presentation, the returned descriptors are detached from
the native frame.

## See also

- [Container options](container-options.md)
- [Responsive layout](../../layout/responsive.md)
- [Scroll view](scroll-view.md)
- [Frame invalidation](../host.md#invalidation-and-inspection)
- [Low-level CUI inspection](../cui.md#layout-and-control-inspection)
- [GUI layout](README.md)

## Source files

- [Layout.fpas](../../../../src/Fpas.Ui/Layout.fpas)
- [Application snapshot types](../../../../src/Fpas.Ui.Gui/Application.fpas)
- [Container tracking](../../../../src/Fpas.Ui.Gui/Internal/Core/Context.fpas)
- [FrameControls.fpas](../../../../src/Fpas.Ui.Gui/Internal/Hosting/FrameControls.fpas)
- [Native box contract tests](../../../../tests/Fixtures/gui-host/NativeBoxContract.fpas)
