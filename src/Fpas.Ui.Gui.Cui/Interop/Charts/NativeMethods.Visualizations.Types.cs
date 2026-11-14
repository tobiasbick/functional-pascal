using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop.Charts;

[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeVizItem { public byte* Label; public float Value; public int Palette; public uint Rgba; }
