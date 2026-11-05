using Fpas.Processes;
using System.Text;

namespace Fpas.Workspaces.Processes;

public sealed record BuildResult(int ExitCode, string Output, string Error, bool Truncated);

public static class BuildProcess
{
    public static Task<BuildResult> Run(string cwd, IReadOnlyList<string> arguments, CancellationToken token, bool inheritInput = false, bool buildEnvironment = true)
    {
        token.ThrowIfCancellationRequested();
        return Capture(DotnetHost(), cwd, arguments, token, inheritInput, buildEnvironment);
    }

    public static Task<BuildResult> RunTool(string path, string cwd, IReadOnlyList<string> arguments, CancellationToken token)
        => Capture(path, cwd, arguments, token, inheritInput: false, buildEnvironment: false);

    private static async Task<BuildResult> Capture(string path, string cwd, IReadOnlyList<string> arguments, CancellationToken token, bool inheritInput, bool buildEnvironment)
    {
        token.ThrowIfCancellationRequested();
        await using var process = OwnedProcess.Start(path, arguments, cwd, buildEnvironment ? new Dictionary<string, string>
        {
            ["MSBUILDDISABLENODEREUSE"] = "1", ["DOTNET_CLI_USE_MSBUILD_SERVER"] = "0"
        } : null, inheritInput ? ProcessStdio.CaptureOutput : ProcessStdio.Capture);
        process.Input.Dispose();
        using var stdout = new StreamReader(process.Output, Encoding.UTF8);
        using var stderr = new StreamReader(process.Error, Encoding.UTF8);
        long captured = 0; var truncated = false;
        async Task<string> Drain(StreamReader reader)
        {
            var buffer = new char[4096]; var result = new StringBuilder(); int count;
            while ((count = await reader.ReadAsync(buffer, token)) > 0)
            {
                var bytes = Encoding.UTF8.GetByteCount(buffer.AsSpan(0, count));
                if (Interlocked.Add(ref captured, bytes) <= 1_048_576) result.Append(buffer, 0, count);
                else truncated = true;
            }
            return result.ToString();
        }
        var output = Drain(stdout); var error = Drain(stderr);
        try
        {
            await process.WaitForExit(token);
            var exitCode = process.ExitCode;
            await process.TerminateAsync();
            return new(exitCode, await output, await error, truncated);
        }
        finally
        {
            await process.DisposeAsync();
            try { await Task.WhenAll(output, error); }
            catch (Exception exception) when (token.IsCancellationRequested && exception is OperationCanceledException or IOException or ObjectDisposedException) { }
        }
    }

    public static string DotnetHost()
    {
        if (Environment.GetEnvironmentVariable("DOTNET_HOST_PATH") is { Length: > 0 } host) return host;
        var root = Environment.GetEnvironmentVariable("DOTNET_ROOT");
        var name = OperatingSystem.IsWindows() ? "dotnet.exe" : "dotnet";
        if (!string.IsNullOrEmpty(root)) return Path.GetFullPath(Path.Combine(root, name));
        foreach (var directory in (Environment.GetEnvironmentVariable("PATH") ?? "").Split(Path.PathSeparator))
        {
            if (directory.Length == 0) continue;
            var candidate = Path.GetFullPath(Path.Combine(directory, name));
            if (File.Exists(candidate)) return candidate;
        }
        throw new IOException("The dotnet host was not found. Set DOTNET_HOST_PATH or install the .NET SDK on PATH.");
    }
}
