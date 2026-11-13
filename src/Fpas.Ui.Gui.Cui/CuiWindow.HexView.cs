using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

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

public sealed partial class CuiWindow
{
    /// <summary>Draws the native HexView widget.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="data">Source bytes, preserved if edited.</param>
    /// <param name="start">First selected byte.</param>
    /// <param name="length">Number of selected bytes.</param>
    /// <param name="cursor">Caret offset.</param>
    /// <param name="marks">Highlighted byte ranges.</param>
    /// <param name="goTo">Byte offset to reveal, or minus one.</param>
    /// <param name="address">Displayed address of the first byte.</param>
    /// <param name="bytesPerRow">Preferred bytes per row.</param>
    /// <param name="widthKind">Native extent kind.</param>
    /// <param name="width">Extent value for fixed and percentage widths.</param>
    /// <param name="height">Logical height, or the native auto/fill sentinel.</param>
    /// <param name="size">Logical font size, or zero for the default.</param>
    /// <param name="flags">Widget-specific input and rendering flags.</param>
    /// <returns>Input proposals or an updated native observation.</returns>
    public unsafe CuiHexViewEvent? HexView(string id, byte[] data, int start, int length, int cursor, CuiHexMark[] marks, ulong address, int goTo, int bytesPerRow, int widthKind, float width, float height, float size, int flags)
    {
        CheckFrame();
        CheckExtent(widthKind, width, nameof(width));
        byte[] bytes = (flags & 3) == 3 ? (byte[])data.Clone() : data;
        using var memory = new ChartMemory();
        var ranges = new NativeHexMark[marks.Length];
        for (int index = 0; index < marks.Length; index++)
        {
            var mark = marks[index];
            ranges[index] = new() { Start = mark.Start, Length = mark.Length, Tone = mark.Tone, Label = memory.Text(OptionalAi(mark.Label)) };
        }

        fixed (byte* source = bytes)
        fixed (NativeHexMark* nativeMarks = ranges)
        {
            int changed = NativeMethods.HexView(context, Stable(id), source, bytes.Length, ref start, ref length, ref cursor,
                nativeMarks, ranges.Length, address, goTo, bytesPerRow, widthKind, width, height, size, flags, out int offset, out int editLength);
            return changed == 0 ? null : new(start, length, cursor, (changed & 2) != 0 ? bytes : null, offset, editLength);
        }
    }
}
