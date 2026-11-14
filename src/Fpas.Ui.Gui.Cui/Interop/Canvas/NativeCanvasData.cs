using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop.Canvas;

[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeTextStyle
{
    internal byte* Name;
    internal float Size, Preview;
    internal int Font;
}

[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeLivePerson
{
    internal byte* Name;
    internal uint Rgba;
    internal int Tone;
    internal float X, Y;
    internal byte* Says;
    internal byte Speaking, Away;
    internal nuint Avatar;
}

[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeCanvasToggle
{
    internal byte* Icon;
    internal byte* Label;
}

[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeSelectionAction
{
    internal byte* Icon;
    internal byte* Label;
    internal int Tone;
}
