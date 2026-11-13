using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

/// <summary>A native filter field.</summary>
/// <param name="Name">Literal heading.</param>
/// <param name="Icon">Optional CUI glyph.</param>
/// <param name="Values">Options in mask bit order.</param>
public readonly record struct CuiFilterField(string Name, string Icon, string[] Values);

/// <summary>The native filter bar's output.</summary>
/// <param name="Changed">Whether the native widget changed a model value.</param>
/// <param name="Search">Resulting search, or null when hidden.</param>
/// <param name="Parts">Whether the search, add, clear and individual chips exist in the frame.</param>
public readonly record struct CuiFilterResult(bool Changed, string? Search, bool[] Parts);

public sealed partial class CuiWindow
{
    /// <summary>Draws the native filter bar and updates the supplied mask copy.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="fields">Available filter fields.</param>
    /// <param name="selected">Writable mask copy, one entry per field.</param>
    /// <param name="search">Current text or null to omit the field.</param>
    /// <param name="capacity">UTF-8 search capacity, including the terminator.</param>
    /// <param name="enabled">Whether input is accepted.</param>
    /// <param name="focus">Whether to focus the search or add button.</param>
    /// <param name="partId">Optional inspection key mapper. Parts are search, add, clear, then chips.</param>
    /// <returns>The native change, search and available parts.</returns>
    public unsafe CuiFilterResult FilterBar(string id, CuiFilterField[] fields, uint[] selected, string? search,
        int capacity, bool enabled, bool focus, Func<int, string>? partId)
    {
        CheckFrame();
        CheckPopupDepth();
        if (fields.Length > 32 || fields.Length != selected.Length)
            throw new ArgumentException("Expected up to 32 fields and one mask per field.", nameof(fields));
        var names = new string[fields.Length];
        var icons = new string[fields.Length];
        var counts = new int[fields.Length];
        var options = new List<string>();
        for (int k = 0; k < fields.Length; k++)
        {
            var field = fields[k];
            Utf8.Validate(field.Name, nameof(fields));
            Utf8.Validate(field.Icon, nameof(fields));
            int count = field.Values.Length;
            if (count > 32 || count < 32 && selected[k] >> count != 0)
                throw new ArgumentException("An option mask must name available values.", nameof(selected));
            names[k] = field.Name;
            icons[k] = field.Icon;
            counts[k] = count;
            foreach (string option in field.Values)
            {
                Utf8.Validate(option, nameof(fields));
                options.Add(option);
            }
        }

        byte[]? buffer = search is null ? null : GridTextBuffer(search, capacity, 65536);
        var parts = new int[4 + fields.Length];
        var values = options.ToArray();
        int changed;
        fixed (uint* masks = selected)
        fixed (byte* text = buffer)
        fixed (int* sizes = counts, nodes = parts)
            changed = NativeMethods.FilterBar(context, Stable(id), names, icons, values, sizes,
                fields.Length, masks, text, buffer?.Length ?? 0, enabled ? 1 : 0, focus ? 1 : 0, nodes);

        var available = new bool[parts.Length - 1];
        if (partId is not null)
            for (int k = 1; k < parts.Length; k++)
                if (available[k - 1] = parts[k] >= 0)
                    TrackControl(partId(k - 1), parts[k]);
        return new(changed != 0, buffer is null ? null : ReadGridText(buffer), available);
    }
}
