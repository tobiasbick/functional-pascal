# GUI counter

A complete FPAS application using `Fpas.Gui.Application`, shared MVU transitions
and declarative widgets. Build or run `GuiCounter.fpasproj` with `dotnet build` or
`dotnet run --project`. Source-build prerequisites are in the
[native build reference](../../../src/Fpas.Gui.Cui/native/README.md#prerequisites).

Increment updates the model. The checkbox and Unicode editor are controlled by
that same model. Start requests an asynchronous increment; Cancel invalidates its
result while the window remains usable. Show modal demonstrates exclusive input,
Escape dismissal and focus restoration. Closing cancels and joins outstanding
work before destroying native resources.

See the [gallery](../gallery/) for the full graphical widget inventory and the
[host reference](../../../src/Fpas.Gui/README.md) for lifecycle and snapshot contracts.
