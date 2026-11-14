using Fpas.Ui.Gui.Cui.Widgets.Inputs;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's shortcut recorder and its optional clear action.</summary>
    /// <param name="id">Stable application identity.</param>
    /// <param name="value">Model-owned combination.</param>
    /// <param name="placeholder">Literal empty-state caption.</param>
    /// <param name="widthKind">Content zero, fill one, fixed two or percentage three.</param>
    /// <param name="width">Logical extent or percentage.</param>
    /// <param name="enabled">Whether recording and clearing accept input.</param>
    /// <param name="focus">Whether to request keyboard focus.</param>
    /// <param name="clearControl">Clear action tracking ID, or null to omit tracking.</param>
    /// <param name="focusClear">Whether to focus the native clear action.</param>
    /// <returns>The proposed combination and visible native affordances.</returns>
    public unsafe CuiShortcutRecorderResult ShortcutRecorder(string id, CuiKeyCombination value, string placeholder,
        int widthKind, float width, bool enabled, bool focus, string? clearControl, bool focusClear = false)
    {
        CheckFrame(); string identity = Stable(id);
        CheckKeyCombination(value); CheckSmartWidth(widthKind, width); Utf8.Validate(placeholder, nameof(placeholder));
        int key = value.Key, modifiers = value.Modifiers;
        int* nodes = stackalloc int[2];
        NativeMethods.ShortcutRecorder(context, identity, ref key, ref modifiers, Optional(placeholder),
            widthKind, width, enabled ? 1 : 0, focusClear ? 2 : focus ? 1 : 0, nodes);
        if (clearControl is not null)
        {
            TrackControl(id, nodes[0]);
            if (nodes[1] >= 0) TrackControl(clearControl, nodes[1]);
        }
        return new(new(key, modifiers), nodes[1] >= 0);
    }

    /// <summary>Formats a shortcut using CUI's platform-specific key names and modifier order.</summary>
    /// <param name="value">Validated key and modifiers.</param>
    /// <returns>Copied native label, empty for no shortcut.</returns>
    public unsafe string ShortcutLabel(CuiKeyCombination value)
    {
        CheckFrame(); CheckKeyCombination(value);
        Span<byte> text = stackalloc byte[128];
        fixed (byte* buffer = text) NativeMethods.ShortcutLabel(context, value.Key, value.Modifiers, buffer, (nuint)text.Length);
        return Utf8.Encoding.GetString(text[..text.IndexOf((byte)0)]);
    }

    /// <summary>Draws native keycaps for a combination using CUI's formatter.</summary>
    /// <param name="value">Validated key and modifiers.</param>
    /// <returns>The literal label displayed by CUI.</returns>
    public unsafe string DrawShortcutLabel(CuiKeyCombination value)
    {
        CheckFrame(); CheckKeyCombination(value);
        Span<byte> text = stackalloc byte[128];
        fixed (byte* buffer = text) NativeMethods.ShortcutDisplay(context, value.Key, value.Modifiers, buffer, (nuint)text.Length);
        return Utf8.Encoding.GetString(text[..text.IndexOf((byte)0)]);
    }
}
