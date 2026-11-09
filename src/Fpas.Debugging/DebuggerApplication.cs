using Fpas.Debugging.Batch;
using Fpas.Debugging.Failures;

namespace Fpas.Debugging;

public static class DebuggerApplication
{
    /// <summary>Run once with inherited or captured console streams, saving the first unhandled exception as debug JSON.</summary>
    /// <param name="options">Target, console and report options; an existing report destination is rejected.</param>
    /// <param name="dotnet">Absolute path to the installed .NET host.</param>
    /// <param name="token">Cancellation for setup and execution; owned cleanup and report persistence still complete.</param>
    /// <returns>The natural target exit, captured output and saved report path, if any.</returns>
    public static Task<FailureRunResult> RunWithFailureReportAsync(FailureRunOptions options, string dotnet, CancellationToken token = default)
        => FailureRunner.Run(options, dotnet, token);

    public static System.Text.Json.Nodes.JsonNode ReportSchema()
        => System.Text.Json.Schema.JsonSchemaExporter.GetJsonSchemaAsNode(BatchJson.Default.BatchReport);
    public static Task<int> RunDapAsync(string dotnet, Stream input, Stream output, CancellationToken token = default)
        => new Dap.DapServer(dotnet, input, output, token).Run();
    public static async Task<int> RunBatchAsync(BatchOptions options, string dotnet, Stream output, CancellationToken token = default)
    {
        var report = await BatchRunner.Run(options, dotnet, token);
        var bytes = report.Serialize();
        await output.WriteAsync(bytes, CancellationToken.None);
        await output.WriteAsync("\n"u8.ToArray(), CancellationToken.None);
        await output.FlushAsync(CancellationToken.None);
        return report.ExitCode;
    }

    public static async Task<int> ArgumentErrorAsync(string message, Stream output, string help = "fpas debug --help")
    {
        var report = new BatchReport { ExitCode = 2 };
        report.Issue("invalid_argument", "arguments", message + " See " + help + ".");
        await output.WriteAsync(report.Serialize()); await output.WriteAsync("\n"u8.ToArray()); await output.FlushAsync();
        return 2;
    }
}
