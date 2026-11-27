// [Documentation](../../../../docs/ui/gui/cui.md#data-and-marshalling)
using Fpas.Ui.Gui.Cui.Interop;
using System.Globalization;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    private static bool ValidAudioSize(float value) => float.IsFinite(value) && value >= 0;

    private static string SliderNumberFormat(int decimalPlaces, bool showSign = false)
    {
        if (decimalPlaces is < 0 or > 6) throw new ArgumentOutOfRangeException(nameof(decimalPlaces));
        return (showSign ? "%+." : "%.") + decimalPlaces.ToString(CultureInfo.InvariantCulture) + "f";
    }

    private static void ValidateAudioDial(float step, float reset, string unit, float span)
    {
        Utf8.Validate(unit, nameof(unit));
        if (!ValidAudioSize(step) || (step > 0 && !float.IsFinite(span / step)))
            throw new ArgumentOutOfRangeException(nameof(step));
        if (!float.IsFinite(reset)) throw new ArgumentOutOfRangeException(nameof(reset));
    }

    private static void ValidateAudioLevels(float[] levels, int minimumChannels)
    {
        ArgumentNullException.ThrowIfNull(levels);
        if (levels.Length < minimumChannels || levels.Length > 8)
            throw new ArgumentOutOfRangeException(nameof(levels));
        foreach (float peak in levels)
            if (!ValidAudioSize(peak)) throw new ArgumentOutOfRangeException(nameof(levels));
    }
}
