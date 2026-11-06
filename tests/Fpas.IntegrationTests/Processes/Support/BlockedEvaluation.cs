using Fpas.Tests.Support;
using System.Diagnostics;
using System.IO.Pipes;
using System.Runtime.InteropServices;
using System.Xml.Linq;
using Microsoft.Win32.SafeHandles;

namespace Fpas.IntegrationTests.Processes.Support;

// Block real project evaluation on an input owned by the test, without replacing the host.
internal sealed class BlockedEvaluation : IDisposable
{
    private readonly NamedPipeServerStream? pipe;
    private readonly string directory;

    internal BlockedEvaluation(TemporaryDirectory project)
    {
        directory = project.File("Game");
        if (OperatingSystem.IsWindows())
        {
            var name = "fpas-evaluation-" + Guid.NewGuid().ToString("N");
            pipe = new NamedPipeServerStream(name, PipeDirection.Out, 1, PipeTransmissionMode.Byte, PipeOptions.Asynchronous);
            new XDocument(new XElement("Project", new XElement("PropertyGroup", new XElement("BlockedInput",
                "$([System.IO.File]::ReadAllText('\\\\.\\pipe\\" + name + "'))")))).Save(project.File("Game/Game.fpasproj"));
        }
        else
        {
            Assert.True(OperatingSystem.IsLinux(), "Blocked evaluation requires Windows or Linux.");
            var configuration = project.File("NuGet.Config");
            File.Delete(configuration);
            Assert.Equal(0, MakeFifo(configuration, 384));
        }
    }

    internal async Task<Process> ConnectedProcess(int parent, CancellationToken token)
    {
        if (pipe is not null)
        {
            await pipe.WaitForConnectionAsync(token);
            Assert.True(GetNamedPipeClientProcessId(pipe.SafePipeHandle, out var pid));
            var process = Process.GetProcessById(checked((int)pid));
            try { Assert.Equal(parent, ProcessIdentity.Parent(process.Id)); return process; }
            catch { process.Dispose(); throw; }
        }
        while (true)
        {
            foreach (var path in Directory.EnumerateDirectories("/proc"))
            {
                if (!int.TryParse(Path.GetFileName(path), out var pid)) continue;
                try
                {
                    if (ProcessIdentity.Parent(pid) == parent && Directory.ResolveLinkTarget(Path.Combine(path, "cwd"), true)?.FullName == directory)
                        return Process.GetProcessById(pid);
                }
                catch (Exception error) when (error is IOException or UnauthorizedAccessException or ArgumentException) { }
            }
            await Task.Delay(10, token);
        }
    }

    public void Dispose() => pipe?.Dispose();

    [DllImport("kernel32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool GetNamedPipeClientProcessId(SafePipeHandle pipe, out uint processId);

    [DllImport("libc", EntryPoint = "mkfifo", SetLastError = true)]
    private static extern int MakeFifo(string path, uint mode);
}
