using Fpas.Gui.Cui.Imaging;
using Fpas.Gui.Cui.Text;

namespace Fpas.Gui.Cui;

/// <summary>Owns the native-facing editor buffers and images retained by a declarative host.</summary>
public sealed class CuiHostResources : IDisposable
{
    private sealed record EditorEntry(CuiTextBuffer Buffer, bool Multiline);
    private sealed record ImageEntry(CuiImage Image, byte[] Pixels, int Width, int Height);
    private readonly CuiWindow window;
    private readonly int owner = Environment.CurrentManagedThreadId;
    private readonly Dictionary<string, EditorEntry> editors = new(StringComparer.Ordinal);
    private readonly Dictionary<string, ImageEntry> images = new(StringComparer.Ordinal);
    private readonly HashSet<string> usedEditors = new(StringComparer.Ordinal);
    private readonly HashSet<string> usedImages = new(StringComparer.Ordinal);
    private bool disposed;

    /// <summary>Creates a resource owner associated with one live window.</summary>
    /// <param name="window">The owning UI-thread window.</param>
    public CuiHostResources(CuiWindow window) => this.window = window ?? throw new ArgumentNullException(nameof(window));

    /// <summary>Gets the number of retained editing buffers.</summary>
    public int EditorCount { get { Check(); return editors.Count; } }
    /// <summary>Gets the number of retained live image objects.</summary>
    public int ImageCount { get { Check(); return images.Count; } }

    /// <summary>Normalizes native edits before reporting UTF-16 positions to the model.</summary>
    /// <param name="buffer">Buffer edited in this frame.</param>
    /// <param name="multiline">Whether LF line breaks are retained.</param>
    public static void NormalizeEditor(CuiTextBuffer buffer, bool multiline)
    {
        ArgumentNullException.ThrowIfNull(buffer);
        string before = buffer.Text;
        string Normalize(string value) => value.Replace("\r\n", "\n", StringComparison.Ordinal)
            .Replace('\r', '\n').Replace("\n", multiline ? "\n" : " ", StringComparison.Ordinal);
        string after = Normalize(before);
        if (before == after) return;
        var selection = buffer.Selection;
        buffer.SetText(after, new CuiTextSelection(Normalize(before[..selection.Anchor]).Length,
            Normalize(before[..selection.Caret]).Length));
    }

    /// <summary>Begins tracking which resources this frame uses.</summary>
    public void BeginFrame() { Check(); usedEditors.Clear(); usedImages.Clear(); }

    /// <summary>Reconciles a persistent buffer with the model before drawing.</summary>
    /// <param name="id">Stable control identity.</param>
    /// <param name="text">Model-owned document.</param>
    /// <param name="position">Model-owned selection.</param>
    /// <param name="multiline">Whether the editor permits line breaks.</param>
    /// <returns>The reconciled buffer.</returns>
    public CuiTextBuffer Editor(string id, string text, CuiTextSelection position, bool multiline)
    {
        Check(); usedEditors.Add(id);
        int byteCount = Utf8ByteCount(text);
        int needed = checked(byteCount + 4096);
        if (!editors.TryGetValue(id, out var entry) || entry.Multiline != multiline)
        {
            bool focused = window.HasRetainedFocus(id);
            window.ForgetControl(id);
            entry = new(new CuiTextBuffer(text, Math.Max(4096, needed)), multiline);
            editors[id] = entry;
            if (focused) window.RestoreControlFocus(id);
        }
        else if (entry.Buffer.Capacity - byteCount < 1024)
            entry.Buffer.Grow(Math.Max(needed, checked(entry.Buffer.Capacity * 2)));
        if (entry.Buffer.Text != text) entry.Buffer.SetText(text, position);
        else if (entry.Buffer.Selection != position) entry.Buffer.Selection = position;
        return entry.Buffer;
    }

    /// <summary>Uploads pixels only when their immutable-by-contract source changes.</summary>
    /// <param name="id">Stable image identity.</param>
    /// <param name="width">Source pixel width.</param>
    /// <param name="height">Source pixel height.</param>
    /// <param name="rgba">Immutable packed source bytes.</param>
    /// <returns>The window-owned image.</returns>
    public CuiImage Image(string id, int width, int height, byte[] rgba)
    {
        Check(); usedImages.Add(id);
        if (images.TryGetValue(id, out var entry) && ReferenceEquals(entry.Pixels, rgba) && entry.Width == width && entry.Height == height)
            return entry.Image;
        var image = window.CreateImage(width, height, rgba);
        if (entry is not null) entry.Image.Dispose();
        images[id] = new(image, rgba, width, height);
        return image;
    }

    /// <summary>Releases resources no longer present after a successful frame.</summary>
    public void EndFrame()
    {
        Check();
        foreach (string id in editors.Keys.Where(id => !usedEditors.Contains(id)).ToArray())
        { window.ForgetControl(id); editors.Remove(id); }
        foreach (string id in images.Keys.Where(id => !usedImages.Contains(id)).ToArray())
        { images[id].Image.Dispose(); images.Remove(id); }
    }

    /// <summary>Releases all owned images and native editor state on the UI thread.</summary>
    public void Dispose()
    {
        if (disposed) return;
        Check();
        foreach (string id in editors.Keys) window.ForgetControl(id);
        foreach (var image in images.Values) image.Image.Dispose();
        editors.Clear(); images.Clear(); disposed = true;
    }

    private static int Utf8ByteCount(string text) => Interop.Utf8.Encoding.GetByteCount(text);
    private void Check()
    {
        ObjectDisposedException.ThrowIf(disposed, this);
        if (Environment.CurrentManagedThreadId != owner) throw new InvalidOperationException("Use GUI resources on their owner thread.");
    }
}
