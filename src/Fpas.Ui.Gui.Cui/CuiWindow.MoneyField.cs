using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's money editor with grouped resting text and plain editing digits.</summary>
    /// <param name="id">Stable application identity.</param>
    /// <param name="value">Finite model amount fitting the native grouped display.</param>
    /// <param name="label">Literal caption.</param>
    /// <param name="symbol">Currency prefix; empty hides it.</param>
    /// <param name="code">Optional currency-code suffix.</param>
    /// <param name="hint">Optional supporting text.</param>
    /// <param name="decimalPlaces">Precision from zero through fifteen, two by default.</param>
    /// <param name="widthKind">Content zero, fill one, fixed two or percentage three.</param>
    /// <param name="width">Logical extent or percentage.</param>
    /// <param name="enabled">Whether input is accepted.</param>
    /// <param name="focus">Whether to request editor focus.</param>
    /// <param name="track">Whether to retain and inspect the control.</param>
    /// <returns>The proposed native amount.</returns>
    public double MoneyField(string id, double value, string label, string symbol, string code, string hint,
        int decimalPlaces, int widthKind, float width, bool enabled, bool focus, bool track)
    {
        CheckFrame(); string identity = Stable(id);
        CheckMoneyValue(value, decimalPlaces); CheckSmartWidth(widthKind, width);
        Utf8.Validate(label, nameof(label)); Utf8.Validate(symbol, nameof(symbol));
        Utf8.Validate(code, nameof(code)); Utf8.Validate(hint, nameof(hint));
        NativeMethods.MoneyField(context, identity, ref value, Optional(label), symbol, Optional(code),
            Optional(hint), decimalPlaces, widthKind, width, enabled ? 1 : 0, focus ? 1 : 0, out int node);
        if (track) TrackControl(id, node);
        return value;
    }
}
