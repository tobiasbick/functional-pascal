// [Documentation](../../../../docs/pascal/libraries/processes.md)
using System.Diagnostics;

namespace Fpas.IntegrationTests.Processes.Support;

// Shared assertions that processes owned by a tested command have ended.
internal static class ProcessExit
{
    // Opens a handle to the process so its exit stays observable after the id is released.
    internal static Process Track(int pid)
    {
        var process = Process.GetProcessById(pid);
        _ = process.SafeHandle;
        return process;
    }

    // Asserts that a tracked process exited and, on Linux, was also reaped.
    internal static void AssertGone(Process process)
    {
        if (OperatingSystem.IsLinux()) Assert.False(Directory.Exists($"/proc/{process.Id}"));
        else Assert.True(process.HasExited);
    }

    // Asserts that the process with this id has exited or no longer exists.
    internal static void AssertExited(int pid)
    {
        try
        {
            using var process = Process.GetProcessById(pid);
            Assert.True(process.HasExited, $"Process {pid} survived its owner.");
        }
        catch (ArgumentException) { }
    }
}
