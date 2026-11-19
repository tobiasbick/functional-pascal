// [Documentation](../../../docs/pascal/tools/refactoring.md)
using Fpas.Cli.Analysis;
using Fpas.Cli.Diagnostics;
using Fpas.Compiler;
using Fpas.Workspaces.Refactoring;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace Fpas.Cli.Refactoring;

internal sealed record RefactorFile(string Path, bool Written);
internal sealed record RefactorReport(bool Success, string Status, string? PlanPath, RefactorPlan? Plan,
    IReadOnlyList<RefactorFile> Files, bool Rechecked, IReadOnlyList<AnalysisDiagnostic> Diagnostics,
    AnalysisError? Error = null, IReadOnlyList<string>? RecoveryBackups = null)
{
    internal byte[] Bytes() => JsonSerializer.SerializeToUtf8Bytes(this, RefactorJson.Default.RefactorReport);
    internal static RefactorReport Failure(string code, string message) => new(false, code == "cancelled" ? "cancelled" : "failed", null, null, [], false, [], new(code, message));
    internal bool Write(bool json)
    {
        if (json)
        {
            var bytes = Bytes(); var limited = bytes.Length > RefactorPlanJson.MaximumBytes;
            if (limited) bytes = (this with { Success = false, Plan = null, Error = new("result_limit", "Refactoring response exceeds 4 MiB.", true) }).Bytes();
            using var output = Console.OpenStandardOutput(); output.Write(bytes); Console.WriteLine(); return !limited;
        }
        DiagnosticRenderer.Write(Diagnostics);
        if (Error is { } error) Console.Error.WriteLine(DiagnosticRenderer.Safe(error.Code + ": " + error.Message));
        if (Plan is { } plan)
        {
            var operation = plan.Kind switch
            {
                "rename" => "Rename to " + plan.NewName,
                "add-parameter" => "Add parameter " + plan.AddedParameter!.Name,
                "remove-parameter" => "Remove parameter " + plan.Targets[0].Name,
                "implement-protocol" => "Implement " + plan.Protocol!.Type + " on " + plan.Targets[0].Name,
                "change-signature" when plan.SignatureChange?.Order is not null => "Reorder parameters of " + plan.Targets[0].Name,
                "change-signature" => "Change signature of " + plan.Targets[0].Name,
                "inline-variable" => "Inline " + plan.Targets[0].Name,
                "extract-variable" => "Extract variable " + plan.NewName,
                "organize-imports" => "Organize imports",
                "name-arguments" => "Name arguments",
                _ => plan.Kind
            };
            Console.WriteLine(DiagnosticRenderer.Safe($"{operation}: {Status}; {plan.EditCount} edits in {plan.Files.Count} files. Plan: {PlanPath}"));
            foreach (var file in plan.Files)
                foreach (var edit in file.Edits)
                    Console.WriteLine(DiagnosticRenderer.Safe($"  {file.Path}({edit.Location.Line},{edit.Location.Column}): {edit.ExpectedText} -> {edit.NewText}"));
            foreach (var note in plan.Notes ?? []) Console.WriteLine(DiagnosticRenderer.Safe(note));
            if (plan.ChangesPublicApi) Console.WriteLine("Public API changes affect the analyzed FPAS sources.");
        }
        foreach (var backup in RecoveryBackups ?? []) Console.Error.WriteLine("Recovery file: " + DiagnosticRenderer.Safe(backup));
        return true;
    }
}

[JsonSourceGenerationOptions(PropertyNamingPolicy = JsonKnownNamingPolicy.CamelCase, DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull,
    Converters = [typeof(SeverityConverter), typeof(ApplicabilityConverter)])]
[JsonSerializable(typeof(RefactorReport))]
internal partial class RefactorJson : JsonSerializerContext;
