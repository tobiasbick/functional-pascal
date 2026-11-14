using Fpas.Ui.Gui.Cui.Interop.Ai;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_agent_plan", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int AgentPlan(nint ui, string? id, NativePlanStep* steps, int count, string? title, int flags, int* parts);
}
