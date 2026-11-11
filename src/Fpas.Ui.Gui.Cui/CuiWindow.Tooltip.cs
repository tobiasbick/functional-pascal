using System.Runtime.InteropServices;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

/// <summary>Visible native tooltip in a completed frame.</summary>
/// <param name="Text">Rendered caption with its optional shortcut hint.</param>
/// <param name="Bounds">Finalized bounds in logical points.</param>
public sealed record CuiTooltipFrame(string Text, CuiRectangle Bounds);

public sealed partial class CuiWindow
{
    /// <summary>Attaches native pointer help to the last widget's interactive anchor.</summary>
    /// <param name="text">Nonblank caption without CUI's shortcut delimiter.</param>
    /// <param name="shortcut">Optional shortcut hint. The formatted tooltip fits in 255 UTF-8 bytes.</param>
    public void Tooltip(string text, string shortcut = "")
    {
        CheckFrame();
        Utf8.Validate(text, nameof(text));
        Utf8.Validate(shortcut, nameof(shortcut));
        ArgumentException.ThrowIfNullOrWhiteSpace(text);
        if (text.Contains("  ·  ", StringComparison.Ordinal))
            throw new ArgumentException("The caption contains CUI's shortcut delimiter.", nameof(text));
        int length = Utf8.Encoding.GetByteCount(text)
            + (shortcut.Length == 0 ? 0 : 6 + Utf8.Encoding.GetByteCount(shortcut));
        if (length > 255)
            throw new ArgumentException("Tooltip text and its shortcut must fit in 255 UTF-8 bytes.", nameof(text));
        NativeMethods.Tooltip(context, text, shortcut.Length == 0 ? null : shortcut);
    }

    /// <summary>Reads the tooltip rendered by CUI in the completed frame.</summary>
    /// <returns>The visible tooltip, or null when none was drawn.</returns>
    public CuiTooltipFrame? VisibleTooltip()
    {
        CheckOpen();
        if (inFrame || !hasPresentedFrame)
            throw new InvalidOperationException("Present the current frame before reading its tooltip.");
        nint text = NativeMethods.TooltipInspect(context, out int node);
        return text == 0 ? null : new(Marshal.PtrToStringUTF8(text)!, NativeMethods.NodeRectangle(context, node));
    }
}
