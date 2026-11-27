// [Documentation](../../../../docs/ui/gui/instruments/dot-chart.md)
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop.Charts;

[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeDotOptions
{
    internal NativeChartOptions Chart;
    internal double Minimum, Maximum, Step;
    internal byte* XFormat;
    internal byte* Unit;
    internal float Pitch;
    internal int Units, Flags;
    internal nint XLabel, XLabelUser, ValueLabel, ValueLabelUser;
    internal byte** Names;
}
