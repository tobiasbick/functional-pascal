// [Documentation](../../../../docs/ui/gui/overlays/tooltip.md)
using Fpas.Ui.Gui.Cui.Widgets.Overlays;
using System.Runtime.InteropServices;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Attaches native pointer help to the last widget's interactive anchor.</summary>
    /// <param name="text">Nonblank caption without CUI's shortcut delimiter.</param>
    /// <param name="shortcut">Optional shortcut hint. The formatted tooltip fits in 255 UTF-8 bytes.</param>
    public void Tooltip(string text, string shortcut = "")
    {
        CheckFrame();
        CheckTooltipText(text, shortcut, nameof(text));
        NativeMethods.Tooltip(context, text, Optional(shortcut));
    }

    private static void CheckTooltipText(string text, string shortcut, string parameter, bool allowEmpty = false)
    {
        Utf8.Validate(text, parameter);
        Utf8.Validate(shortcut, nameof(shortcut));
        if (allowEmpty && text.Length == 0) return;
        ArgumentException.ThrowIfNullOrWhiteSpace(text, parameter);
        if (text.Contains("  ·  ", StringComparison.Ordinal))
            throw new ArgumentException("The caption contains CUI's shortcut delimiter.", parameter);
        int length = Utf8.Encoding.GetByteCount(text)
            + (shortcut.Length == 0 ? 0 : 6 + Utf8.Encoding.GetByteCount(shortcut));
        if (length > 255)
            throw new ArgumentException("Tooltip text and its shortcut must fit in 255 UTF-8 bytes.", parameter);
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
