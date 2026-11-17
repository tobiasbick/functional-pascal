using System.Runtime.InteropServices;
using System.Runtime.ExceptionServices;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

/// <summary>Owns a low-level CUI window on its creating UI thread and accepts wake requests from workers.</summary>
public sealed partial class CuiWindow : IDisposable
{
    [ThreadStatic]
    private static bool openGlWindow;

    private enum Layout
    {
        Column, Row, Scroll, Sheet, Group, ButtonGroup, Popover, HoverCard, Dialog, Card, ActionFooter,
        Accordion, ExpandableCard, SwipeItem, Tree, VirtualList, VirtualRow, Reorder, ReorderItem,
        Table, TableRow, DataGrid, TreeTable, Spotlight, Kanban, KanbanColumn, KanbanCard, SkeletonGroup,
        Carousel, CarouselSlide, CardStack, CardStackItem, Reasoning, Toolbar, NodeEditor, Veil
    }
    private readonly record struct LayoutScope(Layout Kind, int Node = -1, object? Data = null);

    private readonly int ownerThread = Environment.CurrentManagedThreadId;
    private readonly nint context;
    private readonly bool usesOpenGl;
    private readonly Lock wakeLock = new();
    private readonly List<LayoutScope> layouts = new();
    private nint app;
    private nint wakeTarget;
    private bool inFrame;
    private bool hasPresentedFrame;

    /// <summary>Creates a native window for a host adapter.</summary>
    /// <param name="title">UTF-8-compatible title, copied by the native application.</param>
    /// <param name="width">Positive logical width.</param>
    /// <param name="height">Positive logical height.</param>
    /// <param name="renderer">Requested rendering backend.</param>
    /// <param name="continuous">Whether to request frames while idle; intended for bounded tests.</param>
    /// <param name="placement">
    /// A saved placement to open at, whose size replaces <paramref name="width"/> and <paramref name="height"/>;
    /// null lets the system choose the place.
    /// </param>
    /// <param name="minimumWidth">Minimum logical width, zero for CUI's default.</param>
    /// <param name="minimumHeight">Minimum logical height, zero for CUI's default.</param>
    /// <param name="fixedSize">Whether the native window is nonresizable.</param>
    /// <param name="appId">Linux desktop identity; empty uses CUI's default.</param>
    public CuiWindow(string title, int width = 640, int height = 480,
        CuiRenderer renderer = CuiRenderer.Auto, bool continuous = false, CuiWindowPlacement? placement = null,
        int minimumWidth = 0, int minimumHeight = 0, bool fixedSize = false, string appId = "")
    {
        Utf8.Validate(title, nameof(title));
        Utf8.Validate(appId, nameof(appId));
        ArgumentOutOfRangeException.ThrowIfNegative(minimumWidth);
        ArgumentOutOfRangeException.ThrowIfNegative(minimumHeight);
        if (placement is { } saved)
        {
            if (!Enum.IsDefined(saved.State)) throw new ArgumentOutOfRangeException(nameof(placement));
            (width, height) = (saved.Width, saved.Height);
        }
        ArgumentOutOfRangeException.ThrowIfLessThan(width, 1);
        ArgumentOutOfRangeException.ThrowIfLessThan(height, 1);
        if (!Enum.IsDefined(renderer)) throw new ArgumentOutOfRangeException(nameof(renderer));
        // CUI leaves the window's OpenGL context current on the creating thread.
        if (openGlWindow)
        {
            if (renderer == CuiRenderer.OpenGl)
                throw new InvalidOperationException("A CUI OpenGL window already owns this thread's OpenGL context.");
            renderer = CuiRenderer.Software;
        }
        var at = placement.GetValueOrDefault();
        app = NativeMethods.Create(title, width, height, (int)renderer, continuous ? 1 : 0,
            placement.HasValue ? 1 : 0, at.X, at.Y, at.HasPosition ? 1 : 0,
            at.State == CuiWindowState.Maximized ? 1 : 0,
            minimumWidth, minimumHeight, fixedSize ? 1 : 0, Optional(appId));
        if (app == 0) throw new InvalidOperationException("CUI could not create a window. Check the native display connection.");
        wakeTarget = NativeMethods.OpenWakeTarget(app);
        if (wakeTarget == 0)
        {
            NativeMethods.Destroy(app);
            throw new InvalidOperationException("CUI could not open the window's platform wake target.");
        }
        context = NativeMethods.Context(app);
        usesOpenGl = NativeMethods.IsSoftware(app) == 0;
        if (usesOpenGl) openGlWindow = true;
    }

    /// <summary>Gets the actual native window backend.</summary>
    public string Backend { get { CheckOpen(); return Marshal.PtrToStringUTF8(NativeMethods.Backend(app)) ?? "unknown"; } }

    /// <summary>Reports the selected renderer, including fallback from OpenGL.</summary>
    public bool IsSoftware { get { CheckOpen(); return NativeMethods.IsSoftware(app) != 0; } }

    /// <summary>Gets the logical size most recently observed by the native event loop.</summary>
    public CuiSize Size { get { CheckOpen(); return NativeMethods.Size(app); } }

    /// <summary>Requests a new logical size; the native event loop applies the resize.</summary>
    /// <param name="width">Positive requested width.</param>
    /// <param name="height">Positive requested height.</param>
    public void Resize(int width, int height)
    {
        CheckOpen();
        ArgumentOutOfRangeException.ThrowIfLessThan(width, 1);
        ArgumentOutOfRangeException.ThrowIfLessThan(height, 1);
        NativeMethods.Resize(app, width, height);
    }

    /// <summary>Requests maximization on the current display.</summary>
    public void Maximize()
    {
        CheckOpen();
        NativeMethods.Maximize(app);
    }

    /// <summary>Presents the previous view and begins a frame, or returns false after close.</summary>
    /// <returns>Whether the host should construct another view.</returns>
    public bool NextFrame()
    {
        CheckOpen();
        if (layouts.Count != 0) throw new InvalidOperationException("Close all layouts before beginning another frame.");
        bool presented = inFrame;
        if (presented) editorRequests.Clear();
        if (presented) SnapshotShortcutKeys();
        if (presented) ResolveHostLayout();
        var retiredControls = presented ? controlKeys.Where(pair => !trackedNodes.ContainsKey(pair.Key)).ToArray() : null;
        var retiredModals = presented ? modalKeys.Where(pair => !usedModals.Contains(pair.Key)).ToArray() : null;
        int resizeVersion = resizeRenderVersion;
        int result;
        ExceptionDispatchInfo? callbackFailure;
        try { result = NativeMethods.Pump(app, DeferClose ? 1 : 0); }
        finally { callbackFailure = ReleaseFrameCallbacks(); }
        if (presented) RetireMissingControls(retiredControls!, retiredModals!, resizeVersion != resizeRenderVersion);
        CloseRequested |= result == 2;
        inFrame = result != 0;
        shortcutsRead = false;
        if (inFrame) shortcutOwners = NativeMethods.ShortcutOwners(context);
        hasPresentedFrame |= presented;
        if (inFrame) { trackedNodes.Clear(); scrollRequests.Clear(); scrollNodes.Clear(); usedModals.Clear(); moneyDrawn.Clear(); }
        else { editorRequests.Clear(); moneyFields.Clear(); moneyDrawn.Clear(); CloseWakeTarget(); }
        if (resizeFailure is { } failure)
        {
            resizeFailure = null;
            SetResizeRenderer(null);
            failure.Throw();
        }
        callbackFailure?.Throw();
        return inFrame;
    }

    /// <summary>Posts a platform event so the UI thread can resume an idle frame loop.</summary>
    /// <returns>Whether the event was posted. Returns false after close or disposal.</returns>
    public bool TryWake()
    {
        lock (wakeLock)
            return wakeTarget != 0 && NativeMethods.PostWake(wakeTarget) != 0;
    }

    /// <summary>Presents the current view without waiting for native input and closes its frame.</summary>
    public void Present()
    {
        CheckFrame();
        if (layouts.Count != 0) throw new InvalidOperationException("Close all layouts before presenting.");
        ResolveHostLayout();
        int presented;
        ExceptionDispatchInfo? callbackFailure;
        try { presented = NativeMethods.Present(app); }
        finally { SnapshotShortcutKeys(); inFrame = false; editorRequests.Clear(); callbackFailure = ReleaseFrameCallbacks(); }
        if (presented == 0) throw new InvalidOperationException("CUI could not present the current view.");
        hasPresentedFrame = true;
        RetireMissingControls(
            controlKeys.Where(pair => !trackedNodes.ContainsKey(pair.Key)).ToArray(),
            modalKeys.Where(pair => !usedModals.Contains(pair.Key)).ToArray(), false);
        callbackFailure?.Throw();
    }

    private void RetireMissingControls(KeyValuePair<string, uint>[] controls, KeyValuePair<string, uint>[] modals,
        bool renderedDuringPump)
    {
        foreach (var (id, key) in controls)
            if ((!renderedDuringPump || !trackedNodes.ContainsKey(id)) && controlKeys.TryGetValue(id, out uint current) && current == key)
                ForgetControl(id);
        foreach (var (id, key) in modals)
            if ((!renderedDuringPump || !usedModals.Contains(id)) && modalKeys.TryGetValue(id, out uint current) && current == key)
            { NativeMethods.Forget(context, key); modalKeys.Remove(id); }
        RetireMoneyFields();
    }

    /// <summary>Enables or disables CUI motion reduction.</summary>
    /// <param name="enabled">Whether animations should snap to their targets.</param>
    public void SetReducedMotion(bool enabled)
    {
        CheckOpen();
        NativeMethods.ReducedMotion(context, enabled ? (byte)1 : (byte)0);
    }

    /// <summary>Opens a padded full-width column in the current frame.</summary>
    /// <param name="padding">Nonnegative logical padding.</param>
    /// <param name="gap">Nonnegative logical child spacing.</param>
    public void BeginColumn(float padding = 24, float gap = 12)
    {
        CheckLayoutOptions(padding, gap);
        NativeMethods.BeginColumn(context, padding, gap);
        layouts.Add(new(Layout.Column));
    }

    /// <summary>Closes the innermost layout when it is a column.</summary>
    public void EndColumn() => EndLayout(Layout.Column);

    /// <summary>Opens a padded full-width horizontal row in the current frame.</summary>
    /// <param name="padding">Nonnegative logical padding.</param>
    /// <param name="gap">Nonnegative logical spacing between children.</param>
    public void BeginRow(float padding = 24, float gap = 12)
    {
        CheckLayoutOptions(padding, gap);
        NativeMethods.BeginRow(context, padding, gap);
        layouts.Add(new(Layout.Row));
    }

    /// <summary>Closes the innermost layout when it is a row.</summary>
    public void EndRow() => EndLayout(Layout.Row);

    /// <summary>Opens a full-width scroll container in the current frame.</summary>
    /// <param name="id">Nonempty stable identity unique in the current CUI ID scope.</param>
    /// <param name="height">Positive height in logical units, or zero to fill the available height.</param>
    /// <param name="horizontal">Whether overflowing content may also scroll horizontally.</param>
    public void BeginScroll(string id, float height = 0, bool horizontal = false)
    {
        CheckFrame();
        Utf8.Validate(id, nameof(id));
        ArgumentException.ThrowIfNullOrEmpty(id);
        if (!float.IsFinite(height) || height < 0) throw new ArgumentOutOfRangeException(nameof(height));
        NativeMethods.BeginScroll(context, Stable(id), height, horizontal ? 1 : 0);
        layouts.Add(new(Layout.Scroll));
    }

    /// <summary>Closes the innermost layout when it is a scroll container.</summary>
    public void EndScroll()
    {
        var scope = RequireLayout(Layout.Scroll);
        if (scope.Node >= 0) NativeMethods.GroupEnd(context, scope.Node);
        else NativeMethods.EndScroll(context);
        layouts.RemoveAt(layouts.Count - 1);
    }

    /// <summary>Requests a vertical scroll offset before drawing a previously laid out container.</summary>
    /// <param name="id">The identity passed to <see cref="BeginScroll"/>, in the same CUI ID scope.</param>
    /// <param name="offset">Nonnegative finite offset in logical units from the top. CUI clamps it to the content.</param>
    public void ScrollTo(string id, float offset)
    {
        CheckFrame();
        Utf8.Validate(id, nameof(id));
        ArgumentException.ThrowIfNullOrEmpty(id);
        if (!float.IsFinite(offset) || offset < 0) throw new ArgumentOutOfRangeException(nameof(offset));
        NativeMethods.ScrollTo(context, Stable(id), offset);
    }

    /// <summary>Requests focus for a control when CUI next draws it as focusable.</summary>
    /// <param name="id">Nonempty stable control identity.</param>
    public void RequestFocus(string id)
    {
        CheckFrame();
        Utf8.Validate(id, nameof(id));
        ArgumentException.ThrowIfNullOrEmpty(id);
        NativeMethods.RequestFocus(context, id);
    }

    /// <summary>Adds a heading to the current frame.</summary>
    /// <param name="text">Valid Unicode heading text.</param>
    public void Heading(string text) { CheckFrame(); Utf8.Validate(text, nameof(text)); NativeMethods.Heading(context, text, 2); }

    /// <summary>Adds text to the current frame.</summary>
    /// <param name="text">Valid Unicode display text.</param>
    public void Text(string text) { CheckFrame(); Utf8.Validate(text, nameof(text)); NativeMethods.Text(context, text, 0); }

    /// <summary>Gets the most recent widget's rectangle from the latest native layout.</summary>
    public CuiRectangle LastRectangle { get { CheckFrame(); return NativeMethods.LastRectangle(context); } }

    /// <summary>Renders the current completed view into a PNG after a frame has been presented.</summary>
    /// <param name="path">Unicode output filename in an existing directory.</param>
    public void CapturePng(string path)
    {
        CheckOpen(); Utf8.Validate(path, nameof(path));
        ArgumentException.ThrowIfNullOrEmpty(path);
        if (layouts.Count != 0 || !hasPresentedFrame)
            throw new InvalidOperationException("Capture needs closed layouts and a previously presented frame.");
        int captured;
        try { captured = NativeMethods.Capture(app, path); }
        finally { inFrame = false; }
        if (captured == 0) throw new IOException("CUI could not capture the current view.");
    }

    /// <summary>Destroys all window/context resources on the creating UI thread.</summary>
    public void Dispose()
    {
        if (app == 0) return;
        CheckOpen();
        CloseWakeTarget();
        SetResizeRenderer(null);
        trackedNodes.Clear();
        controlKeys.Clear();
        modalKeys.Clear();
        editorRequests.Clear();
        NativeMethods.FinishFrameForDestroy(app);
        moneyFields.Clear(); moneyDrawn.Clear();
        ReleaseOwnedStrings();
        NativeMethods.Destroy(app);
        ReleaseFrameCallbacks();
        if (usesOpenGl) openGlWindow = false;
        app = 0;
        inFrame = false;
    }

    private void CloseWakeTarget()
    {
        lock (wakeLock)
        {
            if (wakeTarget == 0) return;
            NativeMethods.CloseWakeTarget(wakeTarget);
            wakeTarget = 0;
        }
    }

    private void CheckOpen()
    {
        ObjectDisposedException.ThrowIf(app == 0, this);
        if (ownerThread != Environment.CurrentManagedThreadId)
            throw new InvalidOperationException("CUI must be accessed and disposed on its creating UI thread.");
        if (formattingAxisLabel)
            throw new InvalidOperationException("Axis label callbacks cannot reenter the native window.");
        if (paintingCanvas)
            throw new InvalidOperationException("Canvas paint callbacks cannot reenter the native window.");
    }

    private void CheckFrame()
    {
        CheckOpen();
        if (!inFrame) throw new InvalidOperationException("Begin a native frame before constructing a view or injecting input.");
    }

    private void CheckLayoutOptions(float padding, float gap)
    {
        CheckFrame();
        if (!float.IsFinite(padding) || padding < 0) throw new ArgumentOutOfRangeException(nameof(padding));
        if (!float.IsFinite(gap) || gap < 0) throw new ArgumentOutOfRangeException(nameof(gap));
    }

    private void EndLayout(Layout expected)
    {
        var scope = RequireLayout(expected);
        if (scope.Node >= 0) NativeMethods.GroupEnd(context, scope.Node);
        else NativeMethods.EndBox(context);
        layouts.RemoveAt(layouts.Count - 1);
    }

    private LayoutScope RequireLayout(Layout expected)
    {
        CheckFrame();
        if (layouts.Count == 0 || layouts[^1].Kind != expected)
            throw new InvalidOperationException($"The innermost layout must be {expected}.");
        return layouts[^1];
    }
}
