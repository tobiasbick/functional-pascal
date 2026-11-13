using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeDevTestItem
{
    internal byte* Name;
    internal int Depth;
    internal int Status;
    internal float Duration;
    internal byte* Message;
}

[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeDevStackFrame
{
    internal byte* FunctionName;
    internal byte* File;
    internal int Line;
    internal int Column;
    internal byte External;
    internal byte** Source;
    internal int SourceCount;
    internal int SourceFirst;
}

[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeDevService
{
    internal byte* Name;
    internal float* Days;
    internal int DaysCount;
}

[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeDevSeries
{
    internal byte* Name;
    internal float* Counts;
    internal int Tone;
}

[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeDevCoverageFile
{
    internal byte* Path;
    internal int Lines;
    internal int LinesHit;
    internal int Branches;
    internal int BranchesHit;
    internal int Functions;
    internal int FunctionsHit;
}

[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeDevRevision
{
    internal byte* Hash;
    internal byte* Author;
    internal byte* Summary;
    internal long Time;
}

[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeDevParameter
{
    internal byte* Name;
    internal byte* Location;
    internal byte* Type;
    internal byte Required;
    internal byte* Description;
}

[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeDevEndpoint
{
    internal byte* Method;
    internal byte* Path;
    internal byte* Summary;
    internal byte* Tag;
    internal byte Deprecated;
    internal NativeDevParameter* Parameters;
    internal int ParametersCount;
}
