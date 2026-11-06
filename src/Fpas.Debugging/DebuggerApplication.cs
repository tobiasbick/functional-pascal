using Fpas.Debugging.Batch;

namespace Fpas.Debugging;

public static class DebuggerApplication
{
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

    public static async Task<int> ArgumentErrorAsync(string message, Stream output)
    {
        var report = new BatchReport { ExitCode = 2 };
        report.Issue("invalid_argument", "arguments", message + " See fpas debug --help.");
        await output.WriteAsync(report.Serialize()); await output.WriteAsync("\n"u8.ToArray()); await output.FlushAsync();
        return 2;
    }
}
