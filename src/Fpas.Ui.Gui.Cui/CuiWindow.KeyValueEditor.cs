using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's native KeyValueEditor widget.</summary>
    /// <param name="Id">Id supplied to the widget.</param>
    /// <param name="Pairs">Pairs supplied to the widget.</param>
    /// <param name="Capacity">Capacity supplied to the widget.</param>
    /// <param name="KeyPlaceholder">Key Placeholder supplied to the widget.</param>
    /// <param name="ValuePlaceholder">Value Placeholder supplied to the widget.</param>
    /// <param name="HideToggles">Hide Toggles supplied to the widget.</param>
    /// <param name="HideSecrets">Hide Secrets supplied to the widget.</param>
    /// <param name="ReadOnly">Read Only supplied to the widget.</param>
    /// <param name="WidthKind">Width Kind supplied to the widget.</param>
    /// <param name="Width">Width supplied to the widget.</param>
    /// <param name="Enabled">Enabled supplied to the widget.</param>
    /// <returns>Native edit or action proposals.</returns>
    public unsafe CuiDevPair[] KeyValueEditor(string Id, CuiDevPair[] Pairs, int Capacity, string KeyPlaceholder, string ValuePlaceholder, bool HideToggles, bool HideSecrets, bool ReadOnly, int WidthKind, float Width, bool Enabled)
    {
        CheckFrame();
        CheckExtent(WidthKind, Width, nameof(Width));
        if (Capacity < 1 || Capacity > 4096 || Pairs.Length > Capacity) throw new ArgumentOutOfRangeException(nameof(Capacity));
        var native = new NativeDevPair[Capacity];
        int count = Pairs.Length;
        fixed (NativeDevPair* rows = native)
        {
            for (int i = 0; i < count; ++i) rows[i] = PackPair(Pairs[i]);
            NativeMethods.DevKeyValueEditor(context, Stable(Id), rows, ref count, Capacity, OptionalAi(KeyPlaceholder),
                OptionalAi(ValuePlaceholder), HideToggles ? 1 : 0, HideSecrets ? 1 : 0, ReadOnly ? 1 : 0,
                WidthKind, Width, Enabled ? 1 : 0, out var result);
            return Enabled && !ReadOnly && result.Flags != 0 ? ReadPairs(rows, count) : Pairs;
        }
    }
}
