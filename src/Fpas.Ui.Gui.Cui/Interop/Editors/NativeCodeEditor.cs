using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

[StructLayout(LayoutKind.Sequential)]
internal struct NativeTextMark
{
    internal int Start, End, Color;
}

[StructLayout(LayoutKind.Sequential)]
internal struct NativeCodeEditor
{
    internal int Line, Column, Lines, Cursor, Anchor, Matches, MatchIndex, Focused;
}
