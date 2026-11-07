using System.ComponentModel;
using System.IO.Pipes;
using System.Runtime.InteropServices;
using System.Text.Json;
using Fpas.Debugging.Native;

namespace Fpas.Debugging.Hosting;

internal static class TargetBroker
{
    internal static async Task<int> Run(string name)
    {
        if (!name.StartsWith("fpas-debug-", StringComparison.Ordinal) || name.Length > 96) return 2;
        using var lifetime = new CancellationTokenSource();
        using var startup = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        await using var pipe = new NamedPipeClientStream(".", name, PipeDirection.InOut, PipeOptions.Asynchronous | PipeOptions.CurrentUserOnly);
        await pipe.ConnectAsync(startup.Token);
        await using var channel = new WorkerChannel(pipe);
        var request = await channel.Read(startup.Token) ?? throw new IOException("No target broker request.");
        var target = request.Payload?.Deserialize(WorkerJson.Default.BrokerLaunch) ?? throw new IOException("Invalid target broker request.");
        if (target.Terminal is not null)
            return await TerminalTarget.Run(target, request.Id, channel, startup.Token);
        if (!OperatingSystem.IsLinux()) return 2;
        uint id = 0; nint resume = 0;
        Task<BrokerExit>? exit = null;
        try
        {
            NativeObject.Check(DbgShim.Instance.CreateProcess(DbgShim.CommandLine(target.Dotnet, [target.Program, .. target.Arguments ?? []]), 1, 0, target.Cwd, out id, out resume));
            await channel.Send(new(request.Id, "reply", "create", JsonSerializer.SerializeToElement(new BrokerTarget((int)id), WorkerJson.Default.BrokerTarget)), startup.Token);
            exit = Reap(id);
            var commands = Commands();
            if (await Task.WhenAny(exit, commands) == commands && !exit.IsCompleted)
            {
                _ = Kill((int)id, 9);
                await commands;
            }
            var result = await exit;
            await channel.Send(new(0, "event", "exit", JsonSerializer.SerializeToElement(result, WorkerJson.Default.BrokerExit)), CancellationToken.None);
            lifetime.Cancel();
            try { await commands; } catch (OperationCanceledException) when (lifetime.IsCancellationRequested) { }
            return 0;

            async Task Commands()
            {
                while (await channel.Read(lifetime.Token) is { } command)
                {
                    if (command.Kind != "request" || command.Operation != "resume" || resume == 0) throw new IOException("Invalid target broker command.");
                    NativeObject.Check(DbgShim.Instance.Resume(resume));
                    NativeObject.Check(DbgShim.Instance.CloseResume(resume)); resume = 0;
                    await channel.Send(new(command.Id, "reply", "resume"), lifetime.Token);
                }
            }
        }
        finally
        {
            lifetime.Cancel();
            if (id != 0 && exit?.IsCompleted != true)
            {
                _ = Kill((int)id, 9);
                await (exit ?? Reap(id));
            }
            if (resume != 0) NativeObject.Check(DbgShim.Instance.CloseResume(resume));
        }
    }

    private static async Task<BrokerExit> Reap(uint id)
    {
        while (true)
        {
            var result = WaitPid((int)id, out var status, 1);
            if (result == (int)id)
            {
                var signal = status & 0x7f;
                return signal == 0 ? new((status >> 8) & 0xff) : new(128 + signal, signal);
            }
            if (result < 0 && Marshal.GetLastPInvokeError() != 4) throw new Win32Exception(Marshal.GetLastPInvokeError(), "Unable to collect the target exit status.");
            await Task.Delay(10);
        }
    }

    [DllImport("libc", EntryPoint = "waitpid", SetLastError = true)] private static extern int WaitPid(int pid, out int status, int options);
    [DllImport("libc", EntryPoint = "kill", SetLastError = true)] private static extern int Kill(int pid, int signal);
}
