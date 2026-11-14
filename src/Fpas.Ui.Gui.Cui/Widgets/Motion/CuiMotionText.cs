using Fpas.Ui.Gui.Cui.Widgets.Motion;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    private static NativeMotionText MotionOptions(CuiMotionText value)
    {
        CheckExtent(value.WidthKind, value.Width, nameof(value));
        CheckFeedbackNumber(value.Size, nameof(value));
        CheckFeedbackNumber(value.Speed, nameof(value));
        CheckFeedbackNumber(value.Delay, nameof(value));
        if (value.Font is < 0 or > 3 || value.Color.Palette is < -1 or > 6)
            throw new ArgumentOutOfRangeException(nameof(value));
        return new() { Font = value.Font, WidthKind = value.WidthKind, Width = value.Width,
            Size = value.Size, Speed = value.Speed, Delay = value.Delay,
            Palette = value.Color.Palette, Rgba = value.Color.Rgba,
            Flags = (value.NoWrap ? 1 : 0) | (value.Caret ? 2 : 0) };
    }
}
