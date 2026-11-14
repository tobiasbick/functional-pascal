using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop.Charts;

[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeEventOptions
{
    internal double Minimum, Maximum, Step, LabelStep;
    internal byte* Format;
    internal float Width;
    internal int WidthKind, Units, Flags;
}
