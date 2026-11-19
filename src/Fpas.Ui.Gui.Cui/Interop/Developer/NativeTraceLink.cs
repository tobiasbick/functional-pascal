// [Documentation](../../../../docs/ui/gui/developer-tools/trace-viewer.md)
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop.Developer;

[StructLayout(LayoutKind.Sequential)]
internal struct NativeTraceLink
{
    internal int From, To, Tone;
    internal float Width;
    internal byte Dashed;
}
