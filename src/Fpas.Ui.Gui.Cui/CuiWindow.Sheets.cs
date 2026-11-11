using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Begins a native edge sheet, including its outgoing transition.</summary>
    /// <param name="id">Stable sheet identity.</param>
    /// <param name="title">Visible heading.</param>
    /// <param name="description">Optional line under the heading.</param>
    /// <param name="side">Right, left, bottom, or top, in native enum order.</param>
    /// <param name="size">Logical extent along the edge axis.</param>
    /// <param name="open">Model-owned visibility.</param>
    /// <param name="enabled">Whether native dismissal input is admitted.</param>
    /// <param name="focus">Whether to focus the built-in close button.</param>
    /// <param name="closeControl">Snapshot identity for the close button when open.</param>
    /// <returns>Draw contents for bit 1 and propose dismissal for bit 2.</returns>
    public int BeginSheet(string id, string title, string description, int side, float size,
        bool open, bool enabled, bool focus, string closeControl)
    {
        CheckFrame();
        Utf8.Validate(title, nameof(title));
        Utf8.Validate(description, nameof(description));
        if (side is < 0 or > 3) throw new ArgumentOutOfRangeException(nameof(side));
        if (!float.IsFinite(size) || size <= 0) throw new ArgumentOutOfRangeException(nameof(size));
        if (layouts.Count(static scope => scope.Kind == Layout.Sheet) >= 8)
            throw new InvalidOperationException("CUI supports up to eight nested modal scopes.");

        int baseNode = NativeMethods.LastNode(context);
        int result = NativeMethods.SheetBegin(context, Stable(id), title, description.Length == 0 ? null : description,
            side, size, open ? 1 : 0, enabled ? 1 : 0, focus ? 1 : 0);
        if ((result & 1) != 0)
        {
            TrackModal(id);
            layouts.Add(new(Layout.Sheet, Node: baseNode));
            if (open) TrackLastControl(closeControl);
        }
        return result;
    }

    /// <summary>Finishes a sheet's contents and restores the base layout target.</summary>
    public void EndSheet()
    {
        var scope = RequireLayout(Layout.Sheet);
        NativeMethods.SheetEnd(context, scope.Node);
        layouts.RemoveAt(layouts.Count - 1);
    }
}
