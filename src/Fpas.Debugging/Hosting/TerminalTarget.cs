using System.IO.Pipes;
using System.Text.Json;
using Fpas.Debugging.Core;
using Fpas.Processes.Terminal;

namespace Fpas.Debugging.Hosting;

internal static class TerminalTarget
{
    internal static async Task<int> Run(BrokerLaunch target, long request, WorkerChannel debugger, CancellationToken startup)
    {
        var config = target.Terminal!;
        using var lifetime = new CancellationTokenSource();
        await using var pipe = new NamedPipeClientStream(".", config.Pipe, PipeDirection.InOut, PipeOptions.Asynchronous | PipeOptions.CurrentUserOnly);
        await pipe.ConnectAsync(startup);
        await using var controller = new WorkerChannel(pipe);
        if (config.TuiPipe is { } tuiPipe) Environment.SetEnvironmentVariable("FPAS_TUI_CONTROL_PIPE", tuiPipe);
        await using var terminal = PseudoTerminal.StartSuspended(target.Dotnet, [target.Program, .. target.Arguments ?? []], target.Cwd, config.Columns, config.Rows);
        var token = lifetime.Token;
        var output = Task.Run(() => ReadOutput(terminal, controller, token));
        var controls = Task.Run(() => Control(terminal, controller, token));
        var commands = Task.Run(() => Start(terminal, debugger, token));
        try
        {
            await debugger.Send(new(request, "reply", "create", JsonSerializer.SerializeToElement(new BrokerTarget(terminal.Id), WorkerJson.Default.BrokerTarget)), startup);
            var next = await Task.WhenAny(terminal.Completion, commands, controls, output);
            if (next != terminal.Completion && !terminal.Completion.IsCompleted)
            {
                // EOF or a failed peer ends the ownership of the target.
                await terminal.TerminateAsync();
                await next;
            }
            var code = await terminal.Completion;
            await debugger.Send(new(0, "event", "exit", JsonSerializer.SerializeToElement(new BrokerExit(code), WorkerJson.Default.BrokerExit)), token);
            await terminal.TerminateAsync();
            await output.WaitAsync(TimeSpan.FromSeconds(5));
            return 0;
        }
        finally
        {
            try { await terminal.TerminateAsync(); }
            finally { lifetime.Cancel(); }
            try { await Task.WhenAll(output, controls, commands).WaitAsync(TimeSpan.FromSeconds(5)); }
            catch (OperationCanceledException) when (token.IsCancellationRequested) { }
        }
    }

    private static async Task Start(PseudoTerminal terminal, WorkerChannel debugger, CancellationToken token)
    {
        var released = false;
        while (await debugger.Read(token) is { } message)
        {
            if (released || message.Kind != "request" || message.Operation != "resume")
                throw new IOException("Invalid terminal target command.");
            terminal.ReleaseStartup();
            released = true;
            await debugger.Send(new(message.Id, "reply", message.Operation), token);
        }
    }

    private static async Task Control(PseudoTerminal terminal, WorkerChannel controller, CancellationToken token)
    {
        while (await controller.Read(token) is { } message)
        {
            if (message.Kind != "request" || message.Id <= 0) throw new IOException("Invalid terminal request.");
            try
            {
                switch (message.Operation)
                {
                    case "resize":
                        var size = message.Payload?.Deserialize(WorkerJson.Default.TerminalSize) ?? throw new ArgumentException("Missing terminal dimensions.");
                        terminal.Resize(size.Columns, size.Rows);
                        break;
                    case "input":
                        var bytes = message.Payload?.Deserialize(WorkerJson.Default.ByteArray) ?? throw new ArgumentException("Missing terminal input.");
                        if (bytes.Length > 8192) throw new ArgumentException("A terminal input request can carry up to 8192 bytes.");
                        await terminal.Input.WriteAsync(bytes, token);
                        await terminal.Input.FlushAsync(token);
                        break;
                    default: throw new ArgumentException("Unknown terminal operation.");
                }
                await controller.Send(new(message.Id, "reply", message.Operation), token);
            }
            catch (ArgumentException exception)
            {
                await controller.Send(new(message.Id, "reply", message.Operation, Error: new("invalid_argument", exception.Message)), token);
            }
        }
    }

    private static async Task ReadOutput(PseudoTerminal terminal, WorkerChannel controller, CancellationToken token)
    {
        var bytes = new byte[8192];
        var forwarding = true;
        while (true)
        {
            int count;
            try { count = await terminal.Output.ReadAsync(bytes, token); }
            catch (IOException exception) when (OperatingSystem.IsLinux() && (exception.HResult & 0xffff) == 5)
            { return; } // Linux returns EIO after the last slave closes.
            if (count == 0) return;
            if (forwarding)
            {
                try
                {
                    using var deadline = CancellationTokenSource.CreateLinkedTokenSource(token);
                    deadline.CancelAfter(TimeSpan.FromSeconds(2));
                    await controller.Send(new(0, "event", "output", JsonSerializer.SerializeToElement(bytes[..count], WorkerJson.Default.ByteArray)), deadline.Token);
                }
                catch (Exception error) when (error is IOException or OperationCanceledException)
                { forwarding = false; }
            }
        }
    }
}
