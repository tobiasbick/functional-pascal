using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Whether the last widget produced a layout node in the current frame.</summary>
    public bool HasLastWidget { get { CheckFrame(); return NativeMethods.LastNode(context) >= 0; } }

    /// <summary>Draws a native hairline separator in the current container.</summary>
    public void Separator() { CheckFrame(); NativeMethods.Separator(context); }
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

    private static string Stable(string id)
    {
        Utf8.Validate(id, nameof(id));
        ArgumentException.ThrowIfNullOrEmpty(id);
        return "###" + Convert.ToBase64String(Utf8.Encoding.GetBytes(id));
    }

    /// <summary>Opens a clipped host layout container.</summary>
    /// <param name="horizontal">Whether its main axis is horizontal.</param>
    /// <param name="padding">Logical inset.</param>
    /// <param name="gap">Logical separation.</param>
    /// <param name="alignment">Default=0, start=1, center=2, end=3, stretch=4.</param>
    /// <param name="framed">Whether to draw a border.</param>
    public void BeginGroup(bool horizontal, float padding, float gap, int alignment, bool framed)
        => BeginGroup(horizontal, padding, gap, alignment, framed, false);

    /// <summary>Opens a native host layout container with optional row wrapping.</summary>
    /// <param name="horizontal">Whether its main axis is horizontal.</param>
    /// <param name="padding">Logical inset.</param>
    /// <param name="gap">Logical separation.</param>
    /// <param name="alignment">Default=0, start=1, center=2, end=3, stretch=4.</param>
    /// <param name="framed">Whether to draw a border.</param>
    /// <param name="wrap">Whether a horizontal group continues on another line.</param>
    public void BeginGroup(bool horizontal, float padding, float gap, int alignment, bool framed, bool wrap)
        => BeginGroup(horizontal, padding, gap, alignment, framed, wrap, !wrap);

    /// <summary>Opens a native host layout container with explicit clipping.</summary>
    /// <param name="horizontal">Whether its main axis is horizontal.</param>
    /// <param name="padding">Logical inset.</param>
    /// <param name="gap">Logical separation.</param>
    /// <param name="alignment">Default=0, start=1, center=2, end=3, stretch=4.</param>
    /// <param name="framed">Whether to draw a border.</param>
    /// <param name="wrap">Whether the row wraps across lines.</param>
    /// <param name="clip">Whether to clip children at the container bounds.</param>
    public void BeginGroup(bool horizontal, float padding, float gap, int alignment, bool framed, bool wrap, bool clip)
    {
        CheckLayoutOptions(padding, gap);
        ArgumentOutOfRangeException.ThrowIfNegative(alignment);
        ArgumentOutOfRangeException.ThrowIfGreaterThan(alignment, 4);
        if (wrap && !horizontal) throw new ArgumentException("Wrapping requires a horizontal group.", nameof(wrap));
        int node = NativeMethods.GroupBegin(context, horizontal ? 1 : 0, padding, gap, alignment,
            framed ? 1 : 0, wrap ? 1 : 0, clip ? 1 : 0);
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
    /// <param name="kind">Content=0, fill=1, fixed=2, percentage=3.</param>
    /// <param name="basis">Logical basis or percentage of the parent's inner extent.</param>
    /// <param name="minimum">Minimum extent.</param>
    /// <param name="maximum">Maximum extent or -1.</param>
    /// <param name="grow">Growth weight.</param>
    /// <param name="alignment">Cross-axis placement.</param>
    public void SizeLast(bool horizontal, int kind, float basis, float minimum, float maximum, float grow, int alignment)
    {
        CheckFrame();
        CheckExtent(kind, basis, nameof(basis));
        if (!float.IsFinite(minimum) || minimum < 0 ||
            !float.IsFinite(maximum) || maximum < -1 || maximum >= 0 && maximum < minimum ||
            !float.IsFinite(grow) || grow < 0 || (kind is 2 or 3) && grow != 0 || alignment is < 0 or > 4)
            throw new ArgumentOutOfRangeException(nameof(basis), "Invalid shared layout constraints.");
        int node = NativeMethods.LastNode(context);
        NativeMethods.NodeSize(context, node, horizontal ? 1 : 0, kind, basis, minimum, maximum, grow, alignment);
    }

    private static void CheckExtent(int kind, float value, string name)
    {
        if (kind is < 0 or > 3) throw new ArgumentOutOfRangeException(nameof(kind));
        if (!float.IsFinite(value) || value < 0 || kind == 3 && value > 100)
            throw new ArgumentOutOfRangeException(name);
    }

    /// <summary>Applies a semantic theme role to the last widget.</summary>
    /// <param name="role">Shared role ordinal.</param>
    public void StyleLast(int role) { CheckFrame(); NativeMethods.NodeRole(context, NativeMethods.LastNode(context), role); }

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
        if (mode is < -1 or > 1) throw new ArgumentOutOfRangeException(nameof(mode));
        if (!float.IsFinite(offset) || offset < 0) throw new ArgumentOutOfRangeException(nameof(offset));
        if (mode >= 0) CheckFrame();
        NativeMethods.ScrollState(context, Stable(id), mode, offset, out current, out maximum);
        if (mode >= 0) scrollRequests[id] = (mode, offset);
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

    private void TrackModal(string id, uint? ownedKey = null)
    {
        uint key = ownedKey ?? NativeMethods.ModalKey(context);
        if (modalKeys.TryGetValue(id, out uint previous) && previous != key) NativeMethods.Forget(context, previous);
        modalKeys[id] = key;
        usedModals.Add(id);
    }

    /// <summary>Applies the shared theme's semantic palette atomically.</summary>
    /// <param name="dark">Whether to use dark typography and elevation defaults.</param>
    /// <param name="accent">CUI accent family: neutral=0 through rose=7.</param>
    /// <param name="colors">Exactly 34 packed RGBA colors in CUI semantic token order.</param>
    public unsafe void SetTheme(bool dark, int accent, uint[] colors)
    {
        CheckOpen(); ArgumentNullException.ThrowIfNull(colors);
        ArgumentOutOfRangeException.ThrowIfNegative(accent);
        ArgumentOutOfRangeException.ThrowIfGreaterThan(accent, 7);
        if (colors.Length != 34) throw new ArgumentException("Expected 34 CUI semantic colors.", nameof(colors));
        fixed (uint* values = colors) NativeMethods.SetTheme(context, dark ? 1 : 0, accent, values);
    }

    /// <summary>Copies the active palette in the same semantic order accepted by SetTheme.</summary>
    /// <returns>34 packed RGBA colors detached from native storage.</returns>
    public unsafe uint[] ReadTheme()
    {
        CheckOpen();
        var colors = new uint[34];
        fixed (uint* values = colors) NativeMethods.ReadTheme(context, values);
        return colors;
    }

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

    /// <summary>Supplies a committed text event to the native input queue.</summary>
    /// <param name="text">Text to deliver to the focused editor in the current frame.</param>
    public void InputText(string text)
    {
        CheckFrame();
        Utf8.Validate(text, nameof(text));
        NativeMethods.InputText(context, text);
    }

    /// <summary>Injects wheel movement for integration tests.</summary>
    /// <param name="delta">Vertical movement in lines.</param>
    public void Wheel(float delta) => Wheel(0, delta);

    /// <summary>Supplies horizontal and vertical wheel input to CUI.</summary>
    /// <param name="horizontal">Horizontal movement in native wheel lines.</param>
    /// <param name="vertical">Vertical movement in native wheel lines.</param>
    public void Wheel(float horizontal, float vertical)
    {
        CheckFrame();
        NativeMethods.InputScroll(context, horizontal, vertical);
    }
}
