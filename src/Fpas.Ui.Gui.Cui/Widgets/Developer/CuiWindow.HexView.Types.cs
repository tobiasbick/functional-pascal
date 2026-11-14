using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Developer;

/// <summary>A native byte highlight.</summary>
/// <param name="Start">First byte.</param>
/// <param name="Length">Number of bytes.</param>
/// <param name="Tone">Native highlight tone.</param>
/// <param name="Label">Optional tooltip.</param>
public sealed record CuiHexMark(int Start, int Length, int Tone, string Label);
/// <summary>Native selection and overwrite result.</summary>
/// <param name="Start">First selected byte.</param>
/// <param name="Length">Selection length.</param>
/// <param name="Cursor">Caret offset.</param>
/// <param name="Data">Edited copy, or null.</param>
/// <param name="EditOffset">First edited byte.</param>
/// <param name="EditLength">Number of edited bytes.</param>
public sealed record CuiHexViewEvent(int Start, int Length, int Cursor, byte[]? Data, int EditOffset, int EditLength);

[System.Runtime.InteropServices.StructLayout(System.Runtime.InteropServices.LayoutKind.Sequential)]
internal unsafe struct NativeHexMark
{
    public int Start, Length, Tone;
    public byte* Label;
}
