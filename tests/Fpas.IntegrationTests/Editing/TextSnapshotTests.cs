using Fpas.Compiler;
using Fpas.Tests.Support;
using Fpas.Workspaces.Editing;
using Fpas.Workspaces.Loading;
using Fpas.Workspaces.Snapshots;
using System.Text;

namespace Fpas.IntegrationTests.Editing;

public sealed class TextSnapshotTests
{
    [Fact]
    public void Unsaved_text_edits_preserve_versions_original_offsets_and_detach_mutable_inputs()
    {
        using var directory = new TemporaryDirectory(); var path = directory.File("unsaved.fpas");
        const string text = "\uFEFF// 😀\r\nunit Example; const Value: integer := 1; end unit;\r\n";
        var original = new TextDocumentSnapshot(path, text, 12);
        var documents = new[] { original }; var snapshot = new WorkspaceTextSnapshot(documents);
        var offset = text.IndexOf("integer", StringComparison.Ordinal);
        var edits = new[] { new DiagnosticEdit(new(path, 2, 28, 7, Offset: offset), "integer", "int") };
        var suggestions = new[] { new FixSuggestion("Project", "FP3005", new("Type", DiagnosticApplicability.Automatic, edits)) };
        var plan = DiagnosticFixPlan.Create(snapshot, suggestions);
        documents[0] = new(path, "changed", 13); edits[0] = edits[0] with { NewText = "wrong" }; suggestions[0] = suggestions[0] with { Code = "changed" };
        var result = Assert.Single(plan.Edits.Documents);
        Assert.Equal(text, result.Original.Text); Assert.Equal(12, result.Original.Version);
        Assert.Equal(text.Replace("integer", "int", StringComparison.Ordinal), result.NewText);
        Assert.Equal("int", Assert.Single(plan.Suggestions).Fix.Edits[0].NewText);
        var bytes = Assert.Single(plan.Edits.ToSourceChanges()); bytes.Original[0] = 0; bytes.Replacement[0] = 0;
        var independent = Assert.Single(plan.Edits.ToSourceChanges());
        Assert.Equal(Encoding.UTF8.GetBytes(text), independent.Original);
        Assert.Equal(Encoding.UTF8.GetBytes(result.NewText), independent.Replacement);
        Assert.False(File.Exists(path));
    }

    [Fact]
    public void Linked_owners_share_text_and_reject_conflicting_contents_or_versions()
    {
        var path = Path.GetFullPath("linked.fpas"); var first = new TextDocumentSnapshot(path, "text", 2);
        Assert.Single(new WorkspaceTextSnapshot([first, new(path, "text", 2)]).Documents);
        Assert.Equal("edit_conflict", Assert.Throws<ContextFailure>(() => new WorkspaceTextSnapshot([first, new(path, "other", 2)])).Code);
        Assert.Equal("edit_conflict", Assert.Throws<ContextFailure>(() => new WorkspaceTextSnapshot([first, new(path, "text", 3)])).Code);
    }

    [Fact]
    public void Source_identity_retains_UTF16_code_units_without_replacing_unpaired_surrogates()
    {
        var path = Path.GetFullPath("unsaved.fpas");
        var broken = new TextDocumentSnapshot(path, "\uD800");
        Assert.NotEqual(broken.ContentHash, new TextDocumentSnapshot(path, "\uFFFD").ContentHash);
        Assert.Equal("\uD800", broken.Text);
    }
}
