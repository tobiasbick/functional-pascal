// [Documentation](../../../docs/ui/gui/cui.md#theme-and-resources)
using Fpas.Ui.Gui.Cui.Interop;
using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Resources;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    private nint ownedStrings;

    private unsafe void ReleaseOwnedStrings()
    {
        if (ownedStrings != 0) ownedStrings = NativeMethods.SetStrings(context, ownedStrings, null);
    }

    /// <summary>Copies a caption table into the native context, replacing the previous owned table.</summary>
    /// <param name="strings">Partial caption table; null restores all English defaults.</param>
    public unsafe void SetStrings(CuiStrings? strings)
    {
        CheckResourceChange();
        if (strings is null) { ownedStrings = NativeMethods.SetStrings(context, ownedStrings, null); return; }
        var values = new List<string?>(46);
        Add(strings.Months, 12); Add(strings.ShortMonths, 12);
        Add(strings.Weekdays, 7); Add(strings.ShortWeekdays, 7); Add(strings.MinimalWeekdays, 7);
        values.Add(strings.Today);
        foreach (string? value in values) if (value is not null) Utf8.Validate(value, nameof(strings));
        using var memory = new ChartMemory();
        var pointers = new nint[46];
        for (int i = 0; i < pointers.Length; i++) pointers[i] = (nint)memory.Text(values[i]);
        fixed (nint* data = pointers) ownedStrings = NativeMethods.SetStrings(context, ownedStrings, (byte**)data);

        void Add(string?[]? source, int count)
        {
            if (source is not null && source.Length != count)
                throw new ArgumentException($"Caption arrays must contain exactly {count} entries.", nameof(strings));
            if (source is null) values.AddRange(new string?[count]);
            else values.AddRange(source);
        }
    }
}
