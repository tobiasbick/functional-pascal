using Fpas.Cli.Diagnostics;
using Fpas.Compiler;
using Fpas.Workspaces.Editing;

namespace Fpas.Cli.Analysis;

internal sealed record CheckFixes(string Mode, string State, string InputSnapshotId, IReadOnlyList<FixSuggestion> Suggestions,
    IReadOnlyList<FixFile> Files, int EditCount, int ReviewCount, bool Rechecked = false, IReadOnlyList<string>? RecoveryBackups = null)
{
    internal static CheckFixes From(DiagnosticFixPlan plan, string snapshot, bool preview) => new(preview ? "preview" : "apply",
        plan.Changes.Count == 0 ? "not_needed" : "planned", snapshot, plan.Suggestions, plan.Files, plan.EditCount,
        plan.Suggestions.Count(suggestion => suggestion.Fix.Applicability == DiagnosticApplicability.Review));

    internal void Write()
    {
        Console.Error.WriteLine($"Automatic fixes: {State}; {EditCount} edits in {Files.Count} files; {ReviewCount} review proposals left untouched.");
        foreach (var suggestion in Suggestions)
            foreach (var edit in suggestion.Fix.Edits)
            {
                var action = suggestion.Fix.Applicability == DiagnosticApplicability.Automatic ? "automatic" : "review only";
                Console.Error.WriteLine(DiagnosticRenderer.Safe($"  {edit.Location.Path}({edit.Location.Line},{edit.Location.Column}): {action}: {suggestion.Fix.Title}"));
                Console.Error.WriteLine("    " + DiagnosticRenderer.Safe(edit.ExpectedText) + " -> " + DiagnosticRenderer.Safe(edit.NewText));
            }
        foreach (var backup in RecoveryBackups ?? []) Console.Error.WriteLine("Recovery backup: " + DiagnosticRenderer.Safe(backup));
    }
}
