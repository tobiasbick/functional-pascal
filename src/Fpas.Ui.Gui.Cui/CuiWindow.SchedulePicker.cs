using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's native SchedulePicker widget.</summary>
    /// <param name="Id">Id supplied to the widget.</param>
    /// <param name="Expression">Expression supplied to the widget.</param>
    /// <param name="Capacity">Capacity supplied to the widget.</param>
    /// <param name="Now">Now supplied to the widget.</param>
    /// <param name="Utc">Utc supplied to the widget.</param>
    /// <param name="NextCount">Next Count supplied to the widget.</param>
    /// <param name="WidthKind">Width Kind supplied to the widget.</param>
    /// <param name="Width">Width supplied to the widget.</param>
    /// <param name="Enabled">Enabled supplied to the widget.</param>
    /// <returns>Native edit or action proposals.</returns>
    public unsafe CuiScheduleProposal SchedulePicker(string Id, string Expression, int Capacity, long Now, bool Utc, int NextCount, int WidthKind, float Width, bool Enabled)
    {
        CheckFrame();
        CheckExtent(WidthKind, Width, nameof(Width));
        byte[] value = DevBuffer(Expression, Capacity);
        fixed (byte* text = value)
        {
            NativeMethods.DevSchedulePicker(context, Stable(Id), text, (nuint)value.Length, Now, Utc ? 1 : 0, NextCount,
                WidthKind, Width, Enabled ? 1 : 0, out var result);
            return new(Enabled ? ReadDevText(value) : Expression, (result.Flags & 2) != 0, (long)result.X);
        }
    }
}
