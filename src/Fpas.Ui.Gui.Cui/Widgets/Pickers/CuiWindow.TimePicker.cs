using Fpas.Ui.Gui.Cui.Widgets.Pickers;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws the native time field and its scrolling columns.</summary>
    /// <param name="id">Stable field identity.</param>
    /// <param name="value">Model time at minute precision.</param>
    /// <param name="label">Optional field caption.</param>
    /// <param name="width">Logical field width, or zero for CUI's default.</param>
    /// <param name="twelveHour">Whether to display AM and PM.</param>
    /// <param name="minuteStep">Spacing between minute choices, from one to sixty.</param>
    /// <param name="enabled">Whether the field accepts input.</param>
    /// <param name="focus">Whether to request trigger focus.</param>
    /// <param name="trackChoice">Maps native choice codes to tracking IDs, or null.</param>
    /// <returns>The proposed time and the tracked choices.</returns>
    public unsafe CuiTimePickerResult TimePicker(string id, TimeOnly value, string label, float width,
        bool twelveHour, int minuteStep, bool enabled, bool focus, Func<int, string>? trackChoice)
    {
        CheckFrame();
        Utf8.Validate(label, nameof(label));
        if (!float.IsFinite(width) || width < 0) throw new ArgumentOutOfRangeException(nameof(width));
        if (minuteStep is < 1 or > 60) throw new ArgumentOutOfRangeException(nameof(minuteStep));
        const int nodeCount = 1 + 24 + 60 + 2;
        int* nodes = stackalloc int[nodeCount];
        int proposed = NativeMethods.TimePicker(context, Stable(id), MinuteOfDay(value), Optional(label), width,
            twelveHour ? 1 : 0, minuteStep, enabled ? 1 : 0, focus ? 1 : 0, trackChoice is null ? null : nodes);
        var time = new TimeOnly(proposed / 60, proposed % 60);
        if (trackChoice is null) return new(time, []);

        TrackControl(id, nodes[0]);
        List<int> choices = [];
        for (int i = 1; i < nodeCount; ++i)
        {
            if (nodes[i] < 0) continue;
            TrackControl(trackChoice(i - 1), nodes[i]);
            choices.Add(i - 1);
        }
        return new(time, choices.ToArray());
    }

    private static int MinuteOfDay(TimeOnly value)
    {
        if (value.Ticks % TimeSpan.TicksPerMinute != 0)
            throw new ArgumentException("The time must have minute precision.", nameof(value));
        return value.Hour * 60 + value.Minute;
    }
}
