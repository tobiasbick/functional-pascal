// [Documentation](../../docs/architecture/debugger-core.md)
using Fpas.Debugging.Batch;
using Fpas.Debugging.Failures;

namespace Fpas.Debugging;

/// <summary>Entry points for the FPAS debugger: batch inspection, DAP sessions and failure-report runs.</summary>
public static class DebuggerApplication
{
    /// <summary>Run once with inherited or captured console streams, saving the first unhandled exception as debug JSON.</summary>
    /// <param name="options">Target, console and report options; an existing report destination is rejected.</param>
    /// <param name="dotnet">Absolute path to the installed .NET host.</param>
    /// <param name="token">Cancellation for setup and execution; owned cleanup and report persistence still complete.</param>
    /// <returns>The natural target exit, captured output and saved report path, if any.</returns>
    public static Task<FailureRunResult> RunWithFailureReportAsync(FailureRunOptions options, string dotnet, CancellationToken token = default)
        => FailureRunner.Run(options, dotnet, token);

    /// <summary>Describe the batch debug report format.</summary>
    /// <returns>The JSON schema of the batch report.</returns>
    public static System.Text.Json.Nodes.JsonNode ReportSchema()
        => System.Text.Json.Schema.JsonSchemaExporter.GetJsonSchemaAsNode(BatchJson.Default.BatchReport);
    /// <summary>Serve one Debug Adapter Protocol session over the given streams.</summary>
    /// <param name="dotnet">Absolute path to the installed .NET host.</param>
    /// <param name="input">Client-to-adapter DAP byte stream.</param>
    /// <param name="output">Adapter-to-client DAP byte stream.</param>
    /// <param name="token">Cancellation that ends the session and its owned target.</param>
    /// <returns>0 after an orderly session, 1 after a transport or cleanup failure, or 130 when cancelled.</returns>
    public static Task<int> RunDapAsync(string dotnet, Stream input, Stream output, CancellationToken token = default)
        => new Dap.DapServer(dotnet, input, output, token).Run();
    /// <summary>Run one batch debug invocation and write its JSON report followed by a newline.</summary>
    /// <param name="options">Target, breakpoint, expression and limit options.</param>
    /// <param name="dotnet">Absolute path to the installed .NET host.</param>
    /// <param name="output">Destination of the JSON report.</param>
    /// <param name="token">Cancellation for the invocation; the report is still written.</param>
    /// <returns>The report exit code.</returns>
    public static async Task<int> RunBatchAsync(BatchOptions options, string dotnet, Stream output, CancellationToken token = default)
    {
        var report = await BatchRunner.Run(options, dotnet, token);
        var bytes = report.Serialize();
        await output.WriteAsync(bytes, CancellationToken.None);
        await output.WriteAsync("\n"u8.ToArray(), CancellationToken.None);
        await output.FlushAsync(CancellationToken.None);
        return report.ExitCode;
    }

    /// <summary>Write a batch report that describes an invalid command line.</summary>
    /// <param name="message">Description of the argument error.</param>
    /// <param name="output">Destination of the JSON report.</param>
    /// <param name="help">Command that shows usage help.</param>
    /// <returns>The argument error exit code 2.</returns>
    public static async Task<int> ArgumentErrorAsync(string message, Stream output, string help = "fpas debug --help")
    {
        var report = new BatchReport { ExitCode = 2 };
        report.Issue("invalid_argument", "arguments", message + " See " + help + ".");
        await output.WriteAsync(report.Serialize()); await output.WriteAsync("\n"u8.ToArray()); await output.FlushAsync();
        return 2;
    }
}
