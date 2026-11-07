using System.Reflection;
using System.Text.Json;
using Fpas.Cli.Reporting;
using Fpas.Workspaces.Processes;

namespace Fpas.Cli.Profiling;

// Trace conversion has no cancellation API. A private owned process bounds that operation too.
internal static class ProfileWorker
{
    private const string Switch = "--internal-profile-summary";
    internal static bool IsInvocation(string[] arguments) => arguments.Length == 4 && arguments[0] == Switch;

    internal static int Run(string[] arguments)
    {
        try
        {
            var limit = int.Parse(arguments[2], System.Globalization.CultureInfo.InvariantCulture);
            if (limit is < 1 or > 100) return 2;
            var value = TraceSummary.Read(arguments[1], limit, CancellationToken.None, arguments[3]);
            Console.WriteLine(JsonSerializer.Serialize(value, OperationJson.Default.TraceSummary)); return 0;
        }
        catch (Exception failure) when (failure is not OutOfMemoryException) { Console.Error.WriteLine(failure.Message); return 1; }
    }

    internal static async Task<TraceSummary> Analyze(string file, int limit, CancellationToken token)
    {
        var directory = Path.Combine(Path.GetTempPath(), "fpas-profile-" + Guid.NewGuid().ToString("N"));
        Directory.CreateDirectory(directory);
        try
        {
            var path = Environment.ProcessPath ?? throw new IOException("Cannot locate the CLI executable.");
            var arguments = new List<string>();
            if (Path.GetFileNameWithoutExtension(path).Equals("dotnet", StringComparison.OrdinalIgnoreCase))
                arguments.Add(Fpas.Deployment.DistributionAssets.Managed(Assembly.GetEntryAssembly()!));
            arguments.AddRange([Switch, Path.GetFullPath(file), limit.ToString(System.Globalization.CultureInfo.InvariantCulture), directory]);
            var process = await BuildProcess.RunTool(path, Environment.CurrentDirectory, arguments, token, outputLimit: 4_194_304);
            if (process.ExitCode != 0 || process.Truncated) throw new IOException(process.Truncated ? "Trace summary exceeded 4 MiB." : "Trace decoding failed: " + process.Error);
            return JsonSerializer.Deserialize(process.Output, OperationJson.Default.TraceSummary) ?? throw new IOException("The trace worker returned no summary.");
        }
        finally { Directory.Delete(directory, recursive: true); }
    }
}
