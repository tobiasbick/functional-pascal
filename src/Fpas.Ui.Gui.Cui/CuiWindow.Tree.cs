using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

/// <summary>Native expansion and activation proposals.</summary>
/// <param name="Open">Whether CUI opened the children.</param>
/// <param name="Clicked">Whether the native row was activated.</param>
public readonly record struct CuiTreeResult(bool Open, bool Clicked);

public sealed partial class CuiWindow
{
    /// <summary>Draws a native tree row and opens a child scope when expanded.</summary>
    /// <param name="id">Stable row identity.</param>
    /// <param name="label">Literal caption.</param>
    /// <param name="icon">Optional glyph replacing the file or folder icon.</param>
    /// <param name="leaf">Whether the row has no child scope.</param>
    /// <param name="open">Current model expansion.</param>
    /// <param name="selected">Whether to show selection.</param>
    /// <param name="enabled">Whether the row accepts input.</param>
    /// <param name="focus">Whether to request keyboard focus.</param>
    /// <returns>Native expansion and activation.</returns>
    public CuiTreeResult BeginTree(string id, string label, string icon, bool leaf, bool open,
        bool selected, bool enabled, bool focus)
    {
        CheckFrame();
        Utf8.Validate(label, nameof(label));
        Utf8.Validate(icon, nameof(icon));
        int expanded = open ? 1 : 0;
        int clicked = NativeMethods.TreeBegin(context, Stable(id), label, Optional(icon), leaf ? 1 : 0,
            selected ? 1 : 0, enabled ? 1 : 0, focus ? 1 : 0, ref expanded, out int node);
        if (expanded != 0) layouts.Add(new(Layout.Tree, Node: node));
        return new(expanded != 0, clicked != 0);
    }

    /// <summary>Closes the child scope of an expanded tree row.</summary>
    public void EndTree()
    {
        var scope = RequireLayout(Layout.Tree);
        NativeMethods.TreeEnd(context, scope.Node);
        layouts.RemoveAt(layouts.Count - 1);
    }
}
