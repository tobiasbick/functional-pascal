using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Marks a zero-height destination in the enclosing viewport.</summary>
    /// <param name="id">Globally unique anchor identity.</param>
    public void ScrollAnchor(string id)
    {
        CheckFrame();
        ArgumentException.ThrowIfNullOrEmpty(id);
        Utf8.Validate(id, nameof(id));
        NativeMethods.ScrollAnchor(context, id);
    }
}
