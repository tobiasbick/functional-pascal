using System.Runtime.InteropServices;
using Fpas.Gui.Cui.Interop;
using Fpas.Gui.Cui.Text;

namespace Fpas.Gui.Cui;

/// <summary>Owns a low-level CUI window on its creating UI thread and accepts wake requests from workers.</summary>
public sealed class CuiWindow : IDisposable
{
    private enum Layout { Column, Row, Table }
    private readonly record struct LayoutScope(Layout Kind, int ColumnCount = 0, int CellsLeft = 0);

    private readonly int ownerThread = Environment.CurrentManagedThreadId;
    private readonly nint context;
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
    public CuiWindow(string title, int width = 640, int height = 480,
        CuiRenderer renderer = CuiRenderer.Auto, bool continuous = false)
    {
        Utf8.Validate(title, nameof(title));
        ArgumentOutOfRangeException.ThrowIfLessThan(width, 1);
        ArgumentOutOfRangeException.ThrowIfLessThan(height, 1);
        if (!Enum.IsDefined(renderer)) throw new ArgumentOutOfRangeException(nameof(renderer));
        app = NativeMethods.Create(title, width, height, (int)renderer, continuous ? 1 : 0);
        if (app == 0) throw new InvalidOperationException("CUI could not create a window. Check the native display connection.");
        wakeTarget = NativeMethods.OpenWakeTarget(app);
        if (wakeTarget == 0)
        {
            NativeMethods.Destroy(app);
            throw new InvalidOperationException("CUI could not open the window's platform wake target.");
        }
        context = NativeMethods.Context(app);
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

    /// <summary>Presents the previous view and begins a frame, or returns false after close.</summary>
    /// <returns>Whether the host should construct another view.</returns>
    public bool NextFrame()
    {
        CheckOpen();
        if (layouts.Count != 0) throw new InvalidOperationException("Close all layouts before beginning another frame.");
        bool presented = inFrame;
        inFrame = NativeMethods.Frame(app) != 0;
        hasPresentedFrame |= presented;
        if (!inFrame) CloseWakeTarget();
        return inFrame;
    }

    /// <summary>Posts a platform event so the UI thread can resume an idle frame loop.</summary>
    /// <returns>Whether the event was posted. Returns false after close or disposal.</returns>
    public bool TryWake()
    {
        lock (wakeLock)
            return wakeTarget != 0 && NativeMethods.PostWake(wakeTarget) != 0;
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

    /// <summary>Requests focus for a control when CUI next draws it as focusable.</summary>
    /// <param name="id">Nonempty stable control identity, such as the ID passed to <see cref="TextInput"/>.</param>
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

    /// <summary>Adds a button and reports activation.</summary>
    /// <param name="label">CUI label including its stable identity suffix.</param>
    /// <returns>Whether the button was activated.</returns>
    public bool Button(string label)
    {
        CheckFrame(); Utf8.Validate(label, nameof(label));
        return NativeMethods.Button(context, label, 0) != 0;
    }

    /// <summary>Draws a list item with a host-owned selection state and reports activation.</summary>
    /// <param name="label">Visible UTF-8 label. Use <c>###id</c> to give the item a stable identity independent of its label.</param>
    /// <param name="selected">Whether the host wants this item drawn as selected.</param>
    /// <returns>Whether the item was activated in the current frame.</returns>
    public bool ListItem(string label, bool selected)
    {
        CheckFrame(); Utf8.Validate(label, nameof(label));
        return NativeMethods.ListItem(context, label, selected ? 1 : 0) != 0;
    }

    /// <summary>Draws a tab item, typically in a row, with a host-supplied selection state.</summary>
    /// <param name="label">Visible UTF-8 label. Use <c>###id</c> for a stable identity independent of the label.</param>
    /// <param name="selected">Whether the host wants this tab drawn as selected.</param>
    /// <returns>Whether the tab was activated in the current frame.</returns>
    public bool TabItem(string label, bool selected)
    {
        CheckFrame(); Utf8.Validate(label, nameof(label));
        return NativeMethods.TabItem(context, label, selected ? 1 : 0) != 0;
    }

    /// <summary>Opens a selectable table in the current frame.</summary>
    /// <param name="id">Nonempty stable table identity.</param>
    /// <param name="columnTitles">One or more UTF-8 column titles, copied during the call.</param>
    public void BeginTable(string id, string[] columnTitles)
    {
        CheckFrame();
        Utf8.Validate(id, nameof(id));
        ArgumentException.ThrowIfNullOrEmpty(id);
        ArgumentNullException.ThrowIfNull(columnTitles);
        if (columnTitles.Length == 0) throw new ArgumentException("A table needs at least one column.", nameof(columnTitles));
        foreach (string title in columnTitles) Utf8.Validate(title, nameof(columnTitles));
        if (NativeMethods.BeginTable(context, id, columnTitles, columnTitles.Length) == 0)
            throw new InvalidOperationException("CUI could not open the table.");
        layouts.Add(new(Layout.Table, columnTitles.Length));
    }

    /// <summary>Begins the next row, closing the preceding row if present.</summary>
    /// <param name="selected">Whether the host wants this row drawn as selected.</param>
    /// <returns>Whether the row was activated in the current frame.</returns>
    public bool TableRow(bool selected)
    {
        LayoutScope table = RequireLayout(Layout.Table);
        bool activated = NativeMethods.TableRow(context, selected ? 1 : 0) != 0;
        layouts[^1] = table with { CellsLeft = table.ColumnCount };
        return activated;
    }

    /// <summary>Adds text to the next cell of the current table row.</summary>
    /// <param name="text">Valid Unicode cell text.</param>
    public void TableCell(string text)
    {
        LayoutScope table = RequireLayout(Layout.Table);
        if (table.CellsLeft == 0) throw new InvalidOperationException("Open a table row with an available cell before adding text.");
        Utf8.Validate(text, nameof(text));
        NativeMethods.TableCell(context, text);
        layouts[^1] = table with { CellsLeft = table.CellsLeft - 1 };
    }

    /// <summary>Closes the innermost table and its final row. CUI fills any omitted cells with empty cells.</summary>
    public void EndTable()
    {
        RequireLayout(Layout.Table);
        NativeMethods.EndTable(context);
        layouts.RemoveAt(layouts.Count - 1);
    }

    /// <summary>Adds a checkbox and updates its controlled value.</summary>
    /// <param name="label">CUI label including its stable identity suffix.</param>
    /// <param name="value">Current value, replaced by an edited value.</param>
    /// <returns>Whether the value changed.</returns>
    public bool Checkbox(string label, ref bool value)
    {
        CheckFrame(); Utf8.Validate(label, nameof(label));
        byte nativeValue = value ? (byte)1 : (byte)0;
        bool changed = NativeMethods.Checkbox(context, label, ref nativeValue, 0) != 0;
        value = nativeValue != 0;
        return changed;
    }

    /// <summary>Adds a text editor backed by an owned UTF-8 buffer.</summary>
    /// <param name="id">Nonempty stable control identity.</param>
    /// <param name="value">Buffer owned across frames by the host.</param>
    /// <returns>Native change, submission and focus flags.</returns>
    public unsafe CuiTextInputEvents TextInput(string id, CuiTextBuffer value)
    {
        CheckFrame(); Utf8.Validate(id, nameof(id));
        ArgumentException.ThrowIfNullOrEmpty(id);
        ArgumentNullException.ThrowIfNull(value);
        fixed (byte* buffer = value.Bytes)
            return (CuiTextInputEvents)NativeMethods.Input(context, id, buffer, (nuint)value.Capacity);
    }

    /// <summary>Gets the most recent widget's rectangle from the latest native layout.</summary>
    public CuiRectangle LastRectangle { get { CheckFrame(); return NativeMethods.LastRectangle(context); } }

    /// <summary>Injects a logical pointer position through CUI's input API.</summary>
    /// <param name="x">Finite horizontal coordinate.</param>
    /// <param name="y">Finite vertical coordinate.</param>
    public void MovePointer(float x, float y)
    {
        CheckFrame();
        if (!float.IsFinite(x)) throw new ArgumentOutOfRangeException(nameof(x));
        if (!float.IsFinite(y)) throw new ArgumentOutOfRangeException(nameof(y));
        NativeMethods.MovePointer(context, x, y);
    }

    /// <summary>Injects the left pointer button through CUI's input API.</summary>
    /// <param name="down">Whether the button is pressed.</param>
    public void SetLeftButton(bool down) { CheckFrame(); NativeMethods.LeftButton(context, 0, down ? (byte)1 : (byte)0); }

    /// <summary>Injects text into the focused native editor.</summary>
    /// <param name="text">Valid Unicode input without zero characters.</param>
    public void TypeText(string text) { CheckFrame(); Utf8.Validate(text, nameof(text)); NativeMethods.TypeText(context, text); }

    /// <summary>Renders the current completed view into a PNG after a frame has been presented.</summary>
    /// <param name="path">Output filename; the containing directory must exist.</param>
    public void CapturePng(string path)
    {
        CheckFrame(); Utf8.Validate(path, nameof(path));
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
        NativeMethods.Destroy(app);
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
        RequireLayout(expected);
        NativeMethods.EndBox(context);
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
