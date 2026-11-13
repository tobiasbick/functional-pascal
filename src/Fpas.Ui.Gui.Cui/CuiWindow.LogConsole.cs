using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

/// <summary>One native log line.</summary>
/// <param name="Text">Literal content.</param>
/// <param name="Time">Timestamp in seconds.</param>
/// <param name="Level">Native severity.</param>
public sealed record CuiLogEntry(string Text, double Time, int Level);

[System.Runtime.InteropServices.StructLayout(System.Runtime.InteropServices.LayoutKind.Sequential)]
internal unsafe struct NativeLogEntry
{
    public byte* Text;
    public double Time;
    public int Level;
}

public sealed partial class CuiWindow
{
    /// <summary>Draws the native LogConsole widget.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="entries">Log lines in arrival order.</param>
    /// <param name="widthKind">Native extent kind.</param>
    /// <param name="width">Extent value for fixed and percentage widths.</param>
    /// <param name="height">Logical height, or the native auto/fill sentinel.</param>
    /// <param name="size">Logical font size, or zero for the default.</param>
    /// <param name="flags">Widget-specific input and rendering flags.</param>
    /// <returns>Input proposals or an updated native observation.</returns>
    public unsafe bool LogConsole(string id, CuiLogEntry[] entries, int widthKind, float width, float height, float size, int flags)
    {
        CheckFrame();
        CheckExtent(widthKind, width, nameof(width));
        using var memory = new ChartMemory();
        var rows = new NativeLogEntry[entries.Length];
        for (int index = 0; index < entries.Length; index++)
        {
            var entry = entries[index];
            rows[index] = new() { Text = memory.Text(entry.Text), Time = entry.Time, Level = entry.Level };
        }

        fixed (NativeLogEntry* nativeRows = rows)
            return NativeMethods.LogConsole(context, Stable(id), nativeRows, rows.Length, widthKind, width, height, size, flags) != 0;
    }
}
