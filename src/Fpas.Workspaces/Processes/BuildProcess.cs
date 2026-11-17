using Fpas.Processes;
using System.Text;

namespace Fpas.Workspaces.Processes;

/// <summary>Contains an owned process's exit status and bounded captured output.</summary>
/// <param name="ExitCode">Native exit code, or 130 for captured cancellation.</param>
/// <param name="Output">Captured standard output; empty when streamed live.</param>
/// <param name="Error">Captured standard error; empty when streamed live.</param>
/// <param name="Truncated">Whether captured output exceeded its shared byte budget.</param>
public sealed record BuildResult(int ExitCode, string Output, string Error, bool Truncated)
{
    /// <summary>Gets structured build diagnostics when an MSBuild logger was requested.</summary>
    [System.Text.Json.Serialization.JsonIgnore(Condition = System.Text.Json.Serialization.JsonIgnoreCondition.WhenWritingNull)]
    public IReadOnlyList<ProjectBuildDiagnostic>? Diagnostics { get; init; }
    /// <summary>Gets whether the structured diagnostic stream exceeded its own budget.</summary>
    [System.Text.Json.Serialization.JsonIgnore(Condition = System.Text.Json.Serialization.JsonIgnoreCondition.WhenWritingDefault)]
    public bool DiagnosticsTruncated { get; init; }
}

/// <summary>Receives live process output without buffering or truncation.</summary>
/// <param name="StandardOutput">Writer for standard output.</param>
/// <param name="StandardError">Writer for standard error.</param>
public sealed record ProcessOutput(TextWriter StandardOutput, TextWriter StandardError);

/// <summary>Runs dotnet and explicit tools with bounded output and owned descendant cleanup.</summary>
public static class BuildProcess
{
    /// <summary>Runs the selected dotnet host and joins owned descendants before returning.</summary>
    /// <param name="cwd">Process working directory.</param>
    /// <param name="arguments">Arguments passed without shell interpretation.</param>
    /// <param name="token">Cancels execution and terminates owned processes.</param>
    /// <param name="inheritInput">Whether to inherit caller stdin; otherwise supplies EOF.</param>
    /// <param name="buildEnvironment">Whether to disable build servers and select the bundled compiler.</param>
    /// <param name="environment">Additional environment overrides.</param>
    /// <param name="captureCancellation">Whether cancellation returns exit 130 instead of throwing.</param>
    /// <param name="liveOutput">Optional writers receiving unbounded live output instead of capture.</param>
    /// <returns>Exit status and at most 1 MiB of combined captured output.</returns>
    public static Task<BuildResult> Run(string cwd, IReadOnlyList<string> arguments, CancellationToken token, bool inheritInput = false, bool buildEnvironment = true,
        IReadOnlyDictionary<string, string>? environment = null, bool captureCancellation = false, ProcessOutput? liveOutput = null)
    {
        token.ThrowIfCancellationRequested();
        return Capture(DotnetHost(), cwd, arguments, token, inheritInput, buildEnvironment, environment,
            captureCancellation: captureCancellation, liveOutput: liveOutput);
    }

    /// <summary>Runs an explicit executable without build-specific environment changes.</summary>
    /// <param name="path">Absolute executable path.</param>
    /// <param name="cwd">Process working directory.</param>
    /// <param name="arguments">Arguments passed without shell interpretation.</param>
    /// <param name="token">Cancels execution and terminates owned processes.</param>
    /// <param name="outputLimit">Shared output byte budget, 1..4 MiB.</param>
    /// <returns>Exit status and bounded captured streams.</returns>
    public static Task<BuildResult> RunTool(string path, string cwd, IReadOnlyList<string> arguments, CancellationToken token, int outputLimit = 1_048_576)
        => Capture(path, cwd, arguments, token, inheritInput: false, buildEnvironment: false, outputLimit: outputLimit);

    private static async Task<BuildResult> Capture(string path, string cwd, IReadOnlyList<string> arguments, CancellationToken token, bool inheritInput, bool buildEnvironment,
        IReadOnlyDictionary<string, string>? environment = null, int outputLimit = 1_048_576, bool captureCancellation = false,
        ProcessOutput? liveOutput = null)
    {
        if (outputLimit is < 1 or > 4_194_304) throw new ArgumentOutOfRangeException(nameof(outputLimit));
        token.ThrowIfCancellationRequested();
        var variables = new Dictionary<string, string>(environment ?? new Dictionary<string, string>());
        if (buildEnvironment) { variables["MSBUILDDISABLENODEREUSE"] = "1"; variables["DOTNET_CLI_USE_MSBUILD_SERVER"] = "0"; }
        if (buildEnvironment && Fpas.Deployment.DistributionAssets.IsBundled)
            variables["FpasCompilerPath"] = Environment.ProcessPath!;
        await using var process = OwnedProcess.Start(path, arguments, cwd, variables, inheritInput ? ProcessStdio.CaptureOutput : ProcessStdio.Capture);
        process.Input.Dispose();
        using var stdout = new StreamReader(process.Output, Encoding.UTF8);
        using var stderr = new StreamReader(process.Error, Encoding.UTF8);
        long outputBytes = 0;
        async Task<string> Drain(StreamReader reader, TextWriter? destination)
        {
            var buffer = new char[4096];
            var result = destination is null ? new StringBuilder() : null;
            int count;
            while ((count = await reader.ReadAsync(buffer, captureCancellation ? CancellationToken.None : token)) > 0)
            {
                if (destination is null)
                {
                    var bytes = Encoding.UTF8.GetByteCount(buffer.AsSpan(0, count));
                    if (Interlocked.Add(ref outputBytes, bytes) <= outputLimit) result!.Append(buffer, 0, count);
                }
                else
                {
                    try { destination.Write(buffer, 0, count); }
                    catch (IOException)
                    {
                        await process.TerminateAsync();
                        throw;
                    }
                }
            }
            return result?.ToString() ?? "";
        }
        var output = Drain(stdout, liveOutput?.StandardOutput);
        var error = Drain(stderr, liveOutput?.StandardError);
        try
        {
            await process.WaitForExit(token);
            var exitCode = process.ExitCode;
            await process.TerminateAsync();
            return new(exitCode, await output, await error, outputBytes > outputLimit);
        }
        catch (OperationCanceledException) when (captureCancellation && token.IsCancellationRequested)
        {
            await process.TerminateAsync();
            return new(130, await output, await error, outputBytes > outputLimit);
        }
        finally
        {
            await process.DisposeAsync();
            try { await Task.WhenAll(output, error); }
            catch (Exception exception) when (token.IsCancellationRequested && exception is OperationCanceledException or IOException or ObjectDisposedException) { }
        }
    }

    /// <summary>Finds dotnet using DOTNET_HOST_PATH, DOTNET_ROOT and then PATH.</summary>
    /// <returns>The selected absolute host path, or an I/O failure when unavailable.</returns>
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
