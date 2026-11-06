using System.ComponentModel;
using System.Diagnostics;
using System.Runtime.InteropServices;
using Microsoft.Win32.SafeHandles;

namespace Fpas.Debugging.Native;

internal static class CreatedProcess
{
    // The PID is the receipt from a successful suspended native creation, not a discovered process.
    // Capture runs before the resume handle is closed or the target is allowed to execute.
    internal static Process Capture(uint id)
    {
        Process? process = null;
        try
        {
            process = Process.GetProcessById(checked((int)id));
            _ = process.SafeHandle;
            return process;
        }
        catch (Exception failure)
        {
            try { AbortSuspended(id); }
            catch (Exception cleanup) { throw new AggregateException("Suspended target tracking and cleanup both failed.", failure, cleanup); }
            finally { process?.Dispose(); }
            throw;
        }
    }

    private static void AbortSuspended(uint id)
    {
        if (OperatingSystem.IsWindows())
        {
            using var handle = OpenProcess(0x100001, false, id); // SYNCHRONIZE | PROCESS_TERMINATE
            if (handle.IsInvalid)
            {
                var error = Marshal.GetLastPInvokeError();
                if (error == 87) return; // The created process has already exited.
                throw new Win32Exception(error, "Unable to recover the suspended target.");
            }
            if (!TerminateProcess(handle, 1)) throw new Win32Exception(Marshal.GetLastPInvokeError());
            if (WaitForSingleObject(handle, 5000) != 0) throw new IOException("The suspended target did not exit during cleanup.");
        }
        else
        {
            if (Kill((int)id, 9) != 0 && Marshal.GetLastPInvokeError() != 3) throw new Win32Exception(Marshal.GetLastPInvokeError());
            var deadline = Stopwatch.StartNew();
            while (WaitPid((int)id, out _, 1) == 0)
            {
                if (deadline.Elapsed > TimeSpan.FromSeconds(5)) throw new IOException("The suspended target did not exit during cleanup.");
                Thread.Sleep(10);
            }
        }
    }

    internal static bool ReapExitedChild(Process process)
    {
        if (!OperatingSystem.IsLinux()) return process.HasExited;
        var result = WaitPid(process.Id, out _, 1);
        if (result == process.Id) return true;
        if (result == 0) return false;
        var error = Marshal.GetLastPInvokeError();
        if (error == 4) return false;
        if (error == 10) return !Directory.Exists($"/proc/{process.Id}");
        throw new Win32Exception(error, "Unable to reap the created target.");
    }

    internal static async Task WaitForUnattachedExit(Process process, CancellationToken token)
    {
        while (!ReapExitedChild(process)) await Task.Delay(10, token);
    }

    [DllImport("kernel32.dll", SetLastError = true)] private static extern SafeProcessHandle OpenProcess(uint access, [MarshalAs(UnmanagedType.Bool)] bool inherit, uint id);
    [DllImport("kernel32.dll", SetLastError = true)] [return: MarshalAs(UnmanagedType.Bool)] private static extern bool TerminateProcess(SafeProcessHandle process, uint exit);
    [DllImport("kernel32.dll", SetLastError = true)] private static extern uint WaitForSingleObject(SafeProcessHandle process, uint milliseconds);
    [DllImport("libc", EntryPoint = "kill", SetLastError = true)] private static extern int Kill(int pid, int signal);
    [DllImport("libc", EntryPoint = "waitpid", SetLastError = true)] private static extern int WaitPid(int pid, out int status, int options);
}
