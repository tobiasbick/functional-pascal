using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop.Developer;

[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeDevDiagnostic
{
    internal int Severity;
    internal byte* Message;
    internal byte* File;
    internal int Line;
    internal int Column;
    internal byte* Source;
    internal byte* Code;
}

[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeDevJob
{
    internal byte* Name;
    internal int Stage;
    internal int Status;
    internal float Duration;
    internal byte AllowFailure;
}

[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeDevBinding
{
    internal int Modifiers;
    internal int Key;
    internal byte* Action;
    internal int Category;
}

[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeDevCommit
{
    internal byte* Hash;
    internal byte* Message;
    internal byte* Author;
    internal byte* Date;
    internal byte* Refs;
    internal int Parent;
    internal int MergeParent;
}

[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeDevGraphNode
{
    internal byte* Label;
    internal byte* Detail;
    internal int Tone;
}

[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeDevEdge
{
    internal int From;
    internal int To;
}

[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeDevRequest
{
    internal byte* Name;
    internal byte* Method;
    internal int Status;
    internal byte* Type;
    internal double Size;
    internal double Start;
    internal float Queued;
    internal float Dns;
    internal float Connect;
    internal float Tls;
    internal float Wait;
    internal float Download;
}

[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeDevTrack
{
    internal byte* Name;
    internal byte Collapsed;
}

[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeDevSpan
{
    internal byte* Name;
    internal double Start;
    internal double End;
    internal int Track;
    internal int Depth;
    internal uint ColorKey;
}

[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeDevMarker
{
    internal byte* Name;
    internal double Time;
    internal int Tone;
}
