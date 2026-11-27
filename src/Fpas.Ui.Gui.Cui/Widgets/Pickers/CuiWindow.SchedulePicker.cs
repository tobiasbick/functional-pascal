// [Documentation](../../../../docs/ui/gui/developer-tools/schedule-picker.md)
using Fpas.Ui.Gui.Cui.Widgets.Developer;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's native SchedulePicker widget.</summary>
    /// <param name="Id">Id supplied to the widget.</param>
    /// <param name="Expression">Expression supplied to the widget.</param>
    /// <param name="Capacity">Maximum expression bytes excluding the terminator, at least 11.</param>
    /// <param name="Now">Reference time in Unix seconds, or zero for the clock.</param>
    /// <param name="Utc">Use UTC for the schedule and preview.</param>
    /// <param name="NextCount">Number of upcoming runs to display, from zero through ten.</param>
    /// <param name="WidthKind">Width Kind supplied to the widget.</param>
    /// <param name="Width">Width supplied to the widget.</param>
    /// <param name="Enabled">Enabled supplied to the widget.</param>
    /// <returns>Native edit or action proposals.</returns>
    public unsafe CuiScheduleProposal SchedulePicker(string Id, string Expression, int Capacity, long Now, bool Utc, int NextCount, int WidthKind, float Width, bool Enabled)
    {
        CheckFrame();
        CheckExtent(WidthKind, Width, nameof(Width));
        if (Capacity < 11) throw new ArgumentOutOfRangeException(nameof(Capacity));
        if (NextCount is < 0 or > 10) throw new ArgumentOutOfRangeException(nameof(NextCount));
        if (Now is < -62135596800L or > 253402300799L) throw new ArgumentOutOfRangeException(nameof(Now));
        byte[] value = DevBuffer(Expression, Capacity);
        fixed (byte* text = value)
        {
            NativeMethods.DevSchedulePicker(context, Stable(Id), text, (nuint)value.Length, Now, Utc ? 1 : 0, NextCount,
                WidthKind, Width, Enabled ? 1 : 0, out var result);
            return new(Enabled ? ReadDevText(value) : Expression, (result.Flags & 2) != 0, (long)result.X);
        }
    }
}
