// [Documentation](../../../docs/pascal/program-structure/cli.md#outline-one-file)
using Fpas.Cli.Diagnostics;
using Fpas.Compiler;
using Fpas.Compiler.Analysis;
using Fpas.Workspaces.Loading;
using System.CommandLine;
using System.Security.Cryptography;
using System.Text;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace Fpas.Cli.Analysis;

internal sealed class OutlineCommand
{
    private static readonly UTF8Encoding Utf8 = new(false, true);
    private readonly Argument<string> file = new("file") { Description = "Path to one FPAS source file." };
    private readonly Option<int> limit = new("--limit") { DefaultValueFactory = _ => 200, Description = "Maximum declarations (1..1000)." };
    private readonly Option<int> offset = new("--offset") { DefaultValueFactory = _ => 0, Description = "Skip this many declarations in source order." };
    private readonly Option<int> timeout = new("--timeout") { DefaultValueFactory = _ => 10, Description = "Overall deadline in seconds (1..3600)." };

    internal Command Command { get; } = new("outline", """
        Show the declarations and source spans recognized by the FPAS parser.
        Reads one source file and returns a syntax-only JSON report.

        Examples:
          fpas query outline examples/basics/hello/main.fpas
        """);

    internal OutlineCommand()
    {
        limit.Range(1, 1000);
        offset.Range(0, int.MaxValue);
        timeout.Range(1, 3600);
        Command.Arguments.Add(file);
        foreach (var option in new[] { limit, offset, timeout }) Command.Options.Add(option);
        Command.SetAction(Execute);
    }

    internal Task<int> ArgumentError(ParseResult parse, string message)
        => OutlineReport.Failure(CommandArguments.Supplied(parse, file), new("invalid_argument", message)).WriteAsync(2);

    private async Task<int> Execute(ParseResult parse, CancellationToken token)
    {
        var path = parse.GetValue(file)!;
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(parse.GetValue(timeout)));
        using var combined = CancellationTokenSource.CreateLinkedTokenSource(token, deadline.Token);
        try
        {
            path = Path.GetFullPath(path);
            var input = await BoundedFile.ReadAsync(path, 4 * 1024 * 1024, combined.Token);
            var syntax = SyntaxAnalysis.Structure(new(path, Utf8.GetString(input)), combined.Token);
            var skip = parse.GetValue(offset);
            var ordered = syntax.Declarations.Where(item => item.Kind != "parameter").OrderBy(item => item.NameSpan.Offset).ToArray();
            var page = ordered.Skip(skip).Take(parse.GetValue(limit)).ToArray();
            var complete = syntax.Diagnostics.Count == 0;
            var next = (long)skip + page.Length;
            combined.Token.ThrowIfCancellationRequested();
            return await new OutlineReport(complete ? "ok" : "partial", path, Convert.ToHexString(SHA256.HashData(input)),
                complete, ordered.Length, skip, next < ordered.Length ? (int)next : null, page, syntax.Diagnostics, null).WriteAsync(0);
        }
        catch (OperationCanceledException) when (combined.IsCancellationRequested)
        {
            return await OutlineReport.Failure(path, token.IsCancellationRequested ? new("cancelled", "Outline cancelled.") :
                new("timeout", "Outline exceeded --timeout.")).WriteAsync(token.IsCancellationRequested ? 130 : 1);
        }
        catch (ContextFailure failure)
        {
            return await OutlineReport.Failure(path, AnalysisError.From(failure)).WriteAsync(failure.Code == "invalid_argument" ? 2 : 1);
        }
        catch (Exception error) when (error is IOException or UnauthorizedAccessException or ArgumentException)
        {
            var invalid = error is ArgumentException;
            return await OutlineReport.Failure(path, new(invalid ? "invalid_argument" : "input_failed", error.Message, InputPath: path))
                .WriteAsync(invalid ? 2 : 1);
        }
    }
}

internal sealed record OutlineReport(string Status, string? File, string? SnapshotId, bool Complete, int TotalCount, int Offset, int? NextOffset,
    IReadOnlyList<SyntaxDeclaration> Declarations, IReadOnlyList<CompilerDiagnostic> Diagnostics, AnalysisError? Error)
{
    public int SchemaVersion { get; init; } = 1;

    internal static OutlineReport Failure(string? file, AnalysisError error)
        => new(error.Code == "cancelled" ? "cancelled" : "failed", file, null, false, 0, 0, null, [], [], error);

    internal async Task<int> WriteAsync(int exit)
    {
        var bytes = JsonSerializer.SerializeToUtf8Bytes(this, OutlineJson.Default.OutlineReport);
        if (bytes.Length > QueryResult.OutputLimit)
        {
            bytes = JsonSerializer.SerializeToUtf8Bytes(Failure(File, new("result_limit", "Outline exceeds 4 MiB. Use a smaller --limit.")),
                OutlineJson.Default.OutlineReport);
            exit = 1;
        }
        await Console.OpenStandardOutput().WriteAsync(bytes);
        Console.WriteLine();
        return exit;
    }
}

[JsonSourceGenerationOptions(PropertyNamingPolicy = JsonKnownNamingPolicy.CamelCase, DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull,
    Converters = [typeof(SeverityConverter), typeof(ApplicabilityConverter)])]
[JsonSerializable(typeof(OutlineReport))]
internal partial class OutlineJson : JsonSerializerContext;
