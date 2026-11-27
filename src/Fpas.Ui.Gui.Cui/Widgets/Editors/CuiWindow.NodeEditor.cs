// [Documentation](../../../../docs/ui/gui/canvas-and-bars/node-editor.md)
using Fpas.Ui.Gui.Cui.Widgets.Editors;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Opens the native BeginNodeEditor operation.</summary>
    /// <param name="id">id supplied to the native operation.</param>
    /// <param name="view">view supplied to the native operation.</param>
    /// <param name="widthKind">widthKind supplied to the native operation.</param>
    /// <param name="width">width supplied to the native operation.</param>
    /// <param name="heightKind">heightKind supplied to the native operation.</param>
    /// <param name="height">height supplied to the native operation.</param>
    /// <param name="minimum">minimum supplied to the native operation.</param>
    /// <param name="maximum">maximum supplied to the native operation.</param>
    /// <param name="flags">Native editor chrome input/appearance flags. Composed bodies supply their own input permission.</param>
    /// <returns>The native result.</returns>
    public unsafe bool BeginNodeEditor(string id, CuiNodeView view, int widthKind, float width, int heightKind, float height, float minimum, float maximum, int flags)
    {
        CheckFrame();
        if (NativeMethods.NodeEditorBegin(context, Stable(id), view, widthKind, width, heightKind, height, minimum, maximum, flags) == 0)
            return false;
        NativeMethods.PopDisabled(context);
        layouts.Add(new(Layout.NodeEditor, Data: (flags & 1) != 0));
        return true;
    }

    /// <summary>Closes the native EndNodeEditor operation.</summary>
    /// <returns>The native result.</returns>
    public unsafe CuiNodeView EndNodeEditor()
    {
        RequireLayout(Layout.NodeEditor);
        RestoreContainerInput(Layout.NodeEditor);
        NativeMethods.NodeEditorEnd(context, out var result);
        layouts.RemoveAt(layouts.Count - 1);
        return result;
    }

    /// <summary>Opens a node. Call EndNode for each opened node.</summary>
    /// <param name="id">Stable node ID within the editor.</param>
    /// <param name="x">Canvas X coordinate, updated when the node moves.</param>
    /// <param name="y">Canvas Y coordinate, updated when the node moves.</param>
    /// <param name="title">Visible node title.</param>
    /// <param name="width">Node width in canvas units, or zero for the native default.</param>
    /// <param name="icon">Optional title glyph.</param>
    /// <returns>Whether CUI reports this node as selected.</returns>
    public unsafe bool BeginNode(int id, ref float x, ref float y, string title, float width, string? icon)
    {
        CheckFrame();
        using var input = RestrictChrome(Layout.NodeEditor);
        return NativeMethods.NodeBegin(context, id, ref x, ref y, title, width, icon) != 0;
    }

    /// <summary>Draws an input or output pin in the open node.</summary>
    /// <param name="id">Stable pin ID within the editor.</param>
    /// <param name="label">Visible pin caption.</param>
    /// <param name="output">Whether the pin is an output.</param>
    /// <param name="type">Native pin compatibility type.</param>
    /// <returns>Whether CUI reports a link on this pin.</returns>
    public unsafe bool NodePin(int id, string label, bool output, int type)
    {
        CheckFrame();
        using var input = RestrictChrome(Layout.NodeEditor);
        return NativeMethods.NodePin(context, id, label, output ? 1 : 0, type) != 0;
    }

    /// <summary>Reads a node selection after submitting every node in the editor.</summary>
    /// <param name="id">Stable node ID within the open editor.</param>
    /// <returns>Whether the current editor selection contains the node.</returns>
    public unsafe bool IsNodeSelected(int id)
    {
        RequireLayout(Layout.NodeEditor);
        return NativeMethods.NodeSelected(context, id) != 0;
    }

    /// <summary>Draws a link between two pins.</summary>
    /// <param name="id">Stable link ID within the editor.</param>
    /// <param name="fromPin">Output pin ID.</param>
    /// <param name="toPin">Input pin ID.</param>
    /// <returns>Whether CUI reports this link as selected.</returns>
    public unsafe bool NodeLink(int id, int fromPin, int toPin)
    {
        CheckFrame();
        using var input = RestrictChrome(Layout.NodeEditor);
        return NativeMethods.NodeLink(context, id, fromPin, toPin) != 0;
    }

    /// <summary>Invokes the native NodeEvent operation.</summary>
    /// <param name="kind">kind supplied to the native operation.</param>
    /// <param name="first">first supplied to the native operation.</param>
    /// <param name="second">second supplied to the native operation.</param>
    /// <returns>The native result.</returns>
    public unsafe bool NodeEvent(int kind, out int first, out int second)
    {
        CheckFrame();
        return NativeMethods.NodeEvent(context, kind, out first, out second) != 0;
    }

    /// <summary>Closes the native EndNode operation.</summary>
    public unsafe void EndNode()
    {
        CheckFrame();
        using var input = RestrictChrome(Layout.NodeEditor);
        NativeMethods.NodeEnd(context);
    }

    /// <summary>Opens the native BeginNodeContent operation.</summary>
    public unsafe void BeginNodeContent()
    {
        CheckFrame();
        NativeMethods.NodeContentBegin(context);
    }

    /// <summary>Closes the native EndNodeContent operation.</summary>
    public unsafe void EndNodeContent()
    {
        CheckFrame();
        NativeMethods.NodeContentEnd(context);
    }
}
