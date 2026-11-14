using Fpas.Ui.Gui.Cui.Widgets.Inputs;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws native radio cards with model-owned selection.</summary>
    /// <param name="id">Stable group identity.</param>
    /// <param name="cards">Nonempty positional choices.</param>
    /// <param name="selected">Model-owned index, or minus one.</param>
    /// <param name="columns">Maximum columns, or zero for automatic layout.</param>
    /// <param name="enabled">Whether the group accepts input.</param>
    /// <param name="focus">Zero or one plus the requested focus index.</param>
    /// <param name="trackingIds">Grid and card tracking IDs, or null.</param>
    /// <returns>A proposed index, or minus one when the selection is unchanged.</returns>
    public unsafe int RadioCards(string id, CuiRadioCard[] cards, int selected, int columns,
        bool enabled, int focus, string[]? trackingIds)
    {
        CheckFrame();
        ArgumentNullException.ThrowIfNull(cards);
        int count = cards.Length;
        if (count == 0) throw new ArgumentException("Supply at least one card.", nameof(cards));
        if (selected < -1 || selected >= count) throw new ArgumentOutOfRangeException(nameof(selected));
        ArgumentOutOfRangeException.ThrowIfNegative(columns);
        if (focus < 0 || focus > count) throw new ArgumentOutOfRangeException(nameof(focus));
        if (trackingIds is not null && trackingIds.Length != count + 1)
            throw new ArgumentException("Supply the grid and card tracking IDs.", nameof(trackingIds));
        var titles = new string[count];
        var descriptions = new string[count];
        var icons = new string[count];
        var badges = new string[count];
        for (int index = 0; index < count; index++)
        {
            CuiRadioCard card = cards[index];
            Utf8.Validate(card.Title, nameof(cards));
            if (string.IsNullOrWhiteSpace(card.Title)) throw new ArgumentException("Each card needs a title.", nameof(cards));
            Utf8.Validate(card.Description, nameof(cards));
            Utf8.Validate(card.Icon, nameof(cards));
            Utf8.Validate(card.Badge, nameof(cards));
            titles[index] = card.Title;
            descriptions[index] = card.Description;
            icons[index] = card.Icon;
            badges[index] = card.Badge;
        }
        var nodes = new int[count + 1];
        int proposal;
        fixed (int* parts = nodes)
            proposal = NativeMethods.RadioCards(context, Stable(id), titles, descriptions, icons, badges,
                count, selected, columns, enabled ? 1 : 0, focus, parts);
        if (trackingIds is not null)
            for (int index = 0; index < nodes.Length; index++) TrackControl(trackingIds[index], nodes[index]);
        return proposal;
    }
}
