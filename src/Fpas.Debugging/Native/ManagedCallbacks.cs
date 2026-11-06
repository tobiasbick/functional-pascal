using System.Collections;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Debugging.Native;

internal sealed unsafe partial class ManagedCallbacks(CoreClrSession owner)
{
    private readonly CoreClrSession session = owner;
    internal static readonly Guid CallbackId = new("3D6F5F60-7538-11D3-8D5B-00104B35E7EF");
    private static readonly CallbackWrappers wrappers = new();
    internal NativeObject Export()
    {
        using var unknown = new NativeObject(wrappers.GetOrCreateComInterfaceForObject(this, CreateComInterfaceFlags.None));
        return unknown.Query(CallbackId);
    }

    private static CoreClrSession Owner(nint self) => ComWrappers.ComInterfaceDispatch.GetInstance<ManagedCallbacks>((ComWrappers.ComInterfaceDispatch*)self).session;

    // No managed exception may cross a native callback. The session reports failures to its owner.
    private static int Invoke(nint self, nint controller, Action<CoreClrSession>? action = null, bool resume = true)
    {
        var session = Owner(self);
        try
        {
            action?.Invoke(session);
            if (resume) session.ContinueCallback(controller);
        }
        catch (Exception exception) { session.NativeFailure(exception); }
        return 0;
    }

    private sealed class CallbackWrappers : ComWrappers
    {
        private static readonly ComInterfaceEntry* entries = CreateEntries();
        private static ComInterfaceEntry* CreateEntries()
        {
            GetIUnknownImpl(out var query, out var addRef, out var release);
            var tables = (nint*)RuntimeHelpers.AllocateTypeAssociatedMemory(typeof(CallbackWrappers), (29 + 11) * nint.Size);
            tables[0] = tables[29] = query; tables[1] = tables[30] = addRef; tables[2] = tables[31] = release;
            FillFirst(tables + 3); FillSecond(tables + 32);
            var result = (ComInterfaceEntry*)RuntimeHelpers.AllocateTypeAssociatedMemory(typeof(CallbackWrappers), 2 * sizeof(ComInterfaceEntry));
            result[0] = new() { IID = CallbackId, Vtable = (nint)tables };
            result[1] = new() { IID = new("250E5EEA-DB5C-4C76-B6F3-8C46F12E3203"), Vtable = (nint)(tables + 29) };
            return result;
        }
        protected override ComInterfaceEntry* ComputeVtables(object obj, CreateComInterfaceFlags flags, out int count)
        {
            if (obj is not ManagedCallbacks) throw new ArgumentException("Unsupported callback object.");
            count = 2; return entries;
        }
        protected override object CreateObject(nint externalComObject, CreateObjectFlags flags) => throw new NotSupportedException();
        protected override void ReleaseObjects(IEnumerable objects) => throw new NotSupportedException();
    }

    private static void FillFirst(nint* table)
    {
        table[0] = (nint)(delegate* unmanaged[Stdcall]<nint, nint, nint, nint, int>)&Breakpoint;
        table[1] = (nint)(delegate* unmanaged[Stdcall]<nint, nint, nint, nint, int, int>)&StepComplete;
        table[2] = (nint)(delegate* unmanaged[Stdcall]<nint, nint, nint, int>)&Break;
        table[3] = (nint)(delegate* unmanaged[Stdcall]<nint, nint, nint, int, int>)&Exception;
        table[4] = table[5] = (nint)(delegate* unmanaged[Stdcall]<nint, nint, nint, nint, int>)&ContinueFour;
        table[6] = (nint)(delegate* unmanaged[Stdcall]<nint, nint, int>)&ContinueTwo;
        table[7] = (nint)(delegate* unmanaged[Stdcall]<nint, nint, int>)&ExitProcess;
        table[8] = table[9] = (nint)(delegate* unmanaged[Stdcall]<nint, nint, nint, int>)&ContinueThree;
        table[10] = (nint)(delegate* unmanaged[Stdcall]<nint, nint, nint, int>)&LoadModule;
        table[11] = (nint)(delegate* unmanaged[Stdcall]<nint, nint, nint, int>)&UnloadModule;
        table[12] = table[13] = (nint)(delegate* unmanaged[Stdcall]<nint, nint, nint, int>)&ContinueThree;
        table[14] = (nint)(delegate* unmanaged[Stdcall]<nint, nint, int, uint, int>)&DebuggerError;
        table[15] = (nint)(delegate* unmanaged[Stdcall]<nint, nint, nint, int, char*, char*, int>)&LogMessage;
        table[16] = (nint)(delegate* unmanaged[Stdcall]<nint, nint, nint, int, uint, char*, char*, int>)&LogSwitch;
        table[17] = table[18] = table[19] = table[20] = (nint)(delegate* unmanaged[Stdcall]<nint, nint, nint, int>)&ContinueThree;
        table[21] = (nint)(delegate* unmanaged[Stdcall]<nint, nint, int>)&ContinueTwo;
        table[22] = (nint)(delegate* unmanaged[Stdcall]<nint, nint, nint, int>)&ContinueThree;
        table[23] = (nint)(delegate* unmanaged[Stdcall]<nint, nint, nint, nint, int>)&ContinueFour;
        table[24] = (nint)(delegate* unmanaged[Stdcall]<nint, nint, nint, nint, int, int>)&Remap;
        table[25] = (nint)(delegate* unmanaged[Stdcall]<nint, nint, nint, nint, uint, int>)&BreakpointError;
    }

    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvStdcall)])]
    private static int Breakpoint(nint self, nint domain, nint thread, nint breakpoint)
        => Invoke(self, domain, session => session.BreakpointHit(domain, thread, breakpoint), resume: false);
    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvStdcall)])]
    private static int StepComplete(nint self, nint domain, nint thread, nint stepper, int reason)
        => Invoke(self, domain, session => session.StepCompleted(domain, thread), resume: false);
    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvStdcall)])]
    private static int Break(nint self, nint domain, nint thread)
        => Invoke(self, domain, session => session.Stopped("pause", thread), resume: false);
    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvStdcall)])]
    private static int Exception(nint self, nint domain, nint thread, int unhandled)
        => Invoke(self, domain);
    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvStdcall)])]
    private static int ExitProcess(nint self, nint process)
        => Invoke(self, process, session => session.Exited(), resume: false);
    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvStdcall)])]
    private static int LoadModule(nint self, nint domain, nint module)
        => Invoke(self, domain, session => session.ModuleLoaded(module));
    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvStdcall)])]
    private static int UnloadModule(nint self, nint domain, nint module)
        => Invoke(self, domain, session => session.ModuleUnloaded(module));
    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvStdcall)])]
    private static int ContinueTwo(nint self, nint controller) => Invoke(self, controller);
    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvStdcall)])]
    private static int ContinueThree(nint self, nint controller, nint unused) => Invoke(self, controller);
    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvStdcall)])]
    private static int ContinueFour(nint self, nint controller, nint unused1, nint unused2) => Invoke(self, controller);
    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvStdcall)])]
    private static int DebuggerError(nint self, nint process, int result, uint code)
        => Invoke(self, process, _ => NativeObject.Check(result), resume: false);
    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvStdcall)])]
    private static int LogMessage(nint self, nint domain, nint thread, int level, char* name, char* message) => Invoke(self, domain);
    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvStdcall)])]
    private static int LogSwitch(nint self, nint domain, nint thread, int level, uint reason, char* name, char* parent) => Invoke(self, domain);
    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvStdcall)])]
    private static int Remap(nint self, nint domain, nint thread, nint function, int accurate) => Invoke(self, domain);
    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvStdcall)])]
    private static int BreakpointError(nint self, nint domain, nint thread, nint breakpoint, uint error)
        => Invoke(self, domain, session => session.BreakpointRejected(breakpoint, error));
}
