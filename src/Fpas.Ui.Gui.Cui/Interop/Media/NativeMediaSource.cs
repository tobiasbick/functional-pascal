using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop.Media;

[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeMediaSource
{
    internal byte* Path;
    internal byte* Pixels;
    internal int Width, Height;
    internal long Version;
}
