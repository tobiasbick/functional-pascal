using System.ComponentModel;
using System.Diagnostics;
using System.Runtime.InteropServices;
using Microsoft.Win32.SafeHandles;

namespace Fpas.IntegrationTests.Debugging;

// Suspend only the real launch-owned worker to prove supervisor timeout and containment.
internal sealed class WorkerSuspension : IDisposable
{
    private readonly Process process;
    private readonly List<SafeWaitHandle> threads = [];

    internal WorkerSuspension(Process process)
    {
        this.process = process;
        try
        {
            if (OperatingSystem.IsLinux())
            {
                if (Kill(process.Id, 19) != 0) throw new Win32Exception(Marshal.GetLastPInvokeError());
            }
            else
            {
                foreach (ProcessThread thread in process.Threads)
                {
                    using (thread)
                    {
                        var handle = OpenThread(2, false, (uint)thread.Id);
                        if (handle.IsInvalid) { handle.Dispose(); throw new Win32Exception(Marshal.GetLastPInvokeError()); }
                        if (SuspendThread(handle) == uint.MaxValue) { handle.Dispose(); throw new Win32Exception(Marshal.GetLastPInvokeError()); }
                        threads.Add(handle);
                    }
                }
            }
        }
        catch { Dispose(); throw; }
    }

    public void Dispose()
    {
        if (OperatingSystem.IsLinux())
        {
            if (Directory.Exists($"/proc/{process.Id}")) _ = Kill(process.Id, 18);
        }
        else
        {
            foreach (var thread in threads)
            {
                if (!process.HasExited) _ = ResumeThread(thread);
                thread.Dispose();
            }
            threads.Clear();
        }
    }

    [DllImport("libc", EntryPoint = "kill", SetLastError = true)] private static extern int Kill(int process, int signal);
    [DllImport("kernel32.dll", SetLastError = true)] private static extern SafeWaitHandle OpenThread(uint access, [MarshalAs(UnmanagedType.Bool)] bool inherit, uint thread);
    [DllImport("kernel32.dll", SetLastError = true)] private static extern uint SuspendThread(SafeWaitHandle thread);
    [DllImport("kernel32.dll", SetLastError = true)] private static extern uint ResumeThread(SafeWaitHandle thread);
}
