using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Developer;

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
