// [Documentation](../../../../docs/ui/gui/developer-tools/key-value-editor.md)
using Fpas.Ui.Gui.Cui.Interop.Developer;
using Fpas.Ui.Gui.Cui.Widgets.Developer;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's native KeyValueEditor widget.</summary>
    /// <param name="Id">Id supplied to the widget.</param>
    /// <param name="Pairs">Pairs supplied to the widget.</param>
    /// <param name="Capacity">Capacity supplied to the widget.</param>
    /// <param name="KeyPlaceholder">Caption for an empty key, or null for the native default.</param>
    /// <param name="ValuePlaceholder">Caption for an empty value, or null for the native default.</param>
    /// <param name="HideToggles">Hide Toggles supplied to the widget.</param>
    /// <param name="HideSecrets">Hide Secrets supplied to the widget.</param>
    /// <param name="ReadOnly">Read Only supplied to the widget.</param>
    /// <param name="WidthKind">Width Kind supplied to the widget.</param>
    /// <param name="Width">Width supplied to the widget.</param>
    /// <param name="Enabled">Enabled supplied to the widget.</param>
    /// <returns>Native edit or action proposals.</returns>
    public unsafe CuiDevPair[] KeyValueEditor(string Id, CuiDevPair[] Pairs, int Capacity, string? KeyPlaceholder, string? ValuePlaceholder, bool HideToggles, bool HideSecrets, bool ReadOnly, int WidthKind, float Width, bool Enabled)
    {
        CheckFrame();
        CheckExtent(WidthKind, Width, nameof(Width));
        ArgumentNullException.ThrowIfNull(Pairs);
        if (KeyPlaceholder is not null) Utf8.Validate(KeyPlaceholder, nameof(KeyPlaceholder));
        if (ValuePlaceholder is not null) Utf8.Validate(ValuePlaceholder, nameof(ValuePlaceholder));
        if (Capacity < 1 || Capacity > 4096 || Pairs.Length > Capacity) throw new ArgumentOutOfRangeException(nameof(Capacity));
        var native = new NativeDevPair[Capacity];
        int count = Pairs.Length;
        fixed (NativeDevPair* rows = native)
        {
            for (int i = 0; i < count; ++i) rows[i] = PackPair(Pairs[i]);
            NativeMethods.DevKeyValueEditor(context, Stable(Id), rows, ref count, Capacity, KeyPlaceholder,
                ValuePlaceholder, HideToggles ? 1 : 0, HideSecrets ? 1 : 0, ReadOnly ? 1 : 0,
                WidthKind, Width, Enabled ? 1 : 0, out var result);
            return Enabled && !ReadOnly && result.Flags != 0 ? ReadPairs(rows, count) : Pairs;
        }
    }
}
