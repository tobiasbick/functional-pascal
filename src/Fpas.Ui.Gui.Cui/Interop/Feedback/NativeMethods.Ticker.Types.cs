// [Documentation](../../../../docs/ui/gui/social-and-media/ticker.md)
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop.Feedback;

[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeTickerItem
{
    internal byte* Label, Value;
    internal float Delta;
    internal int LowerIsBetter;
}
