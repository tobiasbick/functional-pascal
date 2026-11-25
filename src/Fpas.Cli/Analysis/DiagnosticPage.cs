// [Documentation](../../../docs/pascal/tools/diagnostics.md#filter-and-page-analysis-reports)
using Fpas.Compiler;
using System.CommandLine;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace Fpas.Cli.Analysis;

internal sealed class DiagnosticPageOptions
{
    private readonly Option<string> file = new("--diagnostic-file") { Description = "Report diagnostics for this source or input file." };
    private readonly Option<string> project = new("--diagnostic-project") { Description = "Report diagnostics from this owning project." };
    private readonly Option<int> limit = new("--diagnostic-limit") { DefaultValueFactory = _ => 100, Description = "Maximum diagnostics in the report (0..1000); 0 returns counts only." };
    private readonly Option<int> offset = new("--diagnostic-offset") { DefaultValueFactory = _ => 0, Description = "Skip this many matching diagnostics in snapshot order." };

    internal void AddTo(Command command)
    {
        limit.Range(0, 1000);
        offset.Range(0, int.MaxValue);
        foreach (var option in new Option[] { file, project, limit, offset }) command.Options.Add(option);
    }

    internal bool Requested(ParseResult parse) => parse.GetResult(file) is { Implicit: false } || parse.GetResult(project) is { Implicit: false } ||
        parse.GetResult(limit) is { Implicit: false } || parse.GetResult(offset) is { Implicit: false };

    internal DiagnosticSelection Read(ParseResult parse)
        => new(FullPath(parse.GetValue(file)), FullPath(parse.GetValue(project)), parse.GetValue(limit), parse.GetValue(offset));

    private static string? FullPath(string? path) => path is null ? null : Path.GetFullPath(path);
}

internal sealed record DiagnosticPage(string? File, string? Project, int TotalCount, int MatchedCount, int Offset, int ReturnedCount, int? NextOffset)
{
    /// <summary>Gets the number of returned diagnostics whose details exceeded the byte budget.</summary>
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingDefault)]
    public int SummarizedCount { get; init; }
}

internal sealed record DiagnosticSelection(string? File, string? Project, int Limit, int Offset)
{
    private const int ByteBudget = 1024 * 1024;
    private static StringComparer Paths => OperatingSystem.IsWindows() ? StringComparer.OrdinalIgnoreCase : StringComparer.Ordinal;

    internal (IReadOnlyList<AnalysisDiagnostic> Diagnostics, DiagnosticPage Page) Select(IReadOnlyList<AnalysisDiagnostic> all)
    {
        var matches = all.Where(diagnostic => (Project is null || Paths.Equals(diagnostic.Project, Project)) &&
            (File is null || Paths.Equals(diagnostic.Location?.Path, File) || Paths.Equals(diagnostic.InputPath, File))).ToArray();
        var items = new List<AnalysisDiagnostic>();
        var consumed = 0;
        var summarized = 0;
        var bytes = 2;
        foreach (var diagnostic in matches.Skip(Offset).Take(Limit))
        {
            var item = diagnostic;
            var size = SerializedSize(item);
            if (size > ByteBudget - 2)
            {
                var text = diagnostic.Message;
                if (text.Length > 512) text = text[..512] + "…";
                item = new AnalysisDiagnostic(diagnostic.Project, new CompilerDiagnostic(diagnostic.Code, text, diagnostic.Location, diagnostic.Severity)
                    { InputPath = diagnostic.InputPath }) { DetailsOmitted = true };
                size = SerializedSize(item);
            }
            if (bytes + size > ByteBudget) break;
            bytes += size;
            consumed++;
            if (item.DetailsOmitted) summarized++;
            items.Add(item);
        }
        var next = (long)Offset + consumed;
        return (items, new(File, Project, all.Count, matches.Length, Offset, items.Count,
            Limit > 0 && next < matches.Length ? (int)next : null) { SummarizedCount = summarized });
    }
    private static int SerializedSize(AnalysisDiagnostic diagnostic)
        => JsonSerializer.SerializeToUtf8Bytes(diagnostic, CheckJson.Default.AnalysisDiagnostic).Length + 1;
}
