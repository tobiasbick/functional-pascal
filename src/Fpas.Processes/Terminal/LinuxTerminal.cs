using Microsoft.Win32.SafeHandles;
using System.ComponentModel;
using System.Runtime.InteropServices;

namespace Fpas.Processes.Terminal;

internal sealed class LinuxTerminal : PseudoTerminal
{
    private readonly SafeFileHandle master;
    private readonly FileStream input;
    private readonly FileStream output;
    private readonly Task<int> completion;
    private Task? termination;
    private bool suspended;
    public override Stream Input => input;
    public override Stream Output => output;
    public override int Id { get; }
    public override Task<int> Completion => completion;

    private LinuxTerminal(int pid, SafeFileHandle master, SafeFileHandle duplicate, bool suspended)
    {
        Id = pid; this.master = master; input = new(master, FileAccess.Write); output = new(duplicate, FileAccess.Read);
        this.suspended = suspended;
        completion = Task.Run(() =>
        {
            int result; int status;
            do { result = WaitPid(pid, out status, 0); } while (result < 0 && Marshal.GetLastPInvokeError() == 4);
            if (result != pid) throw new Win32Exception(Marshal.GetLastPInvokeError());
            return (status & 0x7f) == 0 ? (status >> 8) & 0xff : 128 + (status & 0x7f);
        });
    }

    internal static PseudoTerminal Start(string path, IReadOnlyList<string> arguments, string cwd, int columns, int rows, bool suspended)
    {
        if (Prctl(36, 1, 0, 0, 0) != 0) throw new Win32Exception(Marshal.GetLastPInvokeError());
        var fd = PosixOpenPt(0x102); // O_RDWR | O_NOCTTY; the slave becomes the child's controlling terminal.
        if (fd < 0) throw new Win32Exception(Marshal.GetLastPInvokeError());
        var handle = new SafeFileHandle(fd, true);
        IntPtr actions = IntPtr.Zero, attributes = IntPtr.Zero, argv = IntPtr.Zero, environment = IntPtr.Zero;
        var strings = new List<IntPtr>(); var actionsReady = false; var attributesReady = false;
        try
        {
            Check(GrantPt(fd)); Check(UnlockPt(fd));
            var slave = Marshal.PtrToStringUTF8(PtsName(fd)) ?? throw new IOException("No PTY slave name.");
            var size = new WindowSize { Columns = (ushort)columns, Rows = (ushort)rows }; Check(Ioctl(fd, 0x5414, ref size));
            actions = Marshal.AllocHGlobal(1024); attributes = Marshal.AllocHGlobal(1024);
            Check(PosixSpawnFileActionsInit(actions)); actionsReady = true;
            Check(PosixSpawnAttrInit(attributes)); attributesReady = true;
            Check(PosixSpawnAttrSetFlags(attributes, 0x80)); // POSIX_SPAWN_SETSID (glibc).
            Check(PosixSpawnFileActionsAddOpen(actions, 0, slave, 2, 0));
            Check(PosixSpawnFileActionsAddDup2(actions, 0, 1)); Check(PosixSpawnFileActionsAddDup2(actions, 0, 2));
            Check(PosixSpawnFileActionsAddClose(actions, fd)); Check(PosixSpawnFileActionsAddChdir(actions, cwd));
            IntPtr Vector(IEnumerable<string> values)
            {
                var items = values.Select(value => { var pointer = Marshal.StringToCoTaskMemUTF8(value); strings.Add(pointer); return pointer; }).ToArray();
                var vector = Marshal.AllocHGlobal((items.Length + 1) * IntPtr.Size);
                for (var index = 0; index < items.Length; index++) Marshal.WriteIntPtr(vector, index * IntPtr.Size, items[index]);
                Marshal.WriteIntPtr(vector, items.Length * IntPtr.Size, IntPtr.Zero); return vector;
            }
            // The shell stops before exec, preserving the PID registered with the CLR startup shim.
            argv = Vector(suspended
                ? new[] { "/bin/sh", "-c", "kill -STOP \"$$\" && exec \"$@\"", "fpas-terminal", path }.Concat(arguments)
                : new[] { path }.Concat(arguments));
            environment = Vector(Environment.GetEnvironmentVariables().Cast<System.Collections.DictionaryEntry>().Select(item => item.Key + "=" + item.Value));
            Check(PosixSpawn(out var pid, suspended ? "/bin/sh" : path, actions, attributes, argv, environment));
            var duplicate = Dup(fd);
            if (duplicate < 0) { Kill(-pid, 9); WaitPid(pid, out _, 0); throw new Win32Exception(Marshal.GetLastPInvokeError()); }
            var output = new SafeFileHandle(duplicate, true);
            try
            {
                if (suspended) WaitForStartupStop(pid);
                return new LinuxTerminal(pid, handle, output, suspended);
            }
            catch { output.Dispose(); throw; }
        }
        catch { handle.Dispose(); throw; }
        finally
        {
            if (actionsReady) PosixSpawnFileActionsDestroy(actions); if (attributesReady) PosixSpawnAttrDestroy(attributes);
            if (actions != IntPtr.Zero) Marshal.FreeHGlobal(actions); if (attributes != IntPtr.Zero) Marshal.FreeHGlobal(attributes);
            if (argv != IntPtr.Zero) Marshal.FreeHGlobal(argv); if (environment != IntPtr.Zero) Marshal.FreeHGlobal(environment);
            foreach (var pointer in strings) Marshal.FreeCoTaskMem(pointer);
        }
    }
    private static void WaitForStartupStop(int pid)
    {
        var started = System.Diagnostics.Stopwatch.StartNew();
        while (started.Elapsed < TimeSpan.FromSeconds(5))
        {
            var result = WaitPid(pid, out var status, 3); // WNOHANG | WUNTRACED
            if (result == pid)
            {
                if ((status & 0xff) == 0x7f) return;
                throw new IOException("The terminal startup gate exited before stopping.");
            }
            if (result < 0 && Marshal.GetLastPInvokeError() != 4)
                throw new Win32Exception(Marshal.GetLastPInvokeError());
            Thread.Sleep(5);
        }
        Kill(-pid, 9); WaitPid(pid, out _, 0);
        throw new TimeoutException("The terminal startup gate did not stop.");
    }

    internal override void Resume()
    {
        if (!suspended) throw new InvalidOperationException("The terminal target has already been released.");
        Check(Kill(Id, 18)); // SIGCONT
        suspended = false;
    }
    public override void Resize(int columns, int rows)
    { Dimensions(columns, rows); var size = new WindowSize { Columns = (ushort)columns, Rows = (ushort)rows }; Check(Ioctl((int)master.DangerousGetHandle(), 0x5414, ref size)); }
    public override Task TerminateAsync() => termination ??= Terminate();
    private async Task Terminate()
    {
        if (Kill(-Id, 9) != 0 && Marshal.GetLastPInvokeError() != 3) throw new Win32Exception(Marshal.GetLastPInvokeError());
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(5));
        await completion.WaitAsync(deadline.Token);
        while (true)
        {
            var child = WaitPid(-Id, out _, 1); if (child > 0) continue;
            if (child < 0 && Marshal.GetLastPInvokeError() == 10) break;
            deadline.Token.ThrowIfCancellationRequested(); await Task.Delay(10, deadline.Token);
        }
    }
    public override async ValueTask DisposeAsync()
    { try { await TerminateAsync(); } finally { input.Dispose(); output.Dispose(); } }

    private static void Check(int error) { if (error != 0) throw new Win32Exception(error > 0 ? error : Marshal.GetLastPInvokeError()); }
    [StructLayout(LayoutKind.Sequential)] private struct WindowSize { internal ushort Rows, Columns, X, Y; }
    [DllImport("libc", EntryPoint = "posix_openpt", SetLastError = true)] private static extern int PosixOpenPt(int flags);
    [DllImport("libc", EntryPoint = "grantpt", SetLastError = true)] private static extern int GrantPt(int fd);
    [DllImport("libc", EntryPoint = "unlockpt", SetLastError = true)] private static extern int UnlockPt(int fd);
    [DllImport("libc", EntryPoint = "ptsname")] private static extern IntPtr PtsName(int fd);
    [DllImport("libc", EntryPoint = "dup", SetLastError = true)] private static extern int Dup(int fd);
    [DllImport("libc", EntryPoint = "ioctl", SetLastError = true)] private static extern int Ioctl(int fd, nuint request, ref WindowSize size);
    [DllImport("libc", EntryPoint = "prctl", SetLastError = true)] private static extern int Prctl(int option, nuint a2, nuint a3, nuint a4, nuint a5);
    [DllImport("libc", EntryPoint = "kill", SetLastError = true)] private static extern int Kill(int pid, int signal);
    [DllImport("libc", EntryPoint = "waitpid", SetLastError = true)] private static extern int WaitPid(int pid, out int status, int flags);
    [DllImport("libc", EntryPoint = "posix_spawn")] private static extern int PosixSpawn(out int pid, [MarshalAs(UnmanagedType.LPUTF8Str)] string path, IntPtr actions, IntPtr attributes, IntPtr argv, IntPtr env);
    [DllImport("libc", EntryPoint = "posix_spawn_file_actions_init")] private static extern int PosixSpawnFileActionsInit(IntPtr actions);
    [DllImport("libc", EntryPoint = "posix_spawn_file_actions_destroy")] private static extern int PosixSpawnFileActionsDestroy(IntPtr actions);
    [DllImport("libc", EntryPoint = "posix_spawn_file_actions_addopen")] private static extern int PosixSpawnFileActionsAddOpen(IntPtr actions, int fd, [MarshalAs(UnmanagedType.LPUTF8Str)] string path, int flags, uint mode);
    [DllImport("libc", EntryPoint = "posix_spawn_file_actions_adddup2")] private static extern int PosixSpawnFileActionsAddDup2(IntPtr actions, int oldFd, int newFd);
    [DllImport("libc", EntryPoint = "posix_spawn_file_actions_addclose")] private static extern int PosixSpawnFileActionsAddClose(IntPtr actions, int fd);
    [DllImport("libc", EntryPoint = "posix_spawn_file_actions_addchdir_np")] private static extern int PosixSpawnFileActionsAddChdir(IntPtr actions, [MarshalAs(UnmanagedType.LPUTF8Str)] string path);
    [DllImport("libc", EntryPoint = "posix_spawnattr_init")] private static extern int PosixSpawnAttrInit(IntPtr attributes);
    [DllImport("libc", EntryPoint = "posix_spawnattr_destroy")] private static extern int PosixSpawnAttrDestroy(IntPtr attributes);
    [DllImport("libc", EntryPoint = "posix_spawnattr_setflags")] private static extern int PosixSpawnAttrSetFlags(IntPtr attributes, short flags);
}
