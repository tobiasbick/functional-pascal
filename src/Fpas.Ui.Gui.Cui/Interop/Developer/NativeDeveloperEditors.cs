using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop.Developer;

[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeDevPair
{
    internal fixed byte Key[64];
    internal fixed byte Value[256];
    internal byte Enabled;
    internal byte Secret;
}

[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeDevField
{
    internal byte* Name;
    internal int Kind;
    internal byte** Choices;
    internal int Count;
}

[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeDevRule
{
    internal int Depth;
    internal byte Group;
    internal byte Any;
    internal int Field;
    internal int Operator;
    internal fixed byte Value[64];
}

[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeDevFlag
{
    internal byte* Key;
    internal byte* Description;
    internal fixed byte On[8];
    internal fixed int Rollout[8];
}
