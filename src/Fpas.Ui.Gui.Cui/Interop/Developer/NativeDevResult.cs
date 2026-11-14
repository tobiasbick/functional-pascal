using Fpas.Ui.Gui.Cui.Widgets.Developer;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop.Developer;

[StructLayout(LayoutKind.Sequential)]
internal struct NativeDevResult
{
    internal int Index;
    internal int Action;
    internal int Hover;
    internal int Count;
    internal int Flags;
    internal double X;
    internal double Y;
    internal double Z;
    internal double A;
    internal double B;

    internal readonly CuiDevResult ToManaged() => new(Index, Action, Hover, Count, Flags, X, Y, Z, A, B);
}
