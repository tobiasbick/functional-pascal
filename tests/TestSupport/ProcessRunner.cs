using System.Diagnostics;
using System.Text;

namespace Fpas.Tests.Support;

internal sealed record ProcessResult(int ExitCode, string Output, string Error);

internal static class ProcessRunner
{
    internal static async Task<ProcessResult> RunAsync(IEnumerable<string> arguments, string? input = null,
        string? workingDirectory = null, IReadOnlyDictionary<string, string?>? environment = null, TimeSpan? timeLimit = null)
    {
        var start = new ProcessStartInfo(TestEnvironment.DotnetHost)
        {
            RedirectStandardOutput = true,
            RedirectStandardError = true,
            RedirectStandardInput = input is not null,
            StandardOutputEncoding = Encoding.UTF8,
            StandardErrorEncoding = Encoding.UTF8,
            UseShellExecute = false
        };
        if (input is not null) start.StandardInputEncoding = new UTF8Encoding(false);
        // Reused build workers can inherit redirected pipes beyond the command's lifetime.
        start.Environment["MSBUILDDISABLENODEREUSE"] = "1";
        start.Environment["DOTNET_CLI_USE_MSBUILD_SERVER"] = "0";
        if (workingDirectory is not null)
            start.WorkingDirectory = workingDirectory;
        if (environment is not null)
            foreach (var (key, value) in environment)
                if (value is null) start.Environment.Remove(key);
                else start.Environment[key] = value;
        foreach (var argument in arguments)
            start.ArgumentList.Add(argument);
        using var process = Process.Start(start) ?? throw new InvalidOperationException("Unable to start dotnet.");
        using var timeout = new CancellationTokenSource(timeLimit ?? TimeSpan.FromSeconds(30));
        var output = process.StandardOutput.ReadToEndAsync(timeout.Token);
        var error = process.StandardError.ReadToEndAsync(timeout.Token);
        try
        {
            if (input is not null)
            {
                await process.StandardInput.WriteAsync(input.AsMemory(), timeout.Token);
                process.StandardInput.Close();
            }
            await process.WaitForExitAsync(timeout.Token);
            return new(process.ExitCode, await output, await error);
        }
        catch (OperationCanceledException)
        {
            var exited = process.HasExited;
            if (!exited) process.Kill(entireProcessTree: true);
            await process.WaitForExitAsync();
            throw new TimeoutException($"The '{start.ArgumentList.FirstOrDefault()}' process exceeded its test time limit " +
                (exited ? "after exiting; redirected streams remained open." : "before exiting."));
        }
    }
}
