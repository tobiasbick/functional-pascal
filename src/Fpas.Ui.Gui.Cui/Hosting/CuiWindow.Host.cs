// [Documentation](../../../docs/ui/gui/cui.md#layout-and-control-inspection)
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
    private readonly HashSet<int> scrollMetricNodes = [];
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

    // Maps an empty optional native string to NULL.
    private static string? Optional(string value) => value.Length == 0 ? null : value;

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
        => BeginGroup(horizontal, padding, gap, alignment, framed, wrap, clip, 0);

    /// <summary>Opens a native row or column with main-axis distribution.</summary>
    /// <param name="horizontal">Whether its main axis is horizontal.</param>
    /// <param name="padding">Logical inset.</param>
    /// <param name="gap">Logical separation.</param>
    /// <param name="alignment">Cross-axis alignment in native order.</param>
    /// <param name="framed">Whether to draw a border.</param>
    /// <param name="wrap">Whether a horizontal group wraps.</param>
    /// <param name="clip">Whether to clip children at the container bounds.</param>
    /// <param name="justify">Start, center, end, between, or around in native order.</param>
    /// <param name="id">Optional stable ID, otherwise CUI assigns anonymous scroll identity.</param>
    /// <param name="scroll">None, vertical or both axes, in that order.</param>
    public void BeginGroup(bool horizontal, float padding, float gap, int alignment, bool framed,
        bool wrap, bool clip, int justify, string? id = null, int scroll = 0)
    {
        CheckLayoutOptions(padding, gap);
        ArgumentOutOfRangeException.ThrowIfNegative(alignment);
        ArgumentOutOfRangeException.ThrowIfGreaterThan(alignment, 4);
        if (justify is < 0 or > 4) throw new ArgumentOutOfRangeException(nameof(justify));
        if (wrap && !horizontal) throw new ArgumentException("Wrapping requires a horizontal group.", nameof(wrap));
        OpenGroup(horizontal ? 1 : 0, padding, gap, alignment, framed, wrap, clip, justify, id, scroll);
    }

    private void OpenGroup(int direction, float padding, float gap, int alignment, bool framed,
        bool wrap, bool clip, int justify, string? id, int scroll)
    {
        if (scroll is < 0 or > 2) throw new ArgumentOutOfRangeException(nameof(scroll));
        int node = NativeMethods.GroupBegin(context, direction, padding, gap, alignment,
            framed ? 1 : 0, wrap ? 1 : 0, clip ? 1 : 0, justify, id is null ? null : Stable(id), scroll);
        RegisterBox(Layout.Group, node, scrolling: scroll != 0);
    }

    /// <summary>Closes the innermost host group and makes it the last widget.</summary>
    public void EndGroup() => EndLayout(Layout.Group);

    /// <summary>Applies shared sizing to the last widget.</summary>
    /// <param name="horizontal">Parent main axis.</param>
    /// <param name="kind">Content=0, fill=1, fixed=2, percentage=3.</param>
    /// <param name="basis">Logical basis or percentage of the parent's inner extent.</param>
    /// <param name="minimum">Minimum extent.</param>
    /// <param name="maximum">Maximum extent at least as large as the minimum, or -1 for no maximum.</param>
    /// <param name="grow">Growth weight.</param>
    /// <param name="alignment">Cross-axis placement.</param>
    public void SizeLast(bool horizontal, int kind, float basis, float minimum, float maximum, float grow, int alignment)
    {
        CheckFrame();
        CheckExtent(kind, basis, nameof(basis));
        if (!float.IsFinite(minimum) || minimum < 0 ||
            !float.IsFinite(grow) || grow < 0 || (kind is 2 or 3) && grow != 0 || alignment is < 0 or > 4)
            throw new ArgumentOutOfRangeException(nameof(basis), "Invalid shared layout constraints.");
        if (!float.IsFinite(maximum) || maximum != -1 && maximum < minimum)
            throw new ArgumentOutOfRangeException(nameof(maximum), "The maximum must be at least the minimum or -1.");
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
    public bool ControlFocused(string id)
    {
        CheckOpen();
        int node = trackedNodes[id];
        return controlKeys.TryGetValue(id, out uint key)
            ? NativeMethods.FocusKey(context, key) != 0
            : NativeMethods.Focused(context, node) != 0;
    }

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

    /// <summary>Reads the grid's scroll target or requests an offset before building its rows.</summary>
    /// <param name="id">Stable grid identity.</param>
    /// <param name="offset">A requested logical offset, or null to read only.</param>
    /// <returns>The target before applying the request.</returns>
    public float DataGridScroll(string id, float? offset = null)
    {
        CheckFrame();
        if (offset is float value && (!float.IsFinite(value) || value < 0))
            throw new ArgumentOutOfRangeException(nameof(offset));
        NativeMethods.ScrollState(context, Stable(id), offset.HasValue ? 2 : -2, offset ?? 0, out var current, out _);
        return current;
    }

    /// <summary>Opens a host-controlled scroll view.</summary>
    /// <param name="id">Stable identity.</param>
    /// <param name="showScrollbar">Whether to show its scrollbar.</param>
    /// <param name="enabled">Whether user input is accepted.</param>
    public void BeginViewport(string id, bool showScrollbar, bool enabled = true) =>
        BeginViewport(id, showScrollbar, enabled, false);

    /// <summary>Opens a scroll view with optional native edge fades.</summary>
    /// <param name="id">Stable identity.</param>
    /// <param name="showScrollbar">Whether to show its scrollbar.</param>
    /// <param name="enabled">Whether user input is accepted.</param>
    /// <param name="fadeEdges">Whether to fade edges with hidden content.</param>
    /// <param name="horizontal">Also allow native horizontal scrolling; vertical requests remain vertical.</param>
    /// <param name="widthKind">Native content, fill, fixed or percentage width, zero through three.</param>
    /// <param name="width">Fixed width in logical points or a percentage.</param>
    /// <param name="heightKind">Native extent encoding. Content and fill both use available height.</param>
    /// <param name="height">Fixed height in logical points or a percentage.</param>
    /// <param name="padding">Inset in logical points on every edge of the viewport.</param>
    /// <param name="gap">Separation between direct children in logical points.</param>
    /// <param name="fadeColor">Packed straight-alpha RGBA. Zero uses the nearest opaque backdrop.</param>
    public void BeginViewport(string id, bool showScrollbar, bool enabled, bool fadeEdges, bool horizontal = false,
        int widthKind = 1, float width = 0, int heightKind = 1, float height = 0, float padding = 0, float gap = 0,
        uint fadeColor = 0)
    {
        CheckFrame();
        CheckExtent(widthKind, width, nameof(width));
        CheckExtent(heightKind, height, nameof(height));
        ValidateDistance(padding, nameof(padding));
        ValidateDistance(gap, nameof(gap));
        int node = NativeMethods.Viewport(context, Stable(id), showScrollbar ? 1 : 0, enabled ? 1 : 0,
            fadeEdges ? 1 : 0, horizontal ? 1 : 0, widthKind, width, heightKind, height, padding, gap, fadeColor);
        RegisterBox(Layout.Scroll, node, scrolling: true);
        scrollNodes[id] = node;
    }

    /// <summary>Reads finalized vertical scrolling from a tracked viewport or scrolling box.</summary>
    /// <param name="id">Tracked identity, including modal scope.</param>
    /// <param name="offset">Resolved logical offset.</param>
    /// <param name="maximum">Current logical range.</param>
    public void ScrollMetrics(string id, out float offset, out float maximum)
    {
        CheckOpen();
        int node = trackedNodes[id];
        if (!scrollMetricNodes.Contains(node))
            throw new ArgumentException("The tracked identity must belong to a scroll container.", nameof(id));
        NativeMethods.ScrollMetrics(context, node, out offset, out maximum);
    }

    private void TrackModal(string id, uint? ownedKey = null)
    {
        uint key = ownedKey ?? NativeMethods.ModalKey(context);
        if (modalKeys.TryGetValue(id, out uint previous) && previous != key) NativeMethods.Forget(context, previous);
        modalKeys[id] = key;
        usedModals.Add(id);
    }
}
