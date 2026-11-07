using System.CommandLine;
using System.Text;
using System.Text.Json;
using Fpas.Cli.Reporting;
using Fpas.Processes.Terminal;

namespace Fpas.Cli.Terminal;

internal sealed class TerminalCommand
{
    private readonly Argument<string> script = new("script") { Description = "UTF-8 JSON terminal script (schema: fpas schema terminal-script)." };
    private readonly Command test = new("test", "Execute ordered send/expect/resize/exit steps in a private PTY or ConPTY. Always emits one JSON report.\n\nExamples:\n  fpas terminal test terminal-test.json\n");
    internal Command Command { get; } = new("terminal", "Controlled terminal interaction without a visible window.\n\nExamples:\n  fpas terminal test terminal-test.json\n\n  fpas schema terminal-script\n");
    internal TerminalCommand() { Command.Subcommands.Add(test); test.Arguments.Add(script); test.SetAction(Execute); }
    internal bool Owns(Command command) => command == Command || command == test;
    internal int ArgumentError(string message) => OperationReport.Fail("terminal test", "invalid_argument", message + " See fpas terminal test --help.", 2);

    private async Task<int> Execute(ParseResult result, CancellationToken cancellation)
    {
        PseudoTerminal? child = null; Task? reading = null; CancellationTokenSource? deadline = null, linked = null;
        var text = new StringBuilder(); var completed = 0; var truncated = false; int? exit = null; int? pid = null;
        OperationReport report = new("terminal test", false, "failed"); var code = 1;
        try
        {
            var file = Path.GetFullPath(result.GetValue(script)!);
            if (new FileInfo(file).Length > 65_536) throw new ArgumentException("Terminal scripts are limited to 64 KiB.");
            var value = JsonSerializer.Deserialize(File.ReadAllText(file, new UTF8Encoding(false, true)), OperationJson.Default.TerminalScript)
                ?? throw new ArgumentException("Supply a terminal-script JSON object.");
            Validate(value);
            deadline = new(TimeSpan.FromSeconds(value.TimeoutSeconds)); linked = CancellationTokenSource.CreateLinkedTokenSource(cancellation, deadline.Token);
            var token = linked.Token;
            child = PseudoTerminal.Start(value.Executable, value.Arguments, Path.GetFullPath(value.Directory ?? Path.GetDirectoryName(file)!), value.Columns, value.Rows);
            pid = child.Id;
            var current = child;
            reading = Task.Run(() =>
            {
                using var reader = new StreamReader(current.Output, new UTF8Encoding(false, false), leaveOpen: true);
                var buffer = new char[4096]; var bytes = 0;
                try
                {
                    int count;
                    while ((count = reader.Read(buffer, 0, buffer.Length)) > 0)
                    {
                        lock (text)
                        {
                            bytes += Encoding.UTF8.GetByteCount(buffer.AsSpan(0, count));
                            if (bytes > value.OutputBytes) { truncated = true; linked.Cancel(); break; }
                            text.Append(buffer, 0, count);
                        }
                    }
                }
                catch (IOException failure) when (OperatingSystem.IsLinux() && (failure.HResult & 0xffff) == 5) { }
            });
            var cursor = 0;
            foreach (var step in value.Steps)
            {
                token.ThrowIfCancellationRequested();
                if (step.Action == "send") { await child.Input.WriteAsync(Encoding.UTF8.GetBytes(step.Text!), token); await child.Input.FlushAsync(token); }
                else if (step.Action == "resize") child.Resize(step.Columns!.Value, step.Rows!.Value);
                else if (step.Action == "exit")
                {
                    exit = await child.Completion.WaitAsync(token);
                    if (exit != step.ExitCode) throw new IOException("Target exit did not match the script's expected exit code.");
                }
                else
                {
                    while (true)
                    {
                        lock (text)
                        {
                            var index = text.ToString().IndexOf(step.Text!, cursor, StringComparison.Ordinal);
                            if (index >= 0) { cursor = index + step.Text!.Length; break; }
                        }
                        if (reading.IsCompleted) { await reading; throw new IOException("Terminal closed before the expected output."); }
                        await Task.Delay(10, token);
                    }
                }
                completed++;
            }
            report = new("terminal test", true, "ok"); code = 0;
        }
        catch (OperationCanceledException)
        {
            var reason = cancellation.IsCancellationRequested ? "cancelled" : truncated ? "output_limit" : "timeout";
            report = new("terminal test", false, reason) { Error = new(reason, "Terminal interaction interrupted.") }; code = cancellation.IsCancellationRequested ? 130 : 1;
        }
        catch (Exception failure) when (failure is not OutOfMemoryException)
        { report = new("terminal test", false, "failed") { Error = new(failure is ArgumentException or JsonException ? "invalid_argument" : "terminal_failed", failure.Message) }; code = failure is ArgumentException or JsonException ? 2 : 1; }
        finally
        {
            if (child is not null)
            {
                Exception? cleanup = null;
                try { await child.TerminateAsync(); if (reading is not null) await reading.WaitAsync(TimeSpan.FromSeconds(5)); }
                catch (Exception error) when (error is not OutOfMemoryException) { cleanup = error; }
                try { await child.DisposeAsync(); } catch (Exception error) when (error is not OutOfMemoryException) { cleanup ??= error; }
                if (cleanup is not null) { report = new("terminal test", false, "failed") { Error = new("cleanup_failed", cleanup.Message) }; code = 1; }
            }
            linked?.Dispose(); deadline?.Dispose();
        }
        if (truncated && report.Success)
        { report = new("terminal test", false, "output_limit") { Error = new("output_limit", "Terminal output exceeded outputBytes.") }; code = 1; }
        string transcript; lock (text) transcript = text.ToString();
        return (report with { Terminal = pid is null ? null : new(pid.Value, exit, completed, transcript, truncated) }).Write(code);
    }

    private static void Validate(TerminalScript value)
    {
        if (value.SchemaVersion != 1) throw new ArgumentException("schemaVersion must be 1.");
        if (value.TimeoutSeconds is < 1 or > 3600) throw new ArgumentException("timeoutSeconds must be 1..3600.");
        if (value.OutputBytes is < 1 or > 1_048_576) throw new ArgumentException("outputBytes must be 1..1048576.");
        if (value.Arguments is null || value.Steps is null || value.Steps.Length is < 1 or > 256) throw new ArgumentException("Supply arguments and 1..256 steps.");
        if (value.Steps.Any(step => step is null)) throw new ArgumentException("Steps must be objects.");
        if (value.Steps[^1].Action != "exit") throw new ArgumentException("The final step must be exit.");
        foreach (var step in value.Steps)
        {
            if (step.Action is "send" or "expect")
            { if (string.IsNullOrEmpty(step.Text) || step.Columns is not null || step.Rows is not null || step.ExitCode is not null) throw new ArgumentException("send/expect require nonempty text only."); }
            else if (step.Action == "resize")
            { if (step.Columns is not (>= 1 and <= 500) || step.Rows is not (>= 1 and <= 300) || step.Text is not null || step.ExitCode is not null) throw new ArgumentException("resize requires columns 1..500 and rows 1..300 only."); }
            else if (step.Action == "exit")
            { if (step.ExitCode is null || step.Text is not null || step.Columns is not null || step.Rows is not null) throw new ArgumentException("exit requires exitCode only."); }
            else throw new ArgumentException("Unknown action. Select send, expect, resize or exit.");
        }
    }
}
