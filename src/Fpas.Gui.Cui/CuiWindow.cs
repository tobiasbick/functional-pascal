using System.Runtime.InteropServices;
using Fpas.Gui.Cui.Interop;
using Fpas.Gui.Cui.Text;

namespace Fpas.Gui.Cui;

/// <summary>Owns a low-level CUI window. Access and dispose it on its creating UI thread.</summary>
public sealed class CuiWindow : IDisposable
{
    private readonly int ownerThread = Environment.CurrentManagedThreadId;
    private readonly nint context;
    private nint app;
    private bool inFrame;
    private bool hasPresentedFrame;
    private int columns;

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
        if (columns != 0) throw new InvalidOperationException("Close layout columns before beginning another frame.");
        bool presented = inFrame;
        inFrame = NativeMethods.Frame(app) != 0;
        hasPresentedFrame |= presented;
        return inFrame;
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
        CheckFrame();
        if (!float.IsFinite(padding) || padding < 0) throw new ArgumentOutOfRangeException(nameof(padding));
        if (!float.IsFinite(gap) || gap < 0) throw new ArgumentOutOfRangeException(nameof(gap));
        NativeMethods.BeginColumn(context, padding, gap);
        columns++;
    }

    /// <summary>Closes the current column.</summary>
    public void EndColumn()
    {
        CheckFrame();
        if (columns == 0) throw new InvalidOperationException("No layout column is open.");
        NativeMethods.EndColumn(context);
        columns--;
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
            return (CuiTextInputEvents)NativeMethods.Input(context, id, buffer, (nuint)value.Capacity, 0);
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
        if (columns != 0 || !hasPresentedFrame)
            throw new InvalidOperationException("Capture needs closed columns and a previously presented frame.");
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
        NativeMethods.Destroy(app);
        app = 0;
        inFrame = false;
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
}
