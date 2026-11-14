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
    /// <param name="flags">flags supplied to the native operation.</param>
    /// <returns>The native result.</returns>
    public unsafe bool BeginNodeEditor(string id, CuiNodeView view, int widthKind, float width, int heightKind, float height, float minimum, float maximum, int flags)
    {
        CheckFrame();
        if (NativeMethods.NodeEditorBegin(context, Stable(id), view, widthKind, width, heightKind, height, minimum, maximum, flags) == 0)
            return false;
        layouts.Add(new(Layout.NodeEditor));
        return true;
    }

    /// <summary>Closes the native EndNodeEditor operation.</summary>
    /// <returns>The native result.</returns>
    public unsafe CuiNodeView EndNodeEditor()
    {
        RequireLayout(Layout.NodeEditor);
        NativeMethods.NodeEditorEnd(context, out var result);
        layouts.RemoveAt(layouts.Count - 1);
        return result;
    }

    /// <summary>Opens the native BeginNode operation.</summary>
    /// <param name="id">id supplied to the native operation.</param>
    /// <param name="x">x supplied to the native operation.</param>
    /// <param name="y">y supplied to the native operation.</param>
    /// <param name="title">title supplied to the native operation.</param>
    /// <param name="width">width supplied to the native operation.</param>
    /// <param name="icon">icon supplied to the native operation.</param>
    public unsafe void BeginNode(int id, ref float x, ref float y, string title, float width, string? icon)
    {
        CheckFrame();
        NativeMethods.NodeBegin(context, id, ref x, ref y, title, width, icon);
    }

    /// <summary>Invokes the native NodePin operation.</summary>
    /// <param name="id">id supplied to the native operation.</param>
    /// <param name="label">label supplied to the native operation.</param>
    /// <param name="output">output supplied to the native operation.</param>
    /// <param name="type">type supplied to the native operation.</param>
    public unsafe void NodePin(int id, string label, bool output, int type)
    {
        CheckFrame();
        NativeMethods.NodePin(context, id, label, output ? 1 : 0, type);
    }

    /// <summary>Invokes the native NodeLink operation.</summary>
    /// <param name="id">id supplied to the native operation.</param>
    /// <param name="fromPin">fromPin supplied to the native operation.</param>
    /// <param name="toPin">toPin supplied to the native operation.</param>
    public unsafe void NodeLink(int id, int fromPin, int toPin)
    {
        CheckFrame();
        _ = NativeMethods.NodeLink(context, id, fromPin, toPin);
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
