using System.Diagnostics;
using System.IO.Pipes;
using System.Text.Json;
using Fpas.Debugging.Native;

namespace Fpas.Debugging.Hosting;

// A Linux target needs a parent outside DBI, otherwise DBI consumes its wait status.
// The broker remains in the already-owned worker session and never creates a new process group.
internal sealed class LaunchBroker : IAsyncDisposable
{
    private readonly NamedPipeServerStream pipe;
    private readonly CancellationTokenSource lifetime = new();
    private readonly TaskCompletionSource<BrokerExit> exited = new(TaskCreationOptions.RunContinuationsAsynchronously);
    private readonly TaskCompletionSource resumed = new(TaskCreationOptions.RunContinuationsAsynchronously);
    private WorkerChannel? channel;
    private Process? process;
    private Task reader = Task.CompletedTask;
    private bool disposing;
    internal int TargetId { get; private set; }

    private LaunchBroker(string name) => pipe = new(name, PipeDirection.InOut, 1, PipeTransmissionMode.Byte, PipeOptions.Asynchronous | PipeOptions.CurrentUserOnly);

    internal static async Task<LaunchBroker> Start(WorkerLauncher launcher, BrokerLaunch target, CancellationToken token)
    {
        var name = "fpas-debug-" + Guid.NewGuid().ToString("N");
        var broker = new LaunchBroker(name);
        try
        {
            var start = new ProcessStartInfo(launcher.Executable) { UseShellExecute = false, CreateNoWindow = true, WorkingDirectory = target.Cwd };
            foreach (var argument in launcher.BrokerArguments(name)) start.ArgumentList.Add(argument);
            broker.process = Process.Start(start) ?? throw new IOException("Unable to start the target launch broker.");
            using var deadline = CancellationTokenSource.CreateLinkedTokenSource(token);
            deadline.CancelAfter(TimeSpan.FromSeconds(30));
            var connect = broker.pipe.WaitForConnectionAsync(deadline.Token);
            var exit = broker.process.WaitForExitAsync(deadline.Token);
            if (await Task.WhenAny(connect, exit).WaitAsync(deadline.Token) != connect && !connect.IsCompleted)
                throw new IOException("The target launch broker exited before connecting.");
            await connect;
            broker.channel = new(broker.pipe);
            await broker.channel.Send(new(1, "request", "create", JsonSerializer.SerializeToElement(target, WorkerJson.Default.BrokerLaunch)), deadline.Token);
            var response = await broker.channel.Read(deadline.Token) ?? throw new IOException("The target launch broker disconnected.");
            if (response.Error is { } failure) throw new IOException(failure.Message);
            broker.TargetId = response.Payload?.Deserialize(WorkerJson.Default.BrokerTarget)?.ProcessId ?? throw new IOException("The broker returned no target identity.");
            broker.reader = broker.Read();
            return broker;
        }
        catch { await broker.DisposeAsync(); throw; }
    }

    internal async Task Resume(CancellationToken token)
    {
        await channel!.Send(new(2, "request", "resume"), token);
        await resumed.Task.WaitAsync(token);
    }

    internal async Task<int> ExitCode(CancellationToken token) => (await exited.Task.WaitAsync(token)).Code;
    internal Task<BrokerExit> ExitStatus(CancellationToken token) => exited.Task.WaitAsync(token);

    private async Task Read()
    {
        try
        {
            while (await channel!.Read(lifetime.Token) is { } message)
            {
                if (message.Error is { } error) throw new IOException(error.Message);
                if (message.Kind == "reply" && message.Id == 2) resumed.TrySetResult();
                else if (message.Kind == "event" && message.Operation == "exit")
                    exited.TrySetResult(message.Payload?.Deserialize(WorkerJson.Default.BrokerExit) ?? throw new IOException("Missing target exit status."));
                else throw new IOException("Unexpected target broker message.");
            }
            if (!disposing && !exited.Task.IsCompleted) throw new IOException("The broker disconnected without a target exit status.");
        }
        catch (Exception exception) when (exception is not OperationCanceledException || !lifetime.IsCancellationRequested)
        { exited.TrySetException(exception); resumed.TrySetException(exception); }
        catch (OperationCanceledException) when (lifetime.IsCancellationRequested) { }
    }

    public async ValueTask DisposeAsync()
    {
        if (disposing) return;
        disposing = true;
        try
        {
            if (channel is not null) await channel.DisposeAsync(); else await pipe.DisposeAsync();
            if (process is not null)
            {
                using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(5));
                try { await process.WaitForExitAsync(deadline.Token); }
                catch (OperationCanceledException) { process.Kill(entireProcessTree: true); await process.WaitForExitAsync(); }
            }
        }
        finally
        {
            lifetime.Cancel(); await reader; process?.Dispose(); lifetime.Dispose();
        }
    }
}
