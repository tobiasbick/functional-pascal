using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Debugging.Native;

internal sealed unsafe partial class ManagedCallbacks
{
    private static void FillSecond(nint* table)
    {
        table[0] = (nint)(delegate* unmanaged[Stdcall]<nint, nint, nint, nint, nint, uint, int>)&RemapOpportunity;
        table[1] = (nint)(delegate* unmanaged[Stdcall]<nint, nint, uint, char*, int>)&CreateConnection;
        table[2] = table[3] = (nint)(delegate* unmanaged[Stdcall]<nint, nint, uint, int>)&Connection;
        table[4] = (nint)(delegate* unmanaged[Stdcall]<nint, nint, nint, nint, uint, int, uint, int>)&Exception2;
        table[5] = (nint)(delegate* unmanaged[Stdcall]<nint, nint, nint, int, uint, int>)&ExceptionUnwind;
        table[6] = table[7] = (nint)(delegate* unmanaged[Stdcall]<nint, nint, nint, nint, int>)&ContinueFour;
    }

    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvStdcall)])]
    private static int RemapOpportunity(nint self, nint domain, nint thread, nint oldFunction, nint newFunction, uint offset) => Invoke(self, domain);
    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvStdcall)])]
    private static int CreateConnection(nint self, nint process, uint id, char* name) => Invoke(self, process);
    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvStdcall)])]
    private static int Connection(nint self, nint process, uint id) => Invoke(self, process);
    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvStdcall)])]
    private static int Exception2(nint self, nint domain, nint thread, nint frame, uint offset, int kind, uint flags)
        => Invoke(self, domain, session => session.ExceptionRaised(domain, thread, kind), resume: false);
    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvStdcall)])]
    private static int ExceptionUnwind(nint self, nint domain, nint thread, int kind, uint flags) => Invoke(self, domain);
}
