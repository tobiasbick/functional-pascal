// [Documentation](../../../../docs/ui/gui/smart-inputs/money-field.md)
using Fpas.Ui.Gui.Cui.Interop;
using Fpas.Ui.Gui.Cui.Widgets.Inputs;
using System.Text;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Edits an exact decimal amount through CUI's native text field.</summary>
    /// <param name="id">Stable application identity.</param>
    /// <param name="value">Model amount representable at the selected fractional precision.</param>
    /// <param name="label">Literal caption.</param>
    /// <param name="symbol">Currency prefix; empty hides it.</param>
    /// <param name="code">Optional currency-code suffix.</param>
    /// <param name="hint">Optional supporting text.</param>
    /// <param name="decimalPlaces">Fractional precision from zero through twenty-eight; no implicit rounding.</param>
    /// <param name="widthKind">Content zero, fill one, fixed two or percentage three.</param>
    /// <param name="width">Logical extent or percentage.</param>
    /// <param name="enabled">Whether input is accepted.</param>
    /// <param name="focus">Whether to request editor focus.</param>
    /// <param name="track">Whether to retain and inspect the control.</param>
    /// <returns>An exactly representable proposal; invalid or incomplete drafts retain the model amount.</returns>
    public unsafe decimal MoneyField(string id, decimal value, string label, string symbol, string code, string hint,
        int decimalPlaces, int widthKind, float width, bool enabled, bool focus, bool track)
    {
        CheckFrame(); string identity = Stable(id);
        MoneyAmounts.Validate(value, decimalPlaces); CheckSmartWidth(widthKind, width);
        Utf8.Validate(label, nameof(label)); Utf8.Validate(symbol, nameof(symbol));
        Utf8.Validate(code, nameof(code)); Utf8.Validate(hint, nameof(hint));
        if (!moneyFields.TryGetValue(id, out var edit))
        {
            edit = new MoneyEdit(value, decimalPlaces);
            moneyFields.Add(id, edit);
        }
        if (edit.Expected != value || edit.Places != decimalPlaces || !enabled || !edit.Focused)
            edit.Reset(value, decimalPlaces);
        Span<byte> buffer = stackalloc byte[129];
        int length = Utf8.Encoding.GetBytes(edit.Draft, buffer);
        buffer[length] = 0;
        int flags, node;
        fixed (byte* bytes = buffer)
            flags = NativeMethods.MoneyField(context, identity, bytes, (nuint)buffer.Length,
                Optional(label), symbol, Optional(code), Optional(hint),
                edit.Invalid ? "Enter an exact decimal amount within the selected precision." : null,
                widthKind, width, enabled ? 1 : 0, focus ? 1 : 0, out node);
        edit.Key = NativeMethods.NodeKey(context, node);
        moneyDrawn.Add(id);
        if (track) TrackControl(id, node);
        string draft;
        try { draft = Utf8.Encoding.GetString(buffer[..buffer.IndexOf((byte)0)]); }
        catch (DecoderFallbackException) { draft = edit.Draft; }
        decimal proposed = value;
        bool valid = MoneyAmounts.TryParse(draft, decimalPlaces, out decimal parsed);
        if (enabled && (flags & 1) != 0 && valid) proposed = parsed;
        edit.Expected = proposed;
        edit.Focused = (flags & 4) != 0;
        edit.Draft = draft;
        edit.Invalid = !valid;
        if (!edit.Focused || (flags & 2) != 0) edit.Reset(proposed, decimalPlaces);
        return proposed;
    }
}
