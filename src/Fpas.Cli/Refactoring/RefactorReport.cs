using Fpas.Cli.Analysis;
using Fpas.Cli.Diagnostics;
using Fpas.Compiler;
using Fpas.Workspaces.Refactoring;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace Fpas.Cli.Refactoring;

internal sealed record RefactorFile(string Path, bool Written);
internal sealed record RefactorReport(bool Success, string Status, string? PlanPath, RenamePlan? Plan,
    IReadOnlyList<RefactorFile> Files, bool Rechecked, IReadOnlyList<AnalysisDiagnostic> Diagnostics,
    AnalysisError? Error = null, IReadOnlyList<string>? RecoveryBackups = null)
{
    internal byte[] Bytes() => JsonSerializer.SerializeToUtf8Bytes(this, RefactorJson.Default.RefactorReport);
    internal static RefactorReport Failure(string code, string message) => new(false, code == "cancelled" ? "cancelled" : "failed", null, null, [], false, [], new(code, message));
    internal bool Write(bool json)
    {
        if (json)
        {
            var bytes = Bytes(); var limited = bytes.Length > RenamePlanJson.MaximumBytes;
            if (limited) bytes = (this with { Success = false, Plan = null, Error = new("result_limit", "Refactoring response exceeds 4 MiB.", true) }).Bytes();
            using var output = Console.OpenStandardOutput(); output.Write(bytes); Console.WriteLine(); return !limited;
        }
        DiagnosticRenderer.Write(Diagnostics);
        if (Error is { } error) Console.Error.WriteLine(DiagnosticRenderer.Safe(error.Code + ": " + error.Message));
        if (Plan is { } plan)
        {
            Console.WriteLine(DiagnosticRenderer.Safe($"Rename to {plan.NewName}: {Status}; {plan.EditCount} edits in {plan.Files.Count} files. Plan: {PlanPath}"));
            foreach (var file in plan.Files)
                foreach (var edit in file.Edits)
                    Console.WriteLine(DiagnosticRenderer.Safe($"  {file.Path}({edit.Location.Line},{edit.Location.Column}): {edit.ExpectedText} -> {edit.NewText}"));
            if (plan.ChangesPublicApi) Console.WriteLine("Public API changes: only the analyzed FPAS sources are renamed; external consumers are outside this plan.");
        }
        foreach (var backup in RecoveryBackups ?? []) Console.Error.WriteLine("Recovery file: " + DiagnosticRenderer.Safe(backup));
        return true;
    }
}

[JsonSourceGenerationOptions(PropertyNamingPolicy = JsonKnownNamingPolicy.CamelCase, DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull,
    Converters = [typeof(SeverityConverter), typeof(ApplicabilityConverter)])]
[JsonSerializable(typeof(RefactorReport))]
internal partial class RefactorJson : JsonSerializerContext;
