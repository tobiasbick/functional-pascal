// [Documentation](../../../docs/ui/gui/app-chrome/README.md)
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    // Composed bodies carry their own input permission. Restrict only the
    // container's native chrome so an admitted nested modal can receive input.
    private bool ContainerInput(Layout kind)
    {
        for (int index = layouts.Count - 1; index >= 0; index--)
            if (layouts[index].Kind == kind) return layouts[index].Data is true;
        throw new InvalidOperationException("The native container is not open.");
    }

    private ChromeInputScope RestrictChrome(Layout kind)
    {
        RestoreContainerInput(kind);
        return new(context);
    }

    // Native End consumes the scope originally established by Native Begin.
    // Managed Begin lifted that one scope before presenting composed bodies.
    private void RestoreContainerInput(Layout kind) =>
        NativeMethods.PushDisabled(context, ContainerInput(kind) ? (byte)0 : (byte)1);

    private readonly record struct ChromeInputScope(nint Context) : IDisposable
    {
        public void Dispose() => NativeMethods.PopDisabled(Context);
    }
}
