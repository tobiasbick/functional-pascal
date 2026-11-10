using Fpas.Ui.Gui.Cui.Interop;
using Fpas.Ui.Gui.Cui.Text;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws a native hairline separator in the current container.</summary>
    public void Separator() { CheckFrame(); NativeMethods.Separator(context); }
    /// <summary>Measures literal text for shared table-column content sizing.</summary>
    /// <param name="text">Strict Unicode text.</param>
    /// <returns>Native logical width.</returns>
    public float MeasureText(string text) { CheckFrame(); Utf8.Validate(text, nameof(text)); return NativeMethods.MeasureText(context, text); }
    private readonly Dictionary<string, (int Mode, float Offset)> scrollRequests = [];
    private readonly Dictionary<string, int> scrollNodes = [];
    private readonly Dictionary<string, uint> modalKeys = [];
    private readonly HashSet<string> usedModals = [];
    /// <summary>Begins a declarative traversal with host-owned modal admission.</summary>
    public void BeginHostView() { CheckFrame(); NativeMethods.HostBegin(context); }

    /// <summary>Reads keyboard focus before reconstructing this frame's controls.</summary>
    /// <param name="id">Previously tracked identity.</param>
    /// <returns>Whether it currently owns focus.</returns>
    public bool WasFocused(string id) { CheckFrame(); return HasRetainedFocus(id); }

    internal bool HasRetainedFocus(string id)
    { CheckOpen(); return controlKeys.TryGetValue(id, out uint key) && NativeMethods.FocusKey(context, key) != 0; }

    internal void RestoreControlFocus(string id)
    { CheckOpen(); NativeMethods.RequestFocus(context, Stable(id)); }

    /// <summary>Reports Enter or double-click activation separately from selection.</summary>
    public bool IsActivation { get { CheckFrame(); return NativeMethods.Activate(context) != 0; } }

    /// <summary>Opens a selectable row whose cells are drawn by the caller.</summary>
    /// <param name="id">Stable identity.</param>
    /// <param name="enabled">Input permission.</param>
    /// <param name="selected">Model selection.</param>
    /// <param name="focus">Focus request.</param>
    /// <returns>Native event bits.</returns>
    public int BeginChoiceRow(string id, bool enabled, bool selected, bool focus)
    {
        CheckFrame();
        int node = NativeMethods.ChoiceRow(context, Stable(id), enabled ? 1 : 0, selected ? 1 : 0, focus ? 1 : 0, out int events);
        layouts.Add(new(Layout.Group, Node: node));
        return events;
    }
    private static string Stable(string id)
    {
        Utf8.Validate(id, nameof(id));
        ArgumentException.ThrowIfNullOrEmpty(id);
        return "###" + Convert.ToBase64String(Utf8.Encoding.GetBytes(id));
    }

    /// <summary>Draws an adapter control with literal text and a separate stable identity.</summary>
    /// <param name="id">Identity independent of label text.</param>
    /// <param name="text">Literal visible label.</param>
    /// <param name="kind">Button=0, checkbox=1, choice=2, tab=3.</param>
    /// <param name="enabled">Whether input is accepted.</param>
    /// <param name="selected">Controlled checked/selected value.</param>
    /// <param name="variant">Shared semantic button variant.</param>
    /// <param name="focus">Whether to request focus.</param>
    /// <returns>Activation bit 1 and checked-value bit 2.</returns>
    public int Control(string id, string text, int kind, bool enabled, bool selected, int variant, bool focus)
    {
        CheckFrame(); Utf8.Validate(text, nameof(text));
        ArgumentOutOfRangeException.ThrowIfNegative(kind);
        ArgumentOutOfRangeException.ThrowIfGreaterThan(kind, 3);
        ArgumentOutOfRangeException.ThrowIfNegative(variant);
        ArgumentOutOfRangeException.ThrowIfGreaterThan(variant, 4);
        return NativeMethods.Control(context, Stable(id), text, kind, enabled ? 1 : 0, selected ? 1 : 0, variant, focus ? 1 : 0);
    }

    /// <summary>Opens a clipped host layout container.</summary>
    /// <param name="horizontal">Whether its main axis is horizontal.</param>
    /// <param name="padding">Logical inset.</param>
    /// <param name="gap">Logical separation.</param>
    /// <param name="alignment">Stretch=0, start=1, center=2, end=3.</param>
    /// <param name="framed">Whether to draw a border.</param>
    public void BeginGroup(bool horizontal, float padding, float gap, int alignment, bool framed)
    {
        CheckLayoutOptions(padding, gap);
        ArgumentOutOfRangeException.ThrowIfNegative(alignment);
        ArgumentOutOfRangeException.ThrowIfGreaterThan(alignment, 3);
        int node = NativeMethods.GroupBegin(context, horizontal ? 1 : 0, padding, gap, alignment, framed ? 1 : 0);
        layouts.Add(new(Layout.Group, Node: node));
    }

    /// <summary>Closes the innermost host group and makes it the last widget.</summary>
    public void EndGroup()
    {
        var group = RequireLayout(Layout.Group);
        NativeMethods.GroupEnd(context, group.Node);
        layouts.RemoveAt(layouts.Count - 1);
    }

    /// <summary>Applies shared sizing to the last widget.</summary>
    /// <param name="horizontal">Parent main axis.</param>
    /// <param name="kind">Content=0, fill=1, fixed=2.</param>
    /// <param name="basis">Fixed logical basis.</param>
    /// <param name="minimum">Minimum extent.</param>
    /// <param name="maximum">Maximum extent or -1.</param>
    /// <param name="grow">Growth weight.</param>
    /// <param name="alignment">Cross-axis placement.</param>
    /// <param name="shrink">Nonnegative reduction weight.</param>
    public void SizeLast(bool horizontal, int kind, float basis, float minimum, float maximum, float grow, int alignment, float shrink = 1)
    {
        CheckFrame();
        if (!float.IsFinite(basis) || basis < 0 || !float.IsFinite(minimum) || minimum < 0 ||
            !float.IsFinite(maximum) || maximum < -1 || maximum >= 0 && maximum < minimum ||
            !float.IsFinite(grow) || grow < 0 || !float.IsFinite(shrink) || shrink < 0 || kind is < 0 or > 2 || alignment is < 0 or > 3)
            throw new ArgumentOutOfRangeException(nameof(basis), "Invalid shared layout constraints.");
        int node = NativeMethods.LastNode(context);
        flexNodes[node] = new FlexNode { Node = node, Horizontal = horizontal ? 1 : 0, Kind = kind, Basis = basis,
            Minimum = minimum, Maximum = maximum, Grow = grow, Shrink = shrink };
        NativeMethods.NodeSize(context, node, horizontal ? 1 : 0, kind, basis, minimum, maximum, 0, alignment);
    }

    /// <summary>Applies a semantic theme role to the last widget.</summary>
    /// <param name="role">Shared role ordinal.</param>
    public void StyleLast(int role) { CheckFrame(); NativeMethods.NodeRole(context, NativeMethods.LastNode(context), role); }

    /// <summary>Draws a controlled editor with focus and disabled state.</summary>
    /// <param name="id">Stable identity.</param>
    /// <param name="buffer">Owned text and selection.</param>
    /// <param name="multiline">Whether line breaks are accepted.</param>
    /// <param name="enabled">Input permission.</param>
    /// <param name="focus">Focus request.</param>
    /// <param name="placeholder">Hint displayed for an empty document.</param>
    /// <returns>Native editor events.</returns>
    public CuiTextInputEvents Editor(string id, CuiTextBuffer buffer, bool multiline, bool enabled, bool focus, string placeholder = "")
    {
        CheckFrame();
        Utf8.Validate(placeholder, nameof(placeholder));
        string nativeId = Stable(id);
        NativeMethods.EditorOptions(context, nativeId, enabled ? 1 : 0, focus ? 1 : 0);
        try { return DrawTextEditor(nativeId, buffer, multiline ? 96 : 0, placeholder); }
        finally { NativeMethods.PopDisabled(context); }
    }

    /// <summary>Reads focus from a tracked node after presentation.</summary>
    /// <param name="id">Tracked identity.</param>
    /// <returns>Whether the control has native focus.</returns>
    public bool ControlFocused(string id) { CheckOpen(); return NativeMethods.Focused(context, trackedNodes[id]) != 0; }

    /// <summary>Reads hover from a tracked node after presentation.</summary>
    /// <param name="id">Tracked identity.</param>
    /// <returns>Whether the control is hovered.</returns>
    public bool ControlHovered(string id) { CheckOpen(); return NativeMethods.Hovered(context, trackedNodes[id]) != 0; }

    /// <summary>Reads or reconciles a scroll view target.</summary>
    /// <param name="id">Stable identity.</param>
    /// <param name="mode">Read-only=-1, offset=0, end=1.</param>
    /// <param name="offset">Requested offset.</param>
    /// <param name="current">Previous target.</param>
    /// <param name="maximum">Current scroll range.</param>
    public void ScrollState(string id, int mode, float offset, out float current, out float maximum)
    {
        CheckOpen();
        NativeMethods.ScrollState(context, Stable(id), mode, offset, out current, out maximum);
        if (mode >= 0) { CheckFrame(); scrollRequests[id] = (mode, offset); }
    }

    /// <summary>Opens a host-controlled scroll view.</summary>
    /// <param name="id">Stable identity.</param>
    /// <param name="showScrollbar">Whether to show its scrollbar.</param>
    /// <param name="enabled">Whether user input is accepted.</param>
    public void BeginViewport(string id, bool showScrollbar, bool enabled = true)
    {
        CheckFrame();
        int node = NativeMethods.Viewport(context, Stable(id), showScrollbar ? 1 : 0, enabled ? 1 : 0);
        scrollNodes[id] = node;
        layouts.Add(new(Layout.Scroll, Node: node));
    }

    /// <summary>Reads finalized controlled scrolling from a tracked viewport.</summary>
    /// <param name="id">Tracked identity, including modal scope.</param>
    /// <param name="offset">Resolved logical offset.</param>
    /// <param name="maximum">Current logical range.</param>
    public void ScrollMetrics(string id, out float offset, out float maximum)
    { CheckOpen(); NativeMethods.ScrollMetrics(context, trackedNodes[id], out offset, out maximum); }

    /// <summary>Begins or closes a controlled modal. Close visible contents with EndDialog.</summary>
    /// <param name="id">Stable modal identity.</param>
    /// <param name="open">Model-owned visibility.</param>
    /// <returns>Whether contents are open.</returns>
    public bool Modal(string id, bool open)
    {
        CheckFrame();
        bool visible = NativeMethods.ModalBegin(context, Stable(id), open ? 1 : 0) != 0;
        if (visible)
        {
            uint key = NativeMethods.ModalKey(context);
            if (modalKeys.TryGetValue(id, out uint previous) && previous != key) NativeMethods.Forget(context, previous);
            modalKeys[id] = key; usedModals.Add(id);
            layouts.Add(new(Layout.Dialog));
        }
        return visible;
    }

    /// <summary>Draws a native progress meter.</summary>
    /// <param name="percent">Percentage in 0..100.</param>
    public void Progress(int percent) { CheckFrame(); NativeMethods.Progress(context, percent / 100f, 0); }

    /// <summary>Copies drawing operations into native frame-owned memory.</summary>
    /// <param name="id">Stable canvas identity.</param>
    /// <param name="width">Logical width.</param>
    /// <param name="height">Logical height.</param>
    /// <param name="operations">Rectangle and line operations.</param>
    public unsafe void Canvas(string id, float width, float height, CuiPaint[] operations)
    {
        CheckFrame(); ArgumentNullException.ThrowIfNull(operations);
        if (!float.IsFinite(width) || width <= 0 || !float.IsFinite(height) || height <= 0)
            throw new ArgumentOutOfRangeException(nameof(width));
        foreach (var operation in operations)
            if (operation.Kind is < 0 or > 1 || !float.IsFinite(operation.A) || !float.IsFinite(operation.B) ||
                !float.IsFinite(operation.C) || !float.IsFinite(operation.D) || !float.IsFinite(operation.Thickness) ||
                operation.Kind == 0 && (operation.C < 0 || operation.D < 0) || operation.Kind == 1 && operation.Thickness <= 0)
                throw new ArgumentException("Invalid canvas operation.", nameof(operations));
        fixed (CuiPaint* items = operations) NativeMethods.Canvas(context, Stable(id), width, height, items, operations.Length);
    }

    /// <summary>Selects a native light or dark theme.</summary>
    /// <param name="dark">Whether to use dark colors.</param>
    public void SetDarkTheme(bool dark) { CheckOpen(); NativeMethods.SetTheme(context, dark ? 1 : 0); }

    /// <summary>Reads a native key press during a frame.</summary>
    /// <param name="key">Native key ordinal, tab=1, enter=2, escape=3, space=6, arrows=7..10.</param>
    /// <returns>Whether pressed.</returns>
    public bool KeyPressed(int key) { CheckFrame(); return NativeMethods.KeyPressed(context, key) != 0; }

    /// <summary>Gets native modifier bits during a frame.</summary>
    public int Modifiers { get { CheckFrame(); return NativeMethods.KeyModifiers(context); } }

    /// <summary>Injects a key for a bounded native integration test.</summary>
    /// <param name="key">Native key ordinal.</param>
    /// <param name="down">Pressed state.</param>
    /// <param name="modifiers">Native modifier bits.</param>
    public void SetKey(int key, bool down, int modifiers = 0)
    { CheckFrame(); NativeMethods.InputKey(context, key, down ? (byte)1 : (byte)0, modifiers); }

    /// <summary>Injects wheel movement for integration tests.</summary>
    /// <param name="delta">Vertical movement in lines.</param>
    public void Wheel(float delta) { CheckFrame(); NativeMethods.InputScroll(context, 0, delta); }
}
