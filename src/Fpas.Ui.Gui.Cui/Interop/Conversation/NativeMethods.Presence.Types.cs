using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop.Conversation;

[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativePresenceUser
{
    internal byte* Name, Avatar;
    internal int Palette;
    internal uint Rgba;
    internal int Idle;
}
